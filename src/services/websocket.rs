use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};

use crate::AppState;

pub async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (sender, receiver) = socket.split();

    tokio::spawn(write(sender, state));
    tokio::spawn(read(receiver));
}

async fn read(_receiver: SplitStream<WebSocket>) {
    // ...
}

async fn write(mut sender: SplitSink<WebSocket, Message>, state: AppState) {
    let mut rx = state.tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        sender
            .send(Message::Text(msg.to_string()))
            .await
            .expect("Error while sending message");
    }

    sender.close().await.unwrap();
}
