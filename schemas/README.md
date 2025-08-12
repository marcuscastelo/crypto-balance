# DeBank Scraper Data Models & API Schemas

JSON schemas for all Pydantic models used in the DeBank scraper and REST API.

## Core Data Models

### SpotTokenInfo

Model for spot wallet token information

Schema file: [`SpotTokenInfo.json`](./SpotTokenInfo.json)

### WalletInfo

Model for aggregated wallet information per chain

Schema file: [`WalletInfo.json`](./WalletInfo.json)

### TokenInfo

Model for project token information (supplied, borrowed, rewards, etc.)

Schema file: [`TokenInfo.json`](./TokenInfo.json)

### TokenSection

Model for grouped token sections (e.g., 'Supplied', 'Borrowed', 'Rewards')

Schema file: [`TokenSection.json`](./TokenSection.json)

### Tracking

Model for position tracking within a project (lending, staking, etc.)

Schema file: [`Tracking.json`](./Tracking.json)

### Project

Model for DeFi project information

Schema file: [`Project.json`](./Project.json)

### ChainInfo

Model for blockchain chain information

Schema file: [`ChainInfo.json`](./ChainInfo.json)

### PortfolioMetadata

Model for portfolio metadata

Schema file: [`PortfolioMetadata.json`](./PortfolioMetadata.json)

### PortfolioData

Root model for complete portfolio data

Schema file: [`PortfolioData.json`](./PortfolioData.json)

## API Models

### ScrapeRequest

No description available.

Schema file: [`ScrapeRequest.json`](./ScrapeRequest.json)

### ScrapeResponse

No description available.

Schema file: [`ScrapeResponse.json`](./ScrapeResponse.json)

### JobStatusResponse

No description available.

Schema file: [`JobStatusResponse.json`](./JobStatusResponse.json)

### JobResultResponse

No description available.

Schema file: [`JobResultResponse.json`](./JobResultResponse.json)

