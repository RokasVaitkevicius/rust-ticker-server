use crate::routes::{health, root};
use crate::model::QueryRoot;
use axum::{routing::get, Router, Server, Extension};
use routes::{graphql_playground, graphql_handler};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use std::net::SocketAddr;

mod routes;
mod model;

#[tokio::main]
async fn main() {
    let port = "8080";
    let addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Invalid address format");

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
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
