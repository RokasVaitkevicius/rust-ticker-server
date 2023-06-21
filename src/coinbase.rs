use serde::Deserialize;
use std::error::Error;

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
