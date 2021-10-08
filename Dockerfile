FROM rust
COPY . . 
ENTRYPOINT ["./target/release/my-sb-persistence"]