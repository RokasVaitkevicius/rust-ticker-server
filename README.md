# rust-ticker-server

Rust server, which takes ticker prices from coinbase and returns them through graphql API.

Also working on exposing websocket which subscribes to coinbase websocket server and will save prices in redis.

To run rust server with hot-reload:

```
cargo watch -x 'run' --watch src
```

Linting and formatting:

```
cargo fmt --all
cargo clippy --all --tests
```
