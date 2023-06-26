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

Code to subscribe to local WS

```js
const ws = new WebSocket("ws://127.0.0.1:8080/ws")

ws.onopen = (event) => {
  console.log("sending echo")
  ws.send(
    JSON.stringify({
      event: "echo",
      data: {
        message: "hello",
      },
    })
  )
}

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data)
  console.log("received msg", msg)
}
```
