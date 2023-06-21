use async_graphql::{Context, EmptySubscription, Object, Schema};

use crate::coinbase::fetch_coinbase_price;

pub type ServiceSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
pub struct QueryRoot;
pub struct MutationRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self, _ctx: &Context<'_>) -> &'static str {
        "Hello world"
    }
}

#[Object]
impl MutationRoot {
    async fn ticker_price(&self, base: String, quote: String) -> String {
        match fetch_coinbase_price(base.as_str(), quote.as_str()).await {
            Ok(ticker_data) => ticker_data.amount,
            Err(err) => {
                eprintln!("Error retrieving Coinbase amount: {}", err);
                String::from("Failed to fetch ticker price")
            }
        }
    }
}
