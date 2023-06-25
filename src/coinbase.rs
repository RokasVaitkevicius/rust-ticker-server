use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::error::Error;
use std::fmt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::redis_connection;

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
    // type: String,
    sequence: u64,
    product_id: String,
    price: String,
    open_24h: String,
    volume_24h: String,
    low_24h: String,
    high_24h: String,
    volume_30d: String,
    best_bid: String,
    best_bid_size: String,
    best_ask: String,
    best_ask_size: String,
    side: String,
    time: String,
    trade_id: u64,
    last_size: String,
}

impl fmt::Display for CoinbaseMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "(Ticker: {} - {})", self.product_id, self.price)
    }
}

pub async fn fetch_coinbase_price(base: &str, quote: &str) -> Result<TickerData, Box<dyn Error>> {
    let url = format!("https://api.coinbase.com/v2/prices/{}-{}/buy", base, quote);

    let client = reqwest::Client::new();
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

pub async fn subscribe_coinbase_ticker() -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://ws-feed.exchange.coinbase.com";
    let (mut ws_stream, _) = connect_async(url).await?;

    println!("Connected to WebSocket");

    let subscribe_msg = r#"{
        "type": "subscribe",
        "channels": [{ "name": "ticker", "product_ids": ["BTC-USD"] }]
    }"#;

    ws_stream.send(Message::Text(subscribe_msg.into())).await?;
    println!("Subscribed to ticker channel");

    let mut con = redis_connection::get_redis_connection();

    while let Some(Ok(message)) = ws_stream.next().await {
        match message {
            Message::Text(data) => {
                // println!("Received message: {}", data);

                let v = serde_json::from_str::<serde_json::Value>(&data).unwrap();

                // We only care about ticker messages
                if v["type"] == "ticker" {
                    let coinbase_message: CoinbaseMessage = serde_json::from_str(&data)?;

                    println!("{}", coinbase_message);

                    let key = coinbase_message.product_id.to_string();
                    let _: () = redis::cmd("SET")
                        .arg(key)
                        .arg(coinbase_message.price)
                        .arg("NX")
                        .arg("EX")
                        .arg(20)
                        .query(&mut con)
                        .unwrap();
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
