CREATE TABLE IF NOT EXISTS providers (
  id INT PRIMARY KEY,
  name VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS currencies (
  id INT PRIMARY KEY,
  name VARCHAR(255),
  symbol VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS tickers (
  id INT PRIMARY KEY,
  provider_id INT NOT NULL,
  base_id INT NOT NULL,
  quote_id INT NOT NULL,

  FOREIGN KEY (provider_id) REFERENCES provider (id),
  FOREIGN KEY (base_id) REFERENCES currency (id),
  FOREIGN KEY (quote_id) REFERENCES currency (id)
);

CREATE INDEX IF NOT EXISTS ticker_base_id_idx ON tickers(base_id);
CREATE INDEX IF NOT EXISTS ticker_quote_id_idx ON tickers(quote_id);
CREATE INDEX IF NOT EXISTS ticker_provider_id_idx ON tickers(provider_id);

CREATE TABLE IF NOT EXISTS ticker_prices (
  id INT PRIMARY KEY,
  ticker_id INT NOT NULL,
  precision INT NOT NULL,
  scale INT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  FOREIGN KEY (ticker_id) REFERENCES Ticker(id)
);

CREATE INDEX IF NOT EXISTS ticker_price_ticker_id_idx ON ticker_prices(ticker_id);
