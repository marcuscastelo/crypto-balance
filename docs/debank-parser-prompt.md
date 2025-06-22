# Prompt para LLM: Criação de Parser HTML DeBank → JSON

## Contexto

Você deve criar um programa Python capaz de extrair dados de portfólio DeFi de uma página HTML do DeBank e convertê-los para o formato JSON estruturado mostrado abaixo.

## Análise do HTML de Entrada

O HTML contém uma página de perfil do DeBank com as seguintes características:

### Classes CSS Importantes
- `AssetsOnChain_chainName__jAJuC` - Nome da blockchain
- `AssetsOnChain_usdValue__I1B7X` - Valor USD por chain  
- `L2AccountBalance_balance__SNabQ` - Saldo L2
- `HeaderInfo_value__7Nj3p` - Valores do header
- `Portfolio_container__BVNTK` - Container do portfólio
- `ChainPopover_chainLableText__m-l3D` - Seletor de chain

### Dados a Extrair
1. **Total Portfolio Value**: Valor total em USD
2. **L2 Balance**: Saldo L2 específico  
3. **Chains**: Lista de blockchains com valores
4. **Projects**: Projetos DeFi por chain (Aave, Compound, GMX, etc.)
5. **Tokens**: Tokens individuais com balances e valores USD

### Exemplo de Dados Reais Encontrados
- **HyperEVM**: $14,185 (71%)
- **Arbitrum**: $2,480 (12%) 
- **Ethereum**: $1,716 (9%)
- **Base**: $833 (4%)
- **Total**: $19,863
- **L2 Balance**: $1.6

## Formato JSON de Saída Esperado

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
          "tokens": [
            {
              "name": "ETH",
              "price": "$3,241.23",
              "amount": "0.35",
              "usd_value": "$1,134.43"
            },
            {
              "name": "USDC",
              "price": "$1.00", 
              "amount": "425.50",
              "usd_value": "$425.50"
            }
          ]
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
                    "tokens": [
                      {
                        "token_name": "USDC",
                        "pool": null,
                        "balance": "2,500.00",
                        "rewards": null,
                        "unlock_time": null,
                        "claimable_amount": null,
                        "end_time": null,
                        "usd_value": "$2,500.00",
                        "variant_header": "Supplied"
                      }
                    ]
                  },
                  {
                    "title": "Borrowed", 
                    "tokens": [...]
                  },
                  {
                    "title": "Rewards",
                    "tokens": [...]
                  }
                ]
              }
            ]
          }
        ]
      },
      {
        "name": "Arbitrum",
        "wallet_info": {...},
        "project_info": [
          {
            "name": "GMX",
            "trackings": [
              {
                "tracking_type": "Staking",
                "token_sections": [...]
              },
              {
                "tracking_type": "Perpetual", 
                "token_sections": [...]
              }
            ]
          }
        ]
      }
    ]
  }
}
```

## Requisitos do Programa Python

### Bibliotecas Sugeridas
- `beautifulsoup4` - Para parsing HTML
- `requests` - Para requisições HTTP (se necessário)
- `json` - Para output JSON
- `re` - Para regex de limpeza de dados
- `logging` - Para logs de debug

### Funcionalidades Necessárias

1. **Parser Principal**
   - Função `parse_debank_html(html_content: str) -> dict`
   - Retorna dicionário no formato JSON esperado

2. **Extração de Chains**
   - Buscar elementos com classe `AssetsOnChain_chainName__jAJuC`
   - Extrair valores USD adjacentes
   - Calcular percentuais quando disponíveis

3. **Extração de Projetos**
   - Identificar projetos DeFi por âncoras (`#arb_aave3`, `#bsc_venus`)
   - Mapear para nomes limpos (Aave V3, Venus)
   - Associar à chain correta

4. **Extração de Tokens**
   - Buscar elementos de token com classes específicas
   - Extrair nomes, quantidades, preços e valores USD
   - Limpar formatação ($, vírgulas, etc.)

5. **Categorização de Tracking Types**
   - Mapear projetos para tipos: Lending, Staking, Liquidity Pool, etc.
   - Criar seções apropriadas (Supplied, Borrowed, Rewards)

6. **Utilitários**
   - `clean_usd_value(value: str) -> str` - Remove formatação
   - `extract_number(text: str) -> float` - Extrai números
   - `map_project_name(raw_name: str) -> str` - Normaliza nomes

### Estrutura do Código

```python
import json
import re
from bs4 import BeautifulSoup
from typing import Dict, List, Optional

class DebankHTMLParser:
    def __init__(self):
        self.soup = None
        
    def parse_html(self, html_content: str) -> dict:
        """Parse HTML e retorna JSON estruturado"""
        pass
        
    def extract_total_values(self) -> dict:
        """Extrai valores totais do portfolio"""
        pass
        
    def extract_chains(self) -> List[dict]:
        """Extrai informações de todas as chains"""
        pass
        
    def extract_chain_tokens(self, chain_name: str) -> List[dict]:
        """Extrai tokens da wallet de uma chain"""
        pass
        
    def extract_chain_projects(self, chain_name: str) -> List[dict]:
        """Extrai projetos DeFi de uma chain"""
        pass
        
    def categorize_tracking_type(self, project_name: str) -> str:
        """Determina o tipo de tracking baseado no projeto"""
        pass

def main():
    # Ler arquivo HTML
    with open('debank_profile.html', 'r', encoding='utf-8') as f:
        html_content = f.read()
    
    # Processar
    parser = DebankHTMLParser()
    result = parser.parse_html(html_content)
    
    # Salvar JSON
    with open('portfolio_data.json', 'w', encoding='utf-8') as f:
        json.dump(result, f, indent=2, ensure_ascii=False)
        
    print("Parsing concluído! Arquivo salvo como portfolio_data.json")

if __name__ == "__main__":
    main()
```

### Regras de Mapeamento

1. **Chain Names**: Extrair exatamente como aparecem no HTML
2. **USD Values**: Sempre manter formato "$X,XXX.XX"  
3. **Token Names**: Converter para uppercase quando apropriado
4. **Project Names**: Mapear para nomes padronizados
5. **Tracking Types**: Usar enum fixo (Lending, Staking, etc.)

### Tratamento de Erros

- Valores não encontrados devem ser `null`
- Campos opcionais devem usar `None/null`
- Logs detalhados para debugging
- Validação de dados extraídos

### Exemplo de Uso

```bash
python debank_parser.py input.html output.json
```

## Entregável

Crie um programa Python completo e funcional que:
1. Leia o arquivo HTML fornecido
2. Extraia todos os dados relevantes
3. Gere JSON no formato exato especificado
4. Inclua tratamento de erros robusto
5. Tenha logs informativos para debugging
6. Seja bem documentado com docstrings

O programa deve ser capaz de processar o HTML real do DeBank e produzir o JSON estruturado conforme mostrado nos exemplos.
