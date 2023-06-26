use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
};
use std::time::Duration;
use tokio::time;

use axum::extract::ws::Message;
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};

pub async fn websocket_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    let (sender, receiver) = socket.split();

    tokio::spawn(write(sender));
    tokio::spawn(read(receiver));
}

async fn read(_receiver: SplitStream<WebSocket>) {
    // ...
}

async fn write(mut sender: SplitSink<WebSocket, Message>) {
    let interval = Duration::from_secs(5); // Set the interval to 3 seconds

    loop {
        println!("Sending a message...");
        sender
            .send(Message::Text(r#"{ "message": "Hello world" }"#.to_string()))
            .await
            .unwrap();

        time::sleep(interval).await;
    }
}
