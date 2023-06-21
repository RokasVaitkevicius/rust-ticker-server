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

pub async fn fetch_coinbase_price() -> Result<TickerData, Box<dyn Error>> {
    let url = "https://api.coinbase.com/v2/prices/BTC-USD/buy";
    let response = reqwest::get(url).await?.text().await?;
    let coinbase_response: CoinbaseResponse = serde_json::from_str(&response)?;
    println!("Coinbase response: {:#?}", response);

    Ok(coinbase_response.data)
}
