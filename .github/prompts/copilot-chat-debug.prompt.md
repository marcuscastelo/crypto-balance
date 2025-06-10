# Copilot Chat Debug Session Prompt

## Contexto
- Sessão de debug para integração e scraping de dados do Debank usando Selenium (undetected_chromedriver) em Python.
- O script principal é `debank_scraper.py`.
- Toda a saída do script (stdout e stderr) é salva em `/tmp/copilot-terminal` para análise.
- O objetivo é garantir que o output do Python esteja compatível com o esperado pelo backend Rust (função `explore_debank_profile`).

## Especificações do Script Python
- O script acessa https://debank.com/profile/<address> e extrai informações de todas as blockchains (chains) visíveis.
- Para cada chain:
  - Clica na chain para ativar.
  - Extrai `wallet_info` (valor em USD e lista de tokens).
  - Extrai `project_info` (lista de projetos, cada um com `name` e `trackings`).
    - Cada `trackings` contém `tracking_type` e `token_sections` (cada seção com `title` e `tokens`).
    - Para trackings do tipo "Lending": as seções são "Supplied", "Borrowed" e "Rewards".
    - Para outros tipos: a seção única deve ter `title: "<unused>"`.
- O resultado final é um dicionário JSON:

```json
{
  "Ethereum": {
    "name": "Ethereum",
    "wallet_info": {
      "usd_value": "14835090",
      "tokens": [
        {
          "name": "WHITE",
          "price": "$0.001222",
          "amount": "10,000,000,000",
          "usd_value": "$12,222,484"
        },
        // ... outros tokens ...
      ]
    },
    "project_info": [
      {
        "name": "Uniswap V2",
        "trackings": [
          {
            "tracking_type": "Liquidity Pool",
            "token_sections": [
              {
                "title": "<unused>",
                "tokens": [
                  {
                    "token_name": "EthereumArmy+ETH",
                    "pool": "EthereumArmy+ETH",
                    "balance": null,
                    "rewards": null,
                    "unlock_time": null,
                    "claimable_amount": null,
                    "end_time": null,
                    "usd_value": "$4,184.71",
                    "variant_header": null
                  }
                  // ... outros tokens ...
                ]
              }
            ]
          }
        ]
      },
      {
        "name": "Aave V2",
        "trackings": [
          {
            "tracking_type": "Lending",
            "token_sections": [
              { "title": "Supplied", "tokens": [] },
              { "title": "Borrowed", "tokens": [] },
              { "title": "Rewards", "tokens": [] }
            ]
          }
        ]
      }
      // ... outros projetos ...
    ]
  }
  // ... outras chains ...
}
```
- O script salva o resultado em `debank_output.json` e imprime no terminal.

## Comandos de debug
- Para rodar o script e capturar a saída:
  ```sh
  python3 debank_scraper.py 2>&1 | tee /tmp/copilot-terminal
  ```
- Para inspecionar a saída:
  ```sh
  tail -n 4000 /tmp/copilot-terminal
  ```

## Objetivo da sessão
- Garantir que o scraping está correto e o output JSON está compatível com o esperado pelo Rust.
- Investigar e corrigir eventuais erros de scraping, parsing ou estrutura de dados.
- Validar a integração ponta-a-ponta.

## Observação sobre compatibilidade project_info
- **Python:** Cada item de `project_info` contém os campos `name` e `trackings` (com `tracking_type`, `token_sections` e `tokens`).
- **Rust:** A struct `Project` espera exatamente esses campos.
- **Status:**
  - `tracking_type` já está correto para a maioria dos projetos.
  - O agrupamento de `token_sections` e os campos dos tokens seguem o contrato Rust, conforme exemplo acima.

---

> Use este prompt para iniciar sessões de debug semelhantes envolvendo scraping, integração Python↔Rust, análise de logs em `/tmp/copilot-terminal` e validação de compatibilidade de estruturas de dados.
