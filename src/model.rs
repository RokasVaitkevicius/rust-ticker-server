use async_graphql::{Context, EmptySubscription, Object, Schema};
use redis::Client;

use crate::coinbase::fetch_coinbase_price;
use crate::redis_connection;

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
        let redis_key = format!("{}-{}", base, quote);

        let redis_value: String = redis::cmd("GET")
            .arg(redis_key)
            .query(&mut *redis_connection::get_redis_connection())
            .unwrap_or_else(|_| String::from("Failed to fetch ticker price"));

        println!("Redis value: {}", redis_value);

        match fetch_coinbase_price(base.as_str(), quote.as_str()).await {
            Ok(ticker_data) => ticker_data.amount,
            Err(err) => {
                eprintln!("Error retrieving Coinbase amount: {}", err);
                String::from("Failed to fetch ticker price")
            }
        }
    }
}
