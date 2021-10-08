FROM rust
COPY . . 
ENTRYPOINT ["./target/release/binance-quote-bridge"]