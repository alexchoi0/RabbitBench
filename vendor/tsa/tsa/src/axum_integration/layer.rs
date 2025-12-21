use std::sync::Arc;
use tower::Layer;

use crate::{Adapter, Auth, AuthCallbacks};

use super::middleware::SessionMiddleware;

#[derive(Clone)]
pub struct SessionLayer<A: Adapter + Clone, C: AuthCallbacks + Clone> {
    pub(crate) auth: Arc<Auth<A, C>>,
    pub(crate) cookie_name: String,
    pub(crate) use_header: bool,
}

impl<A: Adapter + Clone, C: AuthCallbacks + Clone> SessionLayer<A, C> {
    pub fn new(auth: Auth<A, C>) -> Self {
        Self {
            auth: Arc::new(auth),
            cookie_name: super::extract::DEFAULT_SESSION_COOKIE_NAME.to_string(),
            use_header: true,
        }
    }

    pub fn with_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.cookie_name = name.into();
        self
    }

    pub fn use_header(mut self, use_header: bool) -> Self {
        self.use_header = use_header;
        self
    }
}

impl<S, A, C> Layer<S> for SessionLayer<A, C>
where
    A: Adapter + Clone + 'static,
    C: AuthCallbacks + Clone + 'static,
{
    type Service = SessionMiddleware<S, A, C>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionMiddleware {
            inner,
            auth: self.auth.clone(),
            cookie_name: self.cookie_name.clone(),
            use_header: self.use_header,
        }
    }
}
