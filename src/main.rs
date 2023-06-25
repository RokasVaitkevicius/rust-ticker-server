use async_graphql::{EmptySubscription, Schema};
use axum::{routing::get, Extension, Router, Server};
use dotenv::dotenv;
use routes::{graphql_handler, graphql_playground};
use std::env;
use std::net::SocketAddr;

use crate::coinbase::subscribe_coinbase_ticker;
use crate::model::{MutationRoot, QueryRoot};
use crate::routes::{health, root};

mod coinbase;
mod model;
mod redis_connection;
mod routes;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Invalid address format");

    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema));

    println!("Server is running on {}", addr);

    if let Err(e) = subscribe_coinbase_ticker().await {
        eprintln!("Coinbase websocket connection error: {}", e);
    };

    if let Err(e) = Server::bind(&addr).serve(app.into_make_service()).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
