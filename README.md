# Usage

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

api_key = "<REPLACE>"
```
3. Change output sheets and ranges under sheets/ranges.rs (these are Google Sheets' named ranges)
4. Run program
