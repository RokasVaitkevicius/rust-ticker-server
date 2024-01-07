use futures_util::StreamExt;
use redis::{Client as RedisClient, Value};
use serde::Deserialize;
use std::{fmt, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::services::ws_message::WsMessage;

#[derive(Deserialize, Debug)]
pub struct BinanceMessage {
    pub s: String, // Symbol
    pub c: String, // Price
}

impl fmt::Display for BinanceMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "(Ticker: {} - {})", self.s, self.c)
    }
}

pub async fn subscribe_binance_ticker(
    ws_tx: &Arc<tokio::sync::Mutex<UnboundedSender<tungstenite::Message>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://stream.binance.com:9443/ws/btcusdt@ticker/ethusdt@ticker";

    let (mut ws_stream, _) = connect_async(url).await?;

    println!("Connected to Binance WebSocket");

    // let mut con = redis_connection::get_redis_connection();
    // TODO: figure out a way how to reuse connection
    let client = RedisClient::open("redis://127.0.0.1:6379/").unwrap();
    let mut connection = client.get_connection().unwrap();

    while let Some(Ok(message)) = ws_stream.next().await {
        match message {
            Message::Text(data) => {
                // println!("Received message: {}", data);

                let binance_message: BinanceMessage = serde_json::from_str(&data)?;
                let ws_message: WsMessage = binance_message.into();

                let redis_result: Result<Value, redis::RedisError> = redis::cmd("SET")
                    .arg(ws_message.get_key())
                    .arg(&ws_message.price)
                    .arg("NX")
                    .arg("GET")
                    .arg("EX")
                    .arg(20)
                    .query(&mut connection);

                match redis_result {
                    Ok(value) => {
                        // Only send value, when it's not a cache hit
                        if value == Value::Nil {
                            let ws_message_string = serde_json::to_string(&ws_message).unwrap();

                            println!("Sending value to the ws client {}", ws_message);
                            ws_tx
                                .lock()
                                .await
                                .send(Message::Text(ws_message_string))
                                .unwrap();
                        }
                    }
                    Err(err) => {
                        eprintln!("Error setting cache: {}", err);
                    }
                }
            }
            Message::Close(_) => {
                println!("WebSocket connection closed");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
