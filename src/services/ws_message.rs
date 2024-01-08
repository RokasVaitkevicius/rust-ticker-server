use log::warn;
use serde::Serialize;
use std::fmt;

use crate::services::binance::BinanceMessage;
use crate::services::coinbase::CoinbaseMessage;

#[derive(Serialize)]
pub struct WsMessage {
    source: String,
    base: String,
    quote: String,
    pub price: String,
}

impl WsMessage {
    pub fn get_key(&self) -> String {
        format!("{}-{}", self.source, self.get_symbol())
    }

    pub fn get_symbol(&self) -> String {
        format!("{}-{}", self.base, self.quote)
    }
}

impl fmt::Display for WsMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(
            f,
            "(Ticker: {} - {} - {})",
            self.source,
            self.get_symbol(),
            self.price
        )
    }
}

impl From<BinanceMessage> for WsMessage {
    fn from(msg: BinanceMessage) -> Self {
        let symbol = get_symbol(msg.s.as_str());

        let (base, quote) = match symbol.split_once('-') {
            Some((base, quote)) => (base.to_string(), quote.to_string()),
            None => ("".to_string(), "".to_string()),
        };

        WsMessage {
            source: "binance".to_string(),
            price: msg.c,
            base,
            quote,
        }
    }
}

impl From<CoinbaseMessage> for WsMessage {
    fn from(msg: CoinbaseMessage) -> Self {
        let (base, quote) = match msg.product_id.split_once('-') {
            Some((base, quote)) => (base.to_string(), quote.to_string()),
            None => ("".to_string(), "".to_string()),
        };

        WsMessage {
            source: "coinbase".to_string(),
            price: msg.price,
            base,
            quote,
        }
    }
}

fn get_symbol(symbol: &str) -> String {
    let known_quote_currencies = [
        "USDT", "BTC", "ETH", "BNB", "DAI", "USD", "EUR", "USDC", "TRY", "BRL", "ZAR", "ARS",
        "RON", "XRP", "UAH", "BIDR", "NGN", "PLN", "RUB", "DOGE", "IDRT",
    ];
    for &quote in known_quote_currencies.iter() {
        if symbol.ends_with(quote) {
            if let Some(base) = symbol.strip_suffix(quote) {
                return format!("{}-{}", base, quote);
            }
        }
    }
    warn!("Unknown symbol: {}", symbol);
    "".to_string()
}
