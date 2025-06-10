# crypto-balance: Overview and Technical Reference

## Table of Contents
- [crypto-balance: Overview and Technical Reference](#crypto-balance-overview-and-technical-reference)
  - [Table of Contents](#table-of-contents)
  - [Project Overview](#project-overview)
  - [Architecture and Organization](#architecture-and-organization)
  - [Main Flow](#main-flow)
  - [Main Components](#main-components)
    - [Debank Scraper](#debank-scraper)
    - [AaHParser](#aahparser)
    - [Google Sheets Integration](#google-sheets-integration)
    - [Routines](#routines)
  - [Patterns and Conventions](#patterns-and-conventions)
  - [Error Handling](#error-handling)
  - [Logging and Observability](#logging-and-observability)
  - [Tests](#tests)
  - [Code References](#code-references)

---

## Project Overview
**crypto-balance** is a Rust application for automating the collection, consolidation, and recording of crypto asset balances across multiple platforms (blockchains, exchanges, DeBank, etc.), with Google Sheets integration for automated financial reporting. The project prioritizes robustness, traceability, modularity, and ease of maintenance.

## Architecture and Organization
- **Modularization:**
  - `src/application/`: Business logic and routines (e.g., DebankRoutine, ExchangeBalancesRoutine).
  - `src/domain/`: Domain types, traits, and enums (e.g., Routine, Chain, TokenInfo).
  - `src/infrastructure/`: External integrations (e.g., Debank scraper, Google Sheets, exchanges, config).
  - `src/main.rs`: Entry point, orchestration, and logging/tracing initialization.
- **Configuration:**
  - Centralized in TOML files and structs under `infrastructure/config/`.
- **Tests:**
  - Included in the same files, under `mod tests { ... }`.

## Main Flow
1. Environment, logging, and tracing initialization (OpenTelemetry, file, terminal).
2. Instantiation of spreadsheet managers and balance repositories.
3. Execution of routines (parallel or sequential) to:
   - Collect balances from DeBank (automated web scraping via Fantoccini/geckodriver).
   - Collect balances from exchanges (Binance, Kraken, etc.).
   - Update named ranges in Google Sheets.
4. Detailed logging and safe resource shutdown.

## Main Components
### Debank Scraper
- Automates the browser (Firefox/geckodriver) to access and extract data from DeBank.
- Uses Fantoccini for browser control.
- Extracts information about wallets, projects, tokens, and balances.
- Converts raw data into Rust structs (`Chain`, `ChainWallet`, `Project`, etc.).

### AaHParser
- Processes data extracted from DeBank.
- Filters and categorizes only relevant tokens (defined in `RELEVANT_DEBANK_TOKENS`).
- Handles different types of tokens (simple, staking, lending, etc.).
- Organizes balances in `HashMap<String, HashMap<String, f64>>` for spreadsheet updates.

### Google Sheets Integration
- Uses `SpreadsheetManager` to read/write named ranges.
- Main methods: `write_named_cell`, `write_named_two_columns`, `get_named_range`.
- Named ranges defined in `sheets/ranges.rs`.

### Routines
- Each routine implements the `Routine` trait (e.g., `DebankRoutine`).
- Main methods: `run`, `main_routine`, `prefetch_named_ranges`.
- Robust error handling and detailed logging.

## Patterns and Conventions
- **Idiomatic Rust:** Use of `Arc`, immutability, modules, derive for structs.
- **Error handling:**
  - `error_stack` for propagation and context.
  - `.attach_printable_lazy` for additional context.
  - `report!` for custom error creation.
- **Logging/tracing:**
  - `tracing` and `#[instrument]` on public/async methods.
  - Custom layers for terminal, file, and OpenTelemetry.
- **Collections:**
  - `HashMap` and `Vec` for data organization.
- **Strings:**
  - `Box<str>` for heap-allocated strings in configs.
  - `format!` for formatting.
- **Tests:**
  - `#[test]` and asserts for parsing, conversion, and error validation.
- **License:**
  - All code is MIT.

## Error Handling
- Fallible functions return `error_stack::Result<T, E>`.
- Scraping, parsing, and integration errors are propagated with detailed context.
- Token match errors are treated as warnings and ignored when appropriate.

## Logging and Observability
- Detailed logs in terminal and file (`crypto_balance.log`).
- Integration with OpenTelemetry for distributed tracing.
- Use of `IndicatifLayer` for progress bar.
- Panic hook to log panics and ensure tracer shutdown.

## Tests
- Unit tests close to the code.
- Cover balance parsing, token matching, errors, and conversions.
- Use of `assert_eq!` and `assert!`.

## Code References
- **main.rs:** Orchestration, logging/tracing initialization, routine execution.
- **debank_routine.rs:** Logic for DeBank scraping and balance update routine.
- **aah_parser.rs:** Parsing and categorization of data extracted from DeBank.
- **debank_scraper.rs:** Browser automation and DeBank scraping.
- **fantoccini_scraper_driver.rs:** Initialization and management of the Fantoccini/geckodriver driver.
- **balance.rs:** Utility function for parsing balances from string to f64.
- **scraper_driver.rs:** Utilities for spawning geckodriver and generating random ports.

---

> **Tip:** Use this file as a quick reference for architecture, patterns, and main project points. For implementation details, check the corresponding code files.
