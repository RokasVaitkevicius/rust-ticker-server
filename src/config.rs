use config::Environment;
use eyre::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_server_port")]
    pub server_port: u16,
    #[serde(default = "default_binance_ws_url")]
    pub binance_ws_url: String,
    #[serde(default = "default_coinbase_ws_url")]
    pub coinbase_ws_url: String,
    #[serde(default = "default_rust_log")]
    pub rust_log: String,
    pub database_url: String,
    pub redis_url: String,
}

impl Config {
    pub fn new() -> Result<Self> {
        let config: Config = config::Config::builder()
            .add_source(Environment::default())
            .build()?
            .try_deserialize()?;

        Ok(config)
    }
}

fn default_server_port() -> u16 {
    8080
}
fn default_binance_ws_url() -> String {
    "wss://stream.binance.com:9443/ws".to_string()
}
fn default_coinbase_ws_url() -> String {
    "wss://ws-feed.exchange.coinbase.com".to_string()
}
fn default_rust_log() -> String {
    "debug".to_string()
}
