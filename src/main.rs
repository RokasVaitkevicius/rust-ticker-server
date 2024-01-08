use async_graphql::{EmptySubscription, Schema};
use axum::{routing::get, Extension, Router, Server};
use dotenv::dotenv;
use futures_util::TryFutureExt;
use sqlx::{self, sqlite::SqlitePoolOptions, SqlitePool};
use std::{env, net::SocketAddr};
use tokio::sync::broadcast;
use tungstenite::Message;

use crate::api::routes::{graphql_handler, graphql_playground, health, root};
use crate::graphql::{MutationRoot, QueryRoot};
use crate::services::{
    binance::subscribe_binance_ticker, coinbase::subscribe_coinbase_ticker, redis_connection,
    websocket::websocket_handler,
};

mod api;
mod graphql;
mod services;

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<Message>,
    pub db_connection: SqlitePool,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Invalid address format");

    let pool: sqlx::Pool<sqlx::Sqlite> = SqlitePoolOptions::new()
        .connect(env::var("DATABASE_URL").unwrap().as_str())
        .await
        .unwrap();

    let (tx, rx) = broadcast::channel::<Message>(100);

    let app_state = AppState {
        db_connection: pool,
        tx,
    };

    let gql_schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(app_state.clone())
        .finish();

    let app = Router::new()
        .route("/", get(root))
        .route("/ws", get(websocket_handler))
        .route("/health", get(health))
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .with_state(app_state.clone())
        .layer(Extension(gql_schema));

    // Spinning up a separate task to subscribe to Coinbase ticker
    let coinbase_tx = app_state.tx.clone();
    tokio::task::spawn(async move {
        subscribe_coinbase_ticker(coinbase_tx)
            .unwrap_or_else(|err| println!("Connecting to socket failed: {}", err))
            .await
    });

    // Spinning up a separate task to subscribe to Binance ticker
    let binance_tx = app_state.tx.clone();
    tokio::task::spawn(async move {
        subscribe_binance_ticker(binance_tx)
            .unwrap_or_else(|err| println!("Connecting to socket failed: {}", err))
            .await
    });

    if let Err(e) = Server::bind(&addr).serve(app.into_make_service()).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
