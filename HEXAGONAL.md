# Crypto Balance - Arquitetura Hexagonal

Esta aplicação foi refatorada para usar **Arquitetura Hexagonal (Ports & Adapters)** com **Cargo Workspace**, permitindo flexibilidade para diferentes entrypoints (CLI, Kafka, HTTP futuro) e saídas (APIs, Sheets, banco de dados futuro).

## 🏗️ Estrutura da Arquitetura

```
crypto-balance/
├── crates/
│   ├── core/                   # Shared Library (Hexagon)
│   │   ├── src/
│   │   │   ├── ports/          # Interfaces/Contracts (Hexagon boundary)
│   │   │   │   ├── application_service.rs  # Core business orchestration
│   │   │   │   ├── command_handler.rs      # CLI command handling
│   │   │   │   ├── event_handler.rs        # Event processing (Kafka)
│   │   │   │   ├── balance_repository.rs   # Data persistence
│   │   │   │   ├── exchange_use_cases.rs   # Exchange operations
│   │   │   │   └── routine.rs              # Business routines
│   │   │   ├── domain/         # Domain entities & rules
│   │   │   ├── application/    # Use cases & business services
│   │   │   └── adapters/       # Secondary adapters (driven)
│   │   │       ├── exchange/   # Exchange APIs (Binance, Kraken)
│   │   │       ├── sheets/     # Google Sheets integration
│   │   │       ├── debank/     # Debank API
│   │   │       └── kafka_publisher.rs # Event publishing
│   │   └── Cargo.toml
│   ├── cli/                    # CLI Application (Primary Adapter)
│   │   ├── src/
│   │   │   ├── cli_adapter.rs
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   └── kafka/                  # Kafka Consumer (Primary Adapter)
│       ├── src/
│       │   ├── kafka_adapter.rs
│       │   └── main.rs
│       └── Cargo.toml
└── Cargo.toml                  # Workspace root
```

## 🚀 Modos de Execução

### 1. Modo CLI
Execute routines via linha de comando:

```bash
# Via Cargo
cargo run -p crypto-balance-cli

# Ou diretamente após build
./target/release/crypto-balance-cli

# Com argumentos
./target/release/crypto-balance-cli run --sequential
./target/release/crypto-balance-cli run-routine DebankRoutine
./target/release/crypto-balance-cli list
./target/release/crypto-balance-cli health
```

### 2. Modo Kafka Consumer
Execute como microsserviço consumindo eventos do Kafka:

```bash
# Via Cargo
KAFKA_BROKERS=localhost:9092 cargo run -p crypto-balance-kafka

# Ou diretamente após build
export KAFKA_BROKERS=localhost:9092
export KAFKA_GROUP_ID=crypto-balance-group  
export KAFKA_TOPICS=crypto-balance-events
./target/release/crypto-balance-kafka
```

### 3. Via Docker Compose
Execute a aplicação completa com Kafka:

```bash
# Subir infra (Kafka + observabilidade)
docker-compose -f docker-compose.kafka.yml up -d

# Testar produzindo evento
docker-compose exec kafka kafka-console-producer \
  --bootstrap-server kafka:29092 \
  --topic crypto-balance-events
```

## 📡 Eventos Kafka

A aplicação suporta os seguintes eventos:

```json
{
  "RunBalanceUpdate": {
    "exchange": "Binance",
    "timestamp": "2024-01-01T12:00:00Z"
  }
}

{
  "RunPriceUpdate": {
    "timestamp": "2024-01-01T12:00:00Z"
  }
}

{
  "RunDebankUpdate": {
    "timestamp": "2024-01-01T12:00:00Z"
  }
}

{
  "HealthCheck": {
    "timestamp": "2024-01-01T12:00:00Z"
  }
}
```

## 🔄 Fluxo da Arquitetura

1. **Entry Point** (CLI/Kafka) recebe comando/evento
2. **Primary Adapter** converte para chamada interna
3. **Application Service** orquestra a execução
4. **Use Cases/Routines** executam lógica de negócio  
5. **Secondary Adapters** fazem integrações externas
6. **Resultado** é retornado através das camadas

## 🧩 Extensibilidade

### Adicionando novo Primary Adapter (ex: HTTP REST)

1. Implementar o trait `CommandHandler`
2. Criar adapter em `adapters/primary/http.rs`
3. Registrar no container DI
4. Adicionar modo no `main.rs`

### Adicionando novo Secondary Adapter (ex: Database)

1. Implementar trait existente (ex: `BalanceRepository`)
2. Criar adapter em `adapters/secondary/database.rs`
3. Configurar no container DI

### Adicionando nova Routine

1. Implementar trait `Routine` em `core/application/`
2. Registrar na factory `create_routines()`

## 📊 Observabilidade

- **Logs**: `crypto_balance.log`
- **Tracing**: OpenTelemetry + Jaeger (http://localhost:16686)
- **Metrics**: Instrumentação via tracing

## 🛠️ Dependências Adicionadas

```toml
# Kafka
rdkafka = { version = "0.36", features = ["cmake-build", "ssl", "sasl"] }

# Dependency Injection
shaku = "0.6"
```

## 🔍 Benefícios da Arquitetura

1. **Testabilidade**: Fácil mock dos ports para testes
2. **Flexibilidade**: Múltiplos entry points (CLI + Kafka + futuro HTTP)
3. **Manutenibilidade**: Separação clara de responsabilidades  
4. **Extensibilidade**: Adicionar novos adapters sem modificar core
5. **Deploy Options**: CLI tool OU microsserviço OU ambos