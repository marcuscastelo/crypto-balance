# Crypto Balance - Hexagonal Architecture with Multi-Crate Workspace

🏗️ **Arquitetura Hexagonal** + 📦 **Cargo Workspace** + 🚀 **Múltiplos Entry Points**

Esta aplicação de balanço de criptomoedas foi completamente refatorada seguindo **Arquitetura Hexagonal (Ports & Adapters)** usando **Cargo Workspace** com múltiplos crates, permitindo execução tanto como **CLI tool** quanto como **microsserviço Kafka**.

## 📋 Quick Start

```bash
# 1. Build all crates
./build.sh

# 2. Run as CLI tool (tradicional)
./target/release/crypto-balance-cli

# 3. Run as Kafka consumer (microservice)
export KAFKA_BROKERS=localhost:9092
./target/release/crypto-balance-kafka

# 4. Run with Docker Compose + Kafka
docker-compose -f docker-compose.kafka.yml up
```

## 🏗️ Workspace Structure

```
crypto-balance/
├── crates/
│   ├── core/              # 📚 Shared library (business logic)
│   ├── cli/               # 💻 CLI application 
│   └── kafka/             # 🔄 Kafka consumer application
├── Cargo.toml             # 🎯 Workspace root
├── build.sh               # 🛠️ Build script
└── docker-compose.kafka.yml
```

## 🎯 Benefícios da Arquitetura

### ✅ **Hexagonal (Ports & Adapters)**
- **Testável**: Mock de interfaces facilita testes
- **Flexível**: Múltiplos entry points (CLI + Kafka + futuro HTTP)
- **Manutenível**: Separação clara de responsabilidades
- **Extensível**: Novos adapters sem modificar core

### ✅ **Multi-Crate Workspace**  
- **Reutilização**: Core business logic compartilhada
- **Build Otimizado**: Compile apenas o que mudou
- **Deploy Flexível**: Apps independentes
- **Desenvolvimento Paralelo**: Equipes podem trabalhar em apps diferentes

## 🚀 Execution Modes

| Mode | Use Case | Command |
|------|----------|---------|
| **CLI** | Automação, scripts, execução manual | `cargo run -p crypto-balance-cli` |
| **Kafka** | Microsserviços, event-driven | `cargo run -p crypto-balance-kafka` |
| **Docker** | Production deployment | `docker-compose up` |

## 📡 Event-Driven Architecture (Kafka)

Suporte para eventos Kafka:

```json
{
  "RunBalanceUpdate": {
    "exchange": "Binance",
    "timestamp": "2024-01-01T12:00:00Z"
  }
}
```

## 📚 Documentation

- **[HEXAGONAL.md](HEXAGONAL.md)** - Detalhes da arquitetura hexagonal
- **[WORKSPACE.md](WORKSPACE.md)** - Guia do workspace multi-crate

## 🛠️ Development

```bash
# Test core library
cargo test -p crypto-balance-core

# Run CLI in dev mode  
cargo run -p crypto-balance-cli -- health

# Run Kafka consumer (needs Kafka)
KAFKA_BROKERS=localhost:9092 cargo run -p crypto-balance-kafka

# Build workspace
cargo build --workspace --release
```

## 🐳 Docker Support

Individual Dockerfiles for each app:
- `crates/cli/Dockerfile` - CLI application
- `crates/kafka/Dockerfile` - Kafka consumer
- `docker-compose.kafka.yml` - Full stack with Kafka

---

# Configuration

1. Create Config.toml in project root folder
2. Fill with API keys and wallet addresses
```toml
[blockchain]
etherscan_api_key = "<REPLACE>"
scrollscan_api_key = "<REPLACE>"
lineascan_api_key = "<REPLACE>"
basescan_api_key = "<REPLACE>"
arbiscan_api_key = "<REPLACE>"
optimistic_etherscan_api_key = "<REPLACE>"
polygonscan_api_key = "<REPLACE>"

[blockchain.hold.evm]
address = "<REPLACE>"

[blockchain.hold_sc.evm]
address = "<REPLACE>"

[blockchain.airdrops.evm]
address = "<REPLACE>"

[blockchain.airdrops.solana]
address = "<REPLACE>"

[blockchain.airdrops.cosmos]
cosmos_address = "<REPLACE>"
osmosis_address = "<REPLACE>"
celestia_address = "<REPLACE>"
injective_address = "<REPLACE>"

[binance]
api_key = "<REPLACE>"
secret_key = "<REPLACE>"

[bybit]
api_key = "<REPLACE>"
secret_key = "<REPLACE>"

[kraken]
api_key = "<REPLACE>"
secret_key = "<REPLACE>"

[sheets]
priv_key = "<REPLACE>"
spreadsheet_id = "<REPLACE>"

[coingecko]
api_key = "<REPLACE>" 

```
3. Change output sheets and ranges under sheets/ranges.rs (these are Google Sheets' named ranges)
4. Run program
