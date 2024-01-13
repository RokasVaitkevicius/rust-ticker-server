use eyre::{bail, Result};
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use redis::{Client as RedisClient, Commands, ExistenceCheck, SetExpiry, SetOptions, Value};
use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use std::fmt;
use tokio::{time::sleep, time::Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::{services::ws_message::WsMessage, AppContext};

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

pub async fn fetch_coinbase_price(base: &str, quote: &str) -> Result<TickerData> {
    let url = format!("https://api.coinbase.com/v2/prices/{}-{}/buy", base, quote);

    let client = ReqwestClient::new();
    let response = client.get(url).send().await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let ticker_data: CoinbaseResponse = serde_json::from_str(&body)?;

        Ok(ticker_data.data)
    } else if let Ok(error_body) = response.text().await {
        bail!(error_body)
    } else {
        bail!("Unknown error")
    }
}

pub async fn subscribe_coinbase_ticker(app_context: AppContext) -> Result<()> {
    let mut redis_connection =
        RedisClient::open(app_context.settings.redis_url)?.get_connection()?;

    loop {
        match connect_async(app_context.settings.coinbase_ws_url.as_str()).await {
            Ok((mut ws_stream, _)) => {
                info!("Connected to Coinbase WebSocket");

                let subscribe_msg = r#"{
                  "type": "subscribe",
                  "channels": [{ "name": "ticker", "product_ids": ["BTC-USDT"] }, { "name": "ticker", "product_ids": ["ETH-USDT"] }]
                }"#;

                ws_stream.send(Message::Text(subscribe_msg.into())).await?;
                info!("Subscribed to ticker channel");

                while let Some(Ok(message)) = ws_stream.next().await {
                    match message {
                        Message::Text(data) => {
                            // info!("Received message: {}", data);

                            let v = serde_json::from_str::<serde_json::Value>(&data)?;

                            // We only care about ticker messages
                            if v["type"] == "ticker" {
                                let coinbase_message: CoinbaseMessage =
                                    serde_json::from_str(&data)?;
                                let ws_message: WsMessage = coinbase_message.into();

                                // info!("{}", coinbase_message);

                                let redis_result: Result<Value, redis::RedisError> =
                                    redis_connection.set_options(
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
                                                serde_json::to_string(&ws_message)?;

                                            info!("Sending value to the ws client {}", ws_message);
                                            app_context
                                                .ticker_tx
                                                .send(Message::Text(ws_message_string))?;
                                        }
                                    }
                                    Err(err) => {
                                        warn!("Error setting cache: {}", err);
                                    }
                                }
                            }
                        }
                        Message::Close(_) => {
                            info!("WebSocket connection closed");
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
