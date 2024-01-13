use async_graphql::{EmptySubscription, Schema};
use axum::{routing::get, Extension, Router, Server};
use dotenv::dotenv;
use futures_util::TryFutureExt;
use log::{error, warn};
use services::binance;
use sqlx::{self, sqlite::SqlitePoolOptions, SqlitePool};
use std::net::SocketAddr;
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
    pub db_connection: SqlitePool,
    pub settings: Settings,
    pub ticker_tx: broadcast::Sender<Message>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let settings = Settings::new().expect("Failed to load configuration");

    let pool: sqlx::Pool<sqlx::Sqlite> = SqlitePoolOptions::new()
        .connect(&settings.database_url)
        .await
        .unwrap();

    let (ticker_tx, _rx) = broadcast::channel::<Message>(100);

    let app_context = AppContext {
        db_connection: pool,
        settings,
        ticker_tx,
    };

    let addr: SocketAddr = format!("0.0.0.0:{}", app_context.settings.server_port)
        .parse()
        .expect("Invalid address format");

    let gql_schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(app_context.clone())
        .finish();

    let app = Router::new()
        .route("/", get(root))
        .route("/ws", get(websocket_handler))
        .route("/health", get(health))
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .with_state(app_context.clone())
        .layer(Extension(gql_schema));

    // Spinning up a separate task to subscribe to Coinbase ticker
    let app_context_cl = app_context.clone();
    tokio::task::spawn(async move {
        subscribe_coinbase_ticker(app_context_cl)
            .unwrap_or_else(|err| warn!("Connecting to socket failed: {}", err))
            .await
    });

    let chunked_streams = binance::get_chunked_ws_streams().await.unwrap();

    for stream in chunked_streams {
        let app_context_cl = app_context.clone();
        tokio::task::spawn(async move {
            subscribe_binance_ticker(app_context_cl, &stream)
                .unwrap_or_else(|err| warn!("Connecting to socket failed: {}", err))
                .await
        });
    }

    if let Err(e) = Server::bind(&addr).serve(app.into_make_service()).await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}
