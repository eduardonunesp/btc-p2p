# Create the build container to compile the btc_handshake world program 
FROM rust:latest as builder
WORKDIR /btc_handshake
COPY . .
RUN cargo build --example handshake --release --locked

# Create the execution container by copying the compiled btc_handshake world to it and running it
FROM debian:bookworm-slim
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y libssl3 ca-certificates && apt clean && rm -rf /var/lib/apt/lists/*
COPY --from=builder /btc_handshake/target/release/examples/handshake /bin/btc_handshake
EXPOSE 8080
CMD ["/bin/btc_handshake"]