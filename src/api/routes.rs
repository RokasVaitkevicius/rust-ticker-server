use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use serde::Serialize;

use crate::graphql::ServiceSchema;

#[derive(Serialize)]
struct Health {
    healthy: bool,
}

pub async fn root() -> impl IntoResponse {
    (StatusCode::OK, Json("Hello world"))
}

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(Health { healthy: true }))
}

pub async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/ws"),
    ))
}

pub async fn graphql_handler(
    schema: Extension<ServiceSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
