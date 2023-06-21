use crate::routes::{health, root};
use crate::model::QueryRoot;
use axum::{routing::get, Router, Server, Extension};
use routes::{graphql_playground, graphql_handler};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};

mod routes;
mod model;

#[tokio::main]
async fn main() {
    let port = "0.0.0.0:8080";
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema));

    println!("Server is running on port {}", port);

    Server::bind(&port.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
