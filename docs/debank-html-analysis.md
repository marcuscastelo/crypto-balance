# DeBank HTML Analysis and Data Extraction

## Overview

Este documento detalha a análise do HTML da página do DeBank e o mapeamento para as entidades Rust do projeto crypto-balance. O objetivo é extrair dados de portfólio DeFi de forma estruturada.

## Estrutura HTML Identificada

### Padrões de Classes CSS

- `AssetsOnChain_chainName__jAJuC` - Nome da blockchain
- `AssetsOnChain_usdValue__I1B7X` - Valor USD por chain
- `L2AccountBalance_balance__SNabQ` - Saldo L2
- `HeaderInfo_value__7Nj3p` - Valores gerais do header
- `Portfolio_container__BVNTK` - Container principal do portfólio
- `ChainPopover_chainLableText__m-l3D` - Texto do seletor de chain

### Dados Extraídos do HTML Real

#### Chains Identificadas
1. **HyperEVM**: $14,185 (71%)
2. **Arbitrum**: $2,480 (12%)
3. **Ethereum**: $1,716 (9%)
4. **Base**: $833 (4%)
5. **Sonic**: Valor não especificado

#### Projetos DeFi Mencionados
- Aave V3 (`#arb_aave3`, `#metis_aave3`)
- Compound V3 (`#scrl_compound3`, `#base_compound3`)
- Venus (`#bsc_venus`)
- Pendle V2 (`#pendle2`)
- SkateFi
- Uniswap V3 (`#celo_uniswap3`)

#### Valores Totais
- **Total Portfolio**: $19,863
- **L2 Balance**: $1.6
- **Net Worth**: $3.7M
- **24h Change**: $17.2
- **Gas Saved**: $2.00

## Mapeamento para Entidades Rust

### Estrutura Hierárquica

```
Chain
├── name: String
├── wallet_info: Option<ChainWallet>
│   ├── usd_value: String
│   └── tokens: Vec<SpotTokenInfo>
└── project_info: Vec<Project>
    └── name: String
    └── trackings: Vec<ProjectTracking>
        ├── tracking_type: String
        └── token_sections: Vec<ProjectTrackingSection>
            ├── title: String
            └── tokens: Vec<TokenInfo>
```

### Tipos de Tracking Identificados

1. **Lending** - Aave, Compound, Venus
   - Seções: Supplied, Borrowed, Rewards
2. **Staking** - Lido, protocolos de staking
   - Seções: Staked, Rewards
3. **Liquidity Pool** - Uniswap, Curve, PancakeSwap
   - Seções: Liquidity, Rewards
4. **Farming** - Yield farming protocols
   - Seções: Farming, Rewards
5. **Perpetual** - GMX, dYdX
   - Seções: Positions, PnL
6. **Governance** - Tokens de governança
   - Seções: Staked, Voting Power

### Campos TokenInfo Mapeados

- `token_name`: Nome do token (ETH, USDC, etc.)
- `pool`: Nome do pool ou protocolo específico
- `balance`: Quantidade do token
- `rewards`: Recompensas acumuladas
- `unlock_time`: Tempo de desbloqueio (para tokens vestidos)
- `claimable_amount`: Quantidade disponível para claim
- `end_time`: Fim do período de staking/farming
- `usd_value`: Valor em USD
- `variant_header`: Tipo da posição (Supplied, Borrowed, etc.)

## Exemplos de Dados por Chain

### Ethereum
- **Wallet**: ETH, USDC, AAVE
- **Projetos**: Aave V3, Compound V3
- **Tracking Types**: Lending

### Arbitrum  
- **Wallet**: ETH, ARB, USDC
- **Projetos**: Aave V3, GMX, Pendle V2
- **Tracking Types**: Lending, Staking, Perpetual, Liquidity Pool

## JSON Schema Resultante

```json
{
  "portfolio": {
    "total_usd_value": "$19,863",
    "l2_balance": "$1.6",
    "chains": [
      {
        "name": "Ethereum",
        "wallet_info": {
          "usd_value": "$1,716",
          "tokens": [...]
        },
        "project_info": [
          {
            "name": "Aave V3",
            "trackings": [
              {
                "tracking_type": "Lending",
                "token_sections": [
                  {
                    "title": "Supplied",
                    "tokens": [...]
                  }
                ]
              }
            ]
          }
        ]
      }
    ]
  }
}
```

## Padrões de Extração Identificados

1. **Valores Monetários**: Sempre prefixados com `$` e podem conter vírgulas
2. **Percentuais**: Aparecem após valores USD (ex: "71%")
3. **Nomes de Tokens**: Geralmente em maiúsculas (ETH, USDC, AAVE)
4. **Nomes de Projetos**: Capitalizados (Aave V3, Compound V3)
5. **URLs de Âncoras**: Contêm chain e projeto (`#arb_aave3`)

## Considerações Técnicas

- O HTML é gerado dinamicamente (React/Next.js)
- Classes CSS são minificadas com hashes
- Dados podem estar em atributos `data-*`
- Valores podem estar em elementos `<span>` aninhados
- Imagens de tokens contêm URLs padronizadas do DeBank

## Próximos Passos

1. Desenvolver parser HTML específico para essas classes
2. Implementar mapeamento robusto para entidades Rust
3. Adicionar validação de dados extraídos
4. Criar testes com dados reais
5. Implementar fallbacks para mudanças na estrutura HTML

---

*Documento criado em: June 22, 2025*  
*Baseado na análise do HTML real do DeBank Profile Page*
