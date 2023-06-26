use async_graphql::{EmptySubscription, Schema};
use axum::{routing::get, Extension, Router, Server};
use dotenv::dotenv;
use futures_util::TryFutureExt;
use routes::{graphql_handler, graphql_playground};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use tungstenite::Message;
use std::sync::Arc;
use std::{env, net::SocketAddr};

use crate::coinbase::subscribe_coinbase_ticker;
use crate::model::{MutationRoot, QueryRoot};
use crate::routes::{health, root};
use crate::websocket::websocket_handler;

mod coinbase;
mod model;
mod redis_connection;
mod routes;
mod websocket;

#[derive(Clone)]
pub struct AppState {
    pub tx: Arc<Mutex<UnboundedSender<Message>>>,
    pub rx: Arc<Mutex<UnboundedReceiver<Message>>>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Invalid address format");

    let gql_schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();

    // tx - transmitter
    // rx - receiver
    let (tx, rx): (UnboundedSender<Message>, UnboundedReceiver<Message>) = mpsc::unbounded_channel();
    let tx = Arc::new(Mutex::new(tx));
    let rx = Arc::new(Mutex::new(rx));

    let app_state = AppState { tx, rx };

    let app = Router::new()
        .route("/", get(root))
        .route("/ws", get(websocket_handler))
        .route("/health", get(health))
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .with_state(app_state.clone())
        .layer(Extension(gql_schema));

    // Spinning up a separate task to subscribe to Coinbase ticker
    tokio::task::spawn(async move {
        subscribe_coinbase_ticker(&app_state.tx)
            .unwrap_or_else(|err| println!("Connecting to socket failed: {}", err))
            .await
    });

    if let Err(e) = Server::bind(&addr).serve(app.into_make_service()).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
