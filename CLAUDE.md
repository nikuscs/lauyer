# lauyer

Fast Rust CLI for searching Portuguese legal information from two sources:

1. **DGSI** (dgsi.pt) — Court jurisprudence. 10+ court databases (STJ, STA, Relações, TCA). IBM Lotus Domino platform, HTML tables, Latin-1 encoded. GET-based search with Domino FT query syntax.
2. **Diário da República** (diariodarepublica.pt) — Official gazette (legislation, portarias, decretos-lei). OutSystems React SPA with ElasticSearch backend. Requires session init (CSRF token + cookies), typed POST body (~30KB), and PesquisaAvancada cookie with base64-encoded search params.

Output is optimized for LLM consumption: clean markdown by default, with compact mode stripping legal boilerplate. Also supports JSON and table output. Includes an HTTP server mode (Axum) for remote deployment.

## Tech Stack

Rust 2024 (MSRV 1.85) · tokio · reqwest · scraper · axum · clap · thiserror/anyhow · chrono · encoding_rs · base64

## Architecture

```
src/
├── lib.rs           # Public module exports
├── main.rs          # Entry point, tokio runtime, tracing, CLI dispatch
├── cli.rs           # Clap derive CLI (dgsi/dr/serve subcommands)
├── config.rs        # TOML config loading (./lauyer.toml → ~/.config/lauyer/ → defaults)
├── error.rs         # LauyerError enum (thiserror)
├── http.rs          # HttpClient + HttpFetcher trait, retry with backoff, Latin-1 decoding
├── format.rs        # OutputFormat, Renderable trait, render pipeline, DateRange, parse_recent
├── compact.rs       # compact_text (whitespace/HTML/boilerplate), strip_stopwords (Portuguese)
├── dgsi/mod.rs      # DGSI module (court search, decision fetching)
├── dr/mod.rs        # DR module (session init, search, document fetching)
└── server/mod.rs    # Axum HTTP server mode
tests/
├── compact_test.rs  # compact_text, strip_stopwords, boilerplate tests
├── config_test.rs   # Config defaults, TOML parsing tests
├── format_test.rs   # OutputFormat, parse_recent, render, write_output tests
└── fixtures/        # Captured HTML/JSON for offline parsing tests
```

## Code Standards

- **Error handling**: `thiserror` in library code (dgsi/, dr/, http.rs), `anyhow` in main.rs/CLI
- **No `unsafe`**: forbidden via lint
- **No `println!`/`eprintln!`**: denied by clippy. Use `tracing::{info,warn,error}!` for logs, `std::io::Write` for output
- **No `unwrap()`/`expect()` in library code**: propagate with `?`
- **Modules decoupled**: dgsi/ and dr/ never import from each other. Shared code lives in http.rs, config.rs, format.rs, compact.rs
- **No inline tests**: all tests go in separate files under `tests/`. Source files in `src/` must NOT contain `#[cfg(test)]` modules. Integration tests that hit real APIs use `#[ignore]` and run via `cargo test -- --ignored`

## Commands

```bash
cargo fmt                     # Format
cargo clippy --all-targets    # Lint (pedantic + nursery)
cargo test                    # Unit tests (no network)
cargo test -- --ignored       # Integration tests (needs network)
cargo build --release         # Release build (LTO, stripped)
```

**Always run before committing:** `cargo fmt && cargo clippy --all-targets && cargo test`

## Key Design Decisions

- **HttpFetcher trait**: modules accept `&dyn HttpFetcher`, not concrete `HttpClient`. Enables mocking in tests.
- **Compact mode is post-processing**: `Renderable::to_markdown()` produces raw markdown, `compact_text()` and `strip_stopwords()` are applied afterward in `format::render()`.
- **Config layering**: TOML file sets defaults → CLI flags override. All config fields have `#[serde(default)]`.
- **Cookie jar shared**: `HttpClient` owns an `Arc<cookie::Jar>`. DR module accesses it via `cookie_jar()` to set cookies programmatically.

## Adding New Courts / Content Types

- **DGSI courts**: add variant to court enum in `src/dgsi/mod.rs` with db name, view UNID, and alias
- **DR content types**: add to content type enum in `src/dr/mod.rs` with UUID and boolean field name. Values must be PascalCase (`AtosSerie1`, not `atosSerie1`)
- **DR act types**: add alias mapping in `src/dr/mod.rs`

## Reference

- `docs/real-world-test.md` — Real-world test scenarios and results
