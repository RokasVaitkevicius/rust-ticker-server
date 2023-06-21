FROM alpine:latest as builder

WORKDIR /app

RUN apk add --no-cache \
    build-base \
    cargo \
    rust

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release

COPY src ./src

RUN cargo build --release

FROM alpine:latest

EXPOSE 8080

RUN apk --no-cache add \
    ca-certificates \
    libgcc

COPY --from=builder /app/target/release/rust-ticker-server /rust-ticker-server

ENTRYPOINT ["/rust-ticker-server"]

CMD []
