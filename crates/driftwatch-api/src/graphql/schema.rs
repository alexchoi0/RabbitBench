use async_graphql::{EmptySubscription, Schema};

use super::mutation::MutationRoot;
use super::query::QueryRoot;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema() -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}
