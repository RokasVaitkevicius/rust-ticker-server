use async_graphql::{EmptySubscription, Schema};
use axum::{routing::get, Extension, Router, Server};
use dotenv::dotenv;
use futures_util::TryFutureExt;
use log::{error, warn};
use services::binance;
use sqlx::{self, sqlite::SqlitePoolOptions, SqlitePool};
use std::{env, net::SocketAddr};
use tokio::sync::broadcast;
use tungstenite::Message;

use crate::api::routes::{graphql_handler, graphql_playground, health, root};
use crate::config::settings::Settings;
use crate::graphql::{MutationRoot, QueryRoot};
use crate::services::{
    binance::subscribe_binance_ticker, coinbase::subscribe_coinbase_ticker, redis_connection,
    websocket::websocket_handler,
};

mod api;
mod config;
mod graphql;
mod services;

#[derive(Clone)]
pub struct AppContext {
    pub tx: broadcast::Sender<Message>,
    pub db_connection: SqlitePool,
    pub settings: Settings,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let settings = Settings::new().expect("Failed to load configuration");

    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Invalid address format");

    let pool: sqlx::Pool<sqlx::Sqlite> = SqlitePoolOptions::new()
        .connect(env::var("DATABASE_URL").unwrap().as_str())
        .await
        .unwrap();

    let (tx, _rx) = broadcast::channel::<Message>(100);

    let app_state = AppContext {
        db_connection: pool,
        tx,
        settings,
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
            .unwrap_or_else(|err| warn!("Connecting to socket failed: {}", err))
            .await
    });

    let chunked_streams = binance::get_chunked_ws_streams().await.unwrap();

    for stream in chunked_streams {
        let binance_tx = app_state.tx.clone();
        tokio::task::spawn(async move {
            subscribe_binance_ticker(binance_tx, &stream)
                .unwrap_or_else(|err| warn!("Connecting to socket failed: {}", err))
                .await
        });
    }

    if let Err(e) = Server::bind(&addr).serve(app.into_make_service()).await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}
