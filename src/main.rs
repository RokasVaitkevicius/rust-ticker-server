use crate::routes::{health, root};
use axum::{routing::get, Router, Server};
mod routes;

#[tokio::main]
async fn main() {
    let port = "0.0.0.0:8000";
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health));

    println!("Server is running on port {}", port);

    Server::bind(&port.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
