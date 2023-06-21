use async_graphql::{Context, Object, Schema};
use async_graphql::{EmptyMutation, EmptySubscription};

pub type ServiceSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self, _ctx: &Context<'_>) -> &'static str {
        "Hello world"
    }
}
