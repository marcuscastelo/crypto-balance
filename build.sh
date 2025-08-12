#!/bin/bash

echo "🏗️  Building crypto-balance workspace..."

# Build all crates
echo "Building core library..."
cargo build --package crypto-balance-core --release

echo "Building CLI application..."
cargo build --package crypto-balance-cli --release

echo "Building Kafka consumer..."
cargo build --package crypto-balance-kafka --release

echo "✅ Build completed!"
echo ""
echo "📦 Binaries created:"
echo "  CLI: target/release/crypto-balance-cli"
echo "  Kafka: target/release/crypto-balance-kafka"
echo ""
echo "🚀 Usage:"
echo "  # Run CLI"
echo "  ./target/release/crypto-balance-cli"
echo ""
echo "  # Run Kafka consumer"
echo "  export KAFKA_BROKERS=localhost:9092"
echo "  export KAFKA_GROUP_ID=crypto-balance-group"
echo "  export KAFKA_TOPICS=crypto-balance-events"
echo "  ./target/release/crypto-balance-kafka"