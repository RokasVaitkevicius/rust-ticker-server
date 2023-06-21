use async_graphql::{Context, Object, Schema};
use async_graphql::{EmptyMutation, EmptySubscription};

use crate::coinbase::fetch_coinbase_price;

pub type ServiceSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self, _ctx: &Context<'_>) -> &'static str {
        "Hello world"
    }

    async fn ticker_price(&self, _ctx: &Context<'_>) -> String {
        match fetch_coinbase_price().await {
            Ok(tickerData) => {
                println!("Coinbase amount: {:#?}", tickerData);
                tickerData.amount
            }
            Err(err) => {
                eprintln!("Error retrieving Coinbase amount: {}", err);
                String::from("Failed to fetch ticker price")
            }
        }
    }
}
