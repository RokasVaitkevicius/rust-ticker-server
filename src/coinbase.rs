use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::error::Error;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

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

    while let Some(Ok(message)) = ws_stream.next().await {
        match message {
            Message::Text(data) => {
                println!("Received message: {}", data);

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(price) = json.get("price") {
                        if let Some(ticker_price) = price.as_str() {
                            println!("Ticker price: {}", ticker_price);
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
