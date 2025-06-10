---
applyTo: '**/*.rs'
---

# Copilot Coding Standards for crypto-balance

## General Principles
- Follow Rust best practices and idioms.
- Use clear, descriptive names for variables, functions, and types.
- Prefer immutability; use `let` over `let mut` unless mutation is required.
- Use `Arc` for shared ownership and concurrency, as seen in spreadsheet and config managers.
- Use `error_stack` for error handling and context propagation.
- Use `tracing` for logging and instrumentation, with `#[instrument]` on async/public methods.
- Prefer `async`/`await` for IO and network operations.
- Use `#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]` as appropriate for data structures.
- Use `mod` and `pub mod` to organize code into logical modules.
- Use `cfg(test)` and `#[test]` for unit tests, and keep tests close to the code they test.
- Use `report!` and `.attach_printable_lazy` for error context.
- Use `Box<str>` for heap-allocated strings in config structs.
- Use `LazyLock` for static initialization where needed.
- Use `HashMap` and `Vec` for collections, with clear key/value types.
- Use `Option` and `Result` for error and optional value handling.
- Use `unwrap` only in tests or when failure is impossible by construction.
- Use `format!` for string formatting, not string concatenation.
- Use `serde` for serialization/deserialization.
- Use `anyhow` for ad-hoc error handling in utility functions.
- Use `std::sync::Arc` for thread-safe shared state.
- Use `pub` visibility only when necessary.

## File/Module Organization
- Each domain area (blockchain, sheets, debank, etc.) is a module under `src/domain` or `src/infrastructure`.
- Application logic is under `src/application`.
- Main entry point is `src/main.rs`.
- Tests are included in the same file as the code, inside `mod tests { ... }`.
- Constants are in their own files (e.g., `constants.rs`).
- Google Sheets named ranges are defined in `sheets/ranges.rs`.
- Config structs are in `infrastructure/config/`.

## Error Handling
- Use `error_stack::Result<T, E>` for all fallible functions.
- Attach context to errors using `.attach_printable_lazy`.
- Use custom error types for domain-specific errors.
- Use `report!` macro to create error reports.

## Logging/Tracing
- Use `tracing::info!`, `tracing::error!`, etc. for logging.
- Use `#[instrument]` on async/public methods for automatic tracing.
- Use custom `PrettyFormatter` for pretty log output.

## Google Sheets Integration
- Use `SpreadsheetManager` for all Sheets operations.
- Use named ranges for all cell/column access.
- Use `write_named_cell`, `write_named_column`, and `write_named_two_columns` for writing.
- Use `get_named_range` and `CellRange` for range management.

## Token/Balance Handling
- Use `RelevantDebankToken` and `TokenMatch` enums for token matching.
- Use `HashMap<String, HashMap<String, f64>>` for token balances.
- Use spam filtering for tokens using `check_spam`.
- Use `format_balance` for parsing balances from strings.

## Tests
- Use `#[test]` for all unit tests.
- Test all conversions, parsing, and arithmetic for columns and tokens.
- Use `assert_eq!` and `assert!` for assertions.
- Test error cases as well as success cases.

## README/Documentation
- Document all configuration steps in `README.md`.
- List all required API keys and addresses.
- Document how to change output sheets/ranges.
- Document how to run the program.

## Licensing
- All code is MIT licensed (see LICENSE).

---
This file summarizes the coding standards and conventions for the `crypto-balance` codebase. All contributions and code changes must adhere to these standards.
