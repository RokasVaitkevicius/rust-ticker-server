use eyre::{Result, WrapErr};
use futures_util::StreamExt;
use log::{info, warn};
use redis::{Client as RedisClient, Commands, ExistenceCheck, SetExpiry, SetOptions, Value};
use serde::Deserialize;
use std::{env, fmt};
use tokio::{sync::broadcast::Sender, time::sleep, time::Duration};
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

pub async fn subscribe_binance_ticker(ws_tx: Sender<Message>) -> Result<()> {
    let url = "wss://stream.binance.com:9443/ws/btcusdt@ticker/ethusdt@ticker";

    let mut redis_connection = RedisClient::open(env::var("REDIS_URL").unwrap().as_str())
        .unwrap()
        .get_connection()
        .unwrap();

    loop {
        match connect_async(url).await {
            Ok((mut ws_stream, _)) => {
                info!("Connected to Binance WebSocket");

                while let Some(Ok(message)) = ws_stream.next().await {
                    match message {
                        Message::Text(data) => {
                            // info!("Received message: {}", data);

                            let binance_message: BinanceMessage = serde_json::from_str(&data)?;
                            let ws_message: WsMessage = binance_message.into();

                            let redis_result: Result<Value, redis::RedisError> = redis_connection
                                .set_options(
                                    ws_message.get_key(),
                                    &ws_message.price,
                                    SetOptions::default()
                                        .conditional_set(ExistenceCheck::NX)
                                        .get(true)
                                        .with_expiration(SetExpiry::EX(20)),
                                );

                            match redis_result {
                                Ok(value) => {
                                    // Only send value, when it's not a cache hit
                                    if value == Value::Nil {
                                        let ws_message_string =
                                            serde_json::to_string(&ws_message).unwrap();

                                        info!("Sending value to the ws client {}", ws_message);
                                        ws_tx
                                            .send(Message::Text(ws_message_string))
                                            .map_err(eyre::Report::from)
                                            .wrap_err("Failed to send WebSocket Binance message")?;
                                    }
                                }
                                Err(err) => {
                                    warn!("Error setting cache: {}", err);
                                }
                            }
                        }
                        Message::Close(_) => {
                            warn!("WebSocket connection closed");
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => {
                warn!(
                    "Failed to connect to WebSocket: {}. Retrying in 5 seconds...",
                    err
                );
            }
        }

        // Wait 5 seconds trying to reconnect
        sleep(Duration::from_secs(5)).await;
    }
}
