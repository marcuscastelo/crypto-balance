FROM rust:1.75 as builder

# Install CMake and other build dependencies for rdkafka
RUN apt-get update && apt-get install -y \
    cmake \
    libssl-dev \
    libsasl2-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy workspace files
COPY Cargo.toml ./
COPY crates/core ./crates/core
COPY crates/kafka ./crates/kafka

# Build the application
RUN cargo build --release --bin crypto-balance-kafka

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsasl2-2 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/crypto-balance-kafka /usr/local/bin/crypto-balance-kafka

# Copy config files if needed
COPY Config.toml ./
COPY priv_key.json ./

ENTRYPOINT ["crypto-balance-kafka"]