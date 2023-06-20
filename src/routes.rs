use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Serialize};

#[derive(Serialize)]
struct Health {
    healthy: bool,
}

pub(crate) async fn root() -> impl IntoResponse {
    (StatusCode::OK, Json("Hello world"))
}

pub(crate) async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(Health { healthy: true }))
}
