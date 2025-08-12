# Crypto Balance - Workspace Multi-Crate

A aplicação foi refatorada para usar **Cargo Workspace** com múltiplos crates, seguindo arquitetura hexagonal:

```
crypto-balance/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── core/              # Library crate (shared business logic)
│   │   ├── src/
│   │   │   ├── ports/     # Interfaces/traits
│   │   │   ├── domain/    # Domain entities
│   │   │   ├── application/ # Use cases & services
│   │   │   ├── adapters/  # Secondary adapters
│   │   │   └── config/    # Configuration
│   │   └── Cargo.toml
│   ├── cli/               # CLI application
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── cli_adapter.rs
│   │   │   └── application_service_factory.rs
│   │   ├── Dockerfile
│   │   └── Cargo.toml
│   └── kafka/             # Kafka consumer application
│       ├── src/
│       │   ├── main.rs
│       │   ├── kafka_adapter.rs
│       │   └── application_service_factory.rs
│       ├── Dockerfile
│       └── Cargo.toml
├── build.sh               # Build script
├── docker-compose.kafka.yml
└── Config.toml            # Shared configuration
```

## 🚀 Build & Run

### Via Script de Build
```bash
# Compilar todos os crates
./build.sh

# Executar CLI
./target/release/crypto-balance-cli

# Executar Kafka consumer  
KAFKA_BROKERS=localhost:9092 ./target/release/crypto-balance-kafka
```

### Via Cargo
```bash
# Build individual
cargo build -p crypto-balance-core
cargo build -p crypto-balance-cli  
cargo build -p crypto-balance-kafka

# Run individual
cargo run -p crypto-balance-cli
cargo run -p crypto-balance-kafka
```

### Via Docker
```bash
# Build CLI image
docker build -f crates/cli/Dockerfile -t crypto-balance-cli .

# Build Kafka image  
docker build -f crates/kafka/Dockerfile -t crypto-balance-kafka .

# Run with docker-compose
docker-compose -f docker-compose.kafka.yml up
```

## 🏗️ Arquitetura por Crate

### `crypto-balance-core` (Library)
- **Responsabilidade**: Lógica de negócio compartilhada
- **Contém**: Ports, Domain, Application services, Secondary adapters
- **Usado por**: CLI e Kafka apps
- **Não executável**: É uma biblioteca

### `crypto-balance-cli` (Binary)  
- **Responsabilidade**: Interface de linha de comando
- **Contém**: CLI adapter, argument parsing, CLI-specific setup
- **Depende de**: `crypto-balance-core`
- **Executável**: `crypto-balance-cli`

### `crypto-balance-kafka` (Binary)
- **Responsabilidade**: Consumer de eventos Kafka
- **Contém**: Kafka adapter, event handling, Kafka-specific setup  
- **Depende de**: `crypto-balance-core`
- **Executável**: `crypto-balance-kafka`

## 🔄 Benefícios da Separação

### 1. **Reutilização de Código**
- Core business logic compartilhada
- Evita duplicação entre CLI e Kafka
- Fácil adicionar novos entry points (HTTP, gRPC, etc.)

### 2. **Build Otimizado**
- Compile apenas o que mudou  
- Deploy independente de cada aplicação
- Imagens Docker menores e específicas

### 3. **Testes Focados**
- Teste core business logic separadamente
- Teste adapters específicos isoladamente
- Mock interfaces facilmente

### 4. **Deploy Flexível**
- Deploy CLI standalone para automações
- Deploy Kafka consumer para event processing  
- Deploy ambos conforme necessidade

### 5. **Desenvolvimento Paralelo**
- Equipes podem trabalhar em apps diferentes
- Core library força contratos bem definidos
- Menos conflitos de merge

## 📦 Workspace Dependencies

As dependências são centralizadas no `Cargo.toml` root:
- Evita versões conflitantes
- Facilita upgrades 
- Reduz tempo de build com shared dependencies

## 🐳 Docker Strategy

Cada app tem seu próprio Dockerfile:
- **CLI**: Imagem leve para execução standalone
- **Kafka**: Inclui dependências específicas (librdkafka)
- **Multi-stage builds**: Otimiza tamanho final das imagens

## 🔧 Development Workflow

```bash
# 1. Trabalhar na core lib
cd crates/core
cargo test
cargo check

# 2. Testar CLI
cd ../cli  
cargo run -- health

# 3. Testar Kafka (precisa Kafka rodando)
cd ../kafka
KAFKA_BROKERS=localhost:9092 cargo run

# 4. Build workspace completo
cd ../..
cargo build --workspace
```

Esta estrutura prepara a aplicação para escala e manutenibilidade a longo prazo! 🚀