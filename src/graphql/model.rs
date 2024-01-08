use async_graphql::{Context, EmptySubscription, Object, Schema, SimpleObject};
use log::{info, warn};

use crate::redis_connection;
use crate::services::coinbase::fetch_coinbase_price;

pub type ServiceSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
pub struct QueryRoot;
pub struct MutationRoot;

#[derive(Debug, SimpleObject)]
struct Provider {
    id: Option<i64>,
    name: Option<String>,
}

#[Object]
impl QueryRoot {
    async fn hello(&self, _ctx: &Context<'_>) -> &'static str {
        "Hello world"
    }

    async fn providers(&self, _ctx: &Context<'_>) -> Option<Vec<Provider>> {
        let state = _ctx.data::<crate::AppState>().unwrap();

        let row: Vec<Provider> = sqlx::query_as!(Provider, "SELECT * FROM providers")
            .fetch_all(&state.db_connection)
            .await
            .unwrap();

        Some(row)
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
                info!("Cache hit: {}", value);
                return value;
            }
            Err(err) => {
                warn!("Cache hit missed key: {}", redis_key);
                warn!("Error: {}", err);
                match fetch_coinbase_price(base.as_str(), quote.as_str()).await {
                    Ok(ticker_data) => ticker_data.amount,
                    Err(err) => {
                        warn!("Error retrieving Coinbase amount: {}", err);
                        // TODO: should return a GraphQL error
                        String::from("Failed to fetch ticker price")
                    }
                }
            }
        }
    }
}
