# Builder stage
FROM rust:1.91 as builder

WORKDIR /usr/src/app
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install OpenSSL - often required for Rust apps making HTTPS requests or using certain DB drivers
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/caxur .

CMD ["./caxur"]
