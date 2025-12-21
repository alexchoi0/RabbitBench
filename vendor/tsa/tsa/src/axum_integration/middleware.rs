use axum::{body::Body, http::Request, response::Response};
use axum_extra::extract::CookieJar;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;

use crate::{Adapter, Auth, AuthCallbacks};

use super::extract::{extract_session_token_from_cookie, SessionState};

#[derive(Clone)]
pub struct SessionMiddleware<S, A: Adapter + Clone, C: AuthCallbacks + Clone> {
    pub(crate) inner: S,
    pub(crate) auth: Arc<Auth<A, C>>,
    pub(crate) cookie_name: String,
    pub(crate) use_header: bool,
}

impl<S, A, C> Service<Request<Body>> for SessionMiddleware<S, A, C>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
    A: Adapter + Clone + Send + Sync + 'static,
    C: AuthCallbacks + Clone + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let auth = self.auth.clone();
        let cookie_name = self.cookie_name.clone();
        let use_header = self.use_header;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let jar = CookieJar::from_headers(req.headers());

            let mut token = extract_session_token_from_cookie(&jar, &cookie_name);

            if token.is_none() && use_header {
                token = req
                    .headers()
                    .get("authorization")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|h| h.strip_prefix("Bearer "))
                    .map(|s| s.to_string());
            }

            let session_state = if let Some(token) = token {
                match auth.validate_session(&token).await {
                    Ok((user, session)) => SessionState {
                        user: Some(user),
                        session: Some(session),
                    },
                    Err(_) => SessionState {
                        user: None,
                        session: None,
                    },
                }
            } else {
                SessionState {
                    user: None,
                    session: None,
                }
            };

            req.extensions_mut().insert(session_state);

            inner.call(req).await
        })
    }
}
