use eyre::{bail, Result, WrapErr};
use futures_util::StreamExt;
use log::{info, warn};
use redis::{Client as RedisClient, Commands, ExistenceCheck, SetExpiry, SetOptions, Value};
use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use std::fmt;
use tokio::{time::sleep, time::Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::{services::ws_message::WsMessage, AppContext};

#[derive(Deserialize, Debug)]
pub struct BinanceMessage {
    pub s: String, // Symbol
    pub c: String, // Price
}

#[derive(Deserialize, Debug)]
struct ExchangeInfo {
    symbols: Vec<Symbol>,
}

#[derive(Deserialize, Debug)]
struct Symbol {
    symbol: String,
}

impl fmt::Display for BinanceMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "(Ticker: {} - {})", self.s, self.c)
    }
}

async fn fetch_market_symbols() -> Result<Vec<String>> {
    let url = "https://api.binance.com/api/v3/exchangeInfo";

    let client = ReqwestClient::new();
    let response = client.get(url).send().await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let exchange_info: ExchangeInfo = serde_json::from_str(&body)?;

        let symbols = exchange_info
            .symbols
            .into_iter()
            .map(|s| s.symbol)
            .collect();

        Ok(symbols)
    } else if let Ok(error_body) = response.text().await {
        bail!(error_body)
    } else {
        bail!("Unknown error")
    }
}

pub async fn get_chunked_ws_streams() -> Result<Vec<String>> {
    let symbols = fetch_market_symbols().await?;

    let chunked_streams = symbols
        .chunks(300)
        .map(|chunk| {
            chunk
                .iter()
                .map(|symbol| format!("{}@ticker", symbol.to_lowercase()))
                .collect::<Vec<_>>()
                .join("/")
        })
        .collect::<Vec<_>>();

    Ok(chunked_streams)
}

pub async fn subscribe_binance_ticker(app_context: AppContext, streams: &str) -> Result<()> {
    let url = format!("{}/{}", app_context.settings.binance_ws_url, streams);

    let mut redis_connection =
        RedisClient::open(app_context.settings.redis_url.as_str())?.get_connection()?;

    loop {
        match connect_async(&url).await {
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
                                        let ws_message_string = serde_json::to_string(&ws_message)?;

                                        info!("Sending value to the ws client {}", ws_message);
                                        app_context
                                            .ticker_tx
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
