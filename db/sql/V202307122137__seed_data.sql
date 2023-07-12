-- Insert providers
INSERT INTO providers (id, name) VALUES (1, 'Coinbase');

-- Insert currencies
INSERT INTO currencies (id, name, symbol) VALUES (1, 'BTC', 'USD');
INSERT INTO currencies (id, name, symbol) VALUES (2, 'ETH', 'USD');

-- Insert tickers
INSERT INTO tickers (id, provider_id, base_id, quote_id) VALUES (1, 1, 1, 2);
INSERT INTO tickers (id, provider_id, base_id, quote_id) VALUES (2, 1, 2, 2);
