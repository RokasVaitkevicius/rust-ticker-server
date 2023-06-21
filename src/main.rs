use axum::{routing::get, Router, Server, Extension};
use routes::{graphql_playground, graphql_handler};
use async_graphql::{EmptySubscription, Schema};
use std::net::SocketAddr;
use dotenv::dotenv;
use std::env;

use crate::routes::{health, root};
use crate::model::{QueryRoot, MutationRoot};

mod routes;
mod model;
mod coinbase;

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

    if let Err(e) = Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
