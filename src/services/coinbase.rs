use futures_util::{SinkExt, StreamExt};
use redis::{Client as RedisClient, Commands, ExistenceCheck, SetExpiry, SetOptions, Value};
use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use std::{error::Error, fmt, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::services::ws_message::WsMessage;

#[derive(Deserialize)]
struct CoinbaseResponse {
    data: TickerData,
}

#[derive(Deserialize, Debug)]
pub struct TickerData {
    pub amount: String,
    pub base: String,
    pub currency: String,
}

#[derive(Deserialize, Debug)]
pub struct CoinbaseMessage {
    pub product_id: String,
    pub price: String,
}

impl fmt::Display for CoinbaseMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "(Ticker: {} - {})", self.product_id, self.price)
    }
}

pub async fn fetch_coinbase_price(base: &str, quote: &str) -> Result<TickerData, Box<dyn Error>> {
    let url = format!("https://api.coinbase.com/v2/prices/{}-{}/buy", base, quote);

    let client = ReqwestClient::new();
    let response = client.get(&url).send().await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let ticker_data: CoinbaseResponse = serde_json::from_str(&body)?;

        Ok(ticker_data.data)
    } else if let Ok(error_body) = response.text().await {
        Err(error_body.into())
    } else {
        Err("Unknown error".into())
    }
}

pub async fn subscribe_coinbase_ticker(
    ws_tx: &Arc<tokio::sync::Mutex<UnboundedSender<tungstenite::Message>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://ws-feed.exchange.coinbase.com";
    let (mut ws_stream, _) = connect_async(url).await?;

    println!("Connected to Coinbase WebSocket");

    let subscribe_msg = r#"{
        "type": "subscribe",
        "channels": [{ "name": "ticker", "product_ids": ["BTC-USDT"] }, { "name": "ticker", "product_ids": ["ETH-USDT"] }]
    }"#;

    ws_stream.send(Message::Text(subscribe_msg.into())).await?;
    println!("Subscribed to ticker channel");

    // let mut con = redis_connection::get_redis_connection();
    // TODO: figure out a way how to reuse connection
    let client = RedisClient::open("redis://127.0.0.1:6379/").unwrap();
    let mut connection = client.get_connection().unwrap();

    while let Some(Ok(message)) = ws_stream.next().await {
        match message {
            Message::Text(data) => {
                // println!("Received message: {}", data);

                let v = serde_json::from_str::<serde_json::Value>(&data).unwrap();

                // We only care about ticker messages
                if v["type"] == "ticker" {
                    let coinbase_message: CoinbaseMessage = serde_json::from_str(&data)?;
                    let ws_message: WsMessage = coinbase_message.into();

                    // println!("{}", coinbase_message);

                    let redis_result: Result<Value, redis::RedisError> = connection.set_options(
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
