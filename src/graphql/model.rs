use async_graphql::{Context, EmptySubscription, Object, Schema};

use crate::services::coinbase::fetch_coinbase_price;
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
        // TODO: check how to use directives to uppercase base and quote
        let redis_key = format!("{}-{}", base.to_uppercase(), quote.to_uppercase());

        let redis_value = redis::cmd("GET")
            .arg(&redis_key)
            .query(&mut *redis_connection::get_redis_connection());

        match redis_value {
            Ok(value) => {
                println!("Cache hit: {}", value);
                return value;
            }
            Err(err) => {
                eprintln!("Cache hit missed key: {}", redis_key);
                eprintln!("Error: {}", err);
                match fetch_coinbase_price(base.as_str(), quote.as_str()).await {
                    Ok(ticker_data) => ticker_data.amount,
                    Err(err) => {
                        eprintln!("Error retrieving Coinbase amount: {}", err);
                        // TODO: should return a GraphQL error
                        String::from("Failed to fetch ticker price")
                    }
                }
            }
        }
    }
}
