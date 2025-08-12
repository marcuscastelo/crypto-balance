FROM rust:1.75 as builder

WORKDIR /usr/src/app

# Copy workspace files
COPY Cargo.toml ./
COPY crates/core ./crates/core
COPY crates/cli ./crates/cli

# Build the application
RUN cargo build --release --bin crypto-balance-cli

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/crypto-balance-cli /usr/local/bin/crypto-balance-cli

# Copy config files if needed
COPY Config.toml ./
COPY priv_key.json ./

ENTRYPOINT ["crypto-balance-cli"]