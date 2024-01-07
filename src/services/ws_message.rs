use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

use crate::services::binance::BinanceMessage;
use crate::services::coinbase::CoinbaseMessage;

#[derive(Serialize)]
pub struct WsMessage {
    source: String,
    symbol: String,
    pub price: String,
}

impl WsMessage {
    pub fn get_key(&self) -> String {
        format!("{}-{}-{}", self.source, self.symbol, self.price)
    }
}

impl fmt::Display for WsMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(
            f,
            "(Ticker: {} - {} - {})",
            self.source, self.symbol, self.price
        )
    }
}

impl From<BinanceMessage> for WsMessage {
    fn from(msg: BinanceMessage) -> Self {
        let symbol_mapping = get_symbol_mapping();
        let symbol = symbol_mapping.get(&msg.s).unwrap_or(&msg.s).clone();

        WsMessage {
            source: "binance".to_string(),
            symbol,
            price: msg.c,
        }
    }
}

impl From<CoinbaseMessage> for WsMessage {
    fn from(msg: CoinbaseMessage) -> Self {
        WsMessage {
            source: "coinbase".to_string(),
            symbol: msg.product_id,
            price: msg.price,
        }
    }
}

fn get_symbol_mapping() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("BTCUSDT".to_string(), "BTC-USDT".to_string());
    map.insert("ETHUSDT".to_string(), "ETH-USDT".to_string());
    return map;
}
