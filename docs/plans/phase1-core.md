# Phase 1: Project Scaffolding & Core Infrastructure

**Goal:** Set up the Rust project with clean architecture, shared types, HTTP client abstraction, config system, and error handling. No DGSI/DR logic yet — just the bones.

**Ref:** See `docs/plans/initial.md` for full context. See `docs/dr_request_template.json` for DR body template.

**Reference projects** (same author, same quality standards — check these for patterns):
- `~/projects/amz-crawler` — HTTP client with TLS fingerprinting, clap CLI, config layering, output formats (table/json/markdown/csv). Good reference for: Cargo.toml metadata, clippy lints, CI workflow, README style.
- `~/projects/olx-tracker` — reqwest + cookie jar, Axum server wrapping CLI, SQLite, notifications. Good reference for: server mode (`src/server/`), proxy handling, config with TOML, `fake_user_agent`.
- `~/projects/scrauper` — Rate limiting (semaphore + governor), retry with exponential backoff, progress bars (indicatif), parallel downloads. Good reference for: concurrency patterns, retry logic, progress feedback.
- `~/projects/crauler` — Workspace project, proxy router module, HTML-to-markdown (`htmd`), encoding (`encoding_rs`). Good reference for: proxy architecture, HTML→markdown conversion, Latin-1 decoding.
- `~/projects/ts-code-scan` — Lean CLI, rayon parallelism, `ignore` crate for file walking. Good reference for: simple clean architecture, compact output formats.

---

## Checklist

### Project Setup
- [ ] Initialize Cargo project: `cargo init --name lawyerr`
- [ ] Set up `Cargo.toml` with full metadata:
  ```toml
  [package]
  name = "lawyerr"
  version = "0.1.0"
  edition = "2024"
  rust-version = "1.85"
  description = "Fast CLI for searching Portuguese legal jurisprudence (DGSI) and legislation (Diário da República)"
  license = "MIT"
  repository = "https://github.com/nikuscs/lawyerr"
  readme = "README.md"
  keywords = ["legal", "portugal", "cli", "dgsi", "rust"]
  categories = ["command-line-utilities"]
  ```
- [ ] Add all dependencies (see initial plan Dependencies section)
- [ ] Add dev-dependencies: `wiremock`, `tokio-test`, `assert_cmd`, `tempfile`
- [ ] Add `rustfmt.toml`:
  ```toml
  edition = "2024"
  max_width = 100
  use_small_heuristics = "Max"
  ```
- [ ] Configure lints in `Cargo.toml` (NOT clippy.toml — matches existing projects):
  ```toml
  [lints.rust]
  unsafe_code = "forbid"

  [lints.clippy]
  all = { level = "warn", priority = -1 }
  pedantic = { level = "warn", priority = -1 }
  nursery = { level = "warn", priority = -1 }
  # Pragmatic allows for CLI work:
  module_name_repetitions = "allow"
  must_use_candidate = "allow"
  missing_errors_doc = "allow"
  missing_panics_doc = "allow"
  needless_pass_by_value = "allow"
  future_not_send = "allow"
  cast_possible_truncation = "allow"
  cast_sign_loss = "allow"
  # Strict denials:
  dbg_macro = "deny"
  todo = "deny"
  unimplemented = "deny"
  print_stdout = "deny"
  print_stderr = "deny"

  [profile.release]
  lto = true
  codegen-units = 1
  strip = true
  ```
- [ ] Add `.gitignore`:
  ```
  /target/
  .env
  *.log
  .DS_Store
  config.toml
  .idea/
  .vscode/
  *.swp
  *.swo
  *~
  .claude/settings.local.json
  tarpaulin-report.*
  lcov.info
  ```
- [ ] Add `LICENSE` (MIT, copyright nikuscs)
- [ ] Add `CLAUDE.md` (project-specific AI guidelines — see below)
- [ ] Add `README.md` (see below)
- [ ] Add `.github/workflows/ci.yml` (see below)
- [ ] Create module directory structure:
  ```
  src/
  ├── main.rs
  ├── cli.rs
  ├── config.rs
  ├── error.rs
  ├── http.rs
  ├── format.rs
  ├── compact.rs
  ├── dgsi/
  │   └── mod.rs
  ├── dr/
  │   └── mod.rs
  └── server/
      └── mod.rs
  ```
- [ ] Verify `cargo build` compiles with empty modules

### Error Handling (`src/error.rs`)
- [ ] Define `thiserror` error enum `LawyerrError` with variants:
  - `Http { source, url }` — reqwest errors with context
  - `Parse { message, source_url }` — HTML/JSON parse failures
  - `Encoding { message }` — Latin-1/UTF-8 issues
  - `Session { message }` — DR session/CSRF failures
  - `Config { message }` — config file errors
  - `Io { source }` — file I/O
- [ ] Implement `From<reqwest::Error>`, `From<std::io::Error>`, etc.
- [ ] Define `type Result<T> = std::result::Result<T, LawyerrError>`

### HTTP Client Abstraction (`src/http.rs`)
- [ ] Create `HttpClient` struct wrapping `reqwest::Client` with:
  - Cookie jar (`reqwest::cookie::Jar` via `Arc`) — shared jar, accessible for DR to set cookies programmatically
  - Optional proxy configuration
  - Default headers (User-Agent, Accept-Encoding)
  - Configurable timeout
- [ ] Expose `HttpClient::cookie_jar(&self) -> &Arc<reqwest::cookie::Jar>` — DR module needs direct access to set `PesquisaAvancada` cookie
- [ ] Expose `HttpClient::inner(&self) -> &reqwest::Client` — DR module needs the raw client for custom requests with extra headers (`outsystems-locale`, etc.)
- [ ] Implement `HttpClient::new(config: &HttpConfig) -> Result<Self>`
- [ ] Implement `HttpClient::get(url) -> Result<Response>` with retry logic
- [ ] Implement `HttpClient::post_json(url, body, headers) -> Result<Response>` with retry logic
- [ ] Implement retry with exponential backoff: `2^(attempt-1) * 500ms`, max 3 retries
- [ ] Distinguish retryable (timeout, 503, connection reset) vs fatal (404, 400) errors
- [ ] Add `HttpClient::get_latin1(url) -> Result<String>` — fetches and decodes Latin-1 to UTF-8 using `encoding_rs`
- [ ] Add `HttpClient::get_bytes(url) -> Result<Vec<u8>>` — raw bytes fetch (for Latin-1 decoding)

### Configuration (`src/config.rs`)
- [ ] Define `Config` struct with serde Deserialize:
  ```rust
  struct Config {
      dgsi: DgsiConfig,
      dr: DrConfig,
      http: HttpConfig,
      output: OutputConfig,
      server: ServerConfig,
  }
  ```
- [ ] Define sub-configs:
  - `DgsiConfig`: `courts: Vec<String>` (default court aliases to search)
  - `DrConfig`: `content_types: Vec<String>` (default content type aliases), `act_types: Vec<String>` (default act type filters)
  - `HttpConfig`: `proxy: Option<String>`, `delay_ms: u64` (default 100), `max_concurrent: usize` (default 10), `timeout_secs: u64` (default 30), `retries: u32` (default 3)
  - `OutputConfig`: `format: OutputFormat` (default markdown), `compact: bool` (default true), `strip_stopwords: bool` (default false)
  - `ServerConfig`: `host: String` (default "0.0.0.0"), `port: u16` (default 3000)
- [ ] Implement config loading: `./lawyerr.toml` → `~/.config/lawyerr/lawyerr.toml` → defaults
- [ ] All config fields should have sensible defaults via `#[serde(default)]` (don't require a config file)
- [ ] CLI flags override config file values (config provides defaults, CLI provides overrides)

### Shared Types (`src/format.rs`)
- [ ] Define `OutputFormat` enum: `Markdown`, `Json`, `Table`
- [ ] Define `Renderable` trait that DGSI and DR results both implement:
  ```rust
  trait Renderable {
      fn to_markdown(&self) -> String;
      fn to_json(&self) -> serde_json::Value;
  }
  ```
  Keep this minimal — compact/stopwords are applied as post-processing, NOT inside the trait.
- [ ] Define `SearchResponse` struct to wrap results from either source:
  ```rust
  struct SearchResponse {
      source: String,         // "dgsi" or "dr"
      query: String,
      total: u64,
      results: Vec<Box<dyn Renderable>>,
  }
  ```
- [ ] Implement output writer: `fn render(response: &SearchResponse, format: OutputFormat, compact: bool, strip_stopwords: bool) -> String`
  - Calls `Renderable::to_markdown()` or `to_json()` per result
  - Applies `compact_text()` as post-processing if `compact` is true
  - Applies `strip_stopwords()` as post-processing if enabled
- [ ] Implement `fn write_output(content: &str, output_path: Option<&Path>) -> Result<()>` — stdout or file
- [ ] Define shared `DateRange` struct:
  ```rust
  struct DateRange {
      since: Option<NaiveDate>,
      until: Option<NaiveDate>,
  }
  ```
- [ ] Implement `--recent` parsing: `parse_recent(s: &str) -> Result<NaiveDate>` — `1w`, `2w`, `1m`, `3m`, `6m`, `1y` → absolute date

### Compact Mode (`src/compact.rs`)
- [ ] Implement `compact_text(input: &str) -> String`:
  - Collapse multiple consecutive blank lines into one
  - Strip leading/trailing whitespace per line
  - Remove HTML tags if any remain (basic tag stripping)
  - Collapse multiple spaces into single space
- [ ] Define Portuguese legal boilerplate patterns to strip:
  - "Acordam no Tribunal da Relação de..."
  - "Acordam os Juízes..."
  - Repeated header lines
- [ ] Implement `strip_stopwords(input: &str) -> String` (opt-in):
  - Safe list: `o`, `a`, `os`, `as`, `um`, `uma`, `uns`, `umas`, `de`, `em`, `por`, `para`, `com`, `e`, `ou`, `que`, `mas`
  - Never-remove list: `não`, `sem`, `nem`, `nunca`, `nenhum`, `nenhuma`, `jamais`, `salvo`, `excepto`, `apenas`, `somente`
  - Word-boundary aware (don't strip partial words)

### CLI Skeleton (`src/cli.rs` + `src/main.rs`)
- [ ] Define clap derive structs with subcommands:
  ```rust
  #[derive(Parser)]
  struct Cli {
      #[command(subcommand)]
      command: Commands,
      #[arg(long, env = "LAWYERR_CONFIG")]
      config: Option<PathBuf>,
      #[arg(long)]
      proxy: Option<String>,
      #[arg(long, default_value = "markdown")]
      format: OutputFormat,
      #[arg(long)]
      output: Option<PathBuf>,
      #[arg(long)]
      no_compact: bool,
      #[arg(long)]
      strip_stopwords: bool,
      #[arg(long)]
      quiet: bool,
  }
  ```
- [ ] Define `Commands` enum: `Dgsi(DgsiCommands)`, `Dr(DrCommands)`, `Serve(ServeArgs)`
- [ ] Define `DgsiCommands`: `Search`, `Fetch`, `Courts`
- [ ] Define `DrCommands`: `Search`, `Fetch`, `Today`, `Types`
- [ ] Wire up `main.rs` with tokio runtime, tracing subscriber, config loading
- [ ] Subcommands should print "not implemented yet" for now

### Verification
- [ ] `cargo fmt --check` — no formatting issues
- [ ] `cargo clippy -- -D warnings` — zero warnings (treat warnings as errors)
- [ ] `cargo build` — compiles without errors
- [ ] `cargo test` — all unit tests pass
- [ ] `cargo run -- --help` — shows help with all subcommands
- [ ] `cargo run -- dgsi search "test" --help` — shows search flags
- [ ] `cargo run -- dr search "test" --help` — shows DR search flags

**Every phase must pass `cargo fmt --check && cargo clippy -- -D warnings && cargo test` before moving to the next.**

---

## Architecture Notes

**Keep modules decoupled:** `dgsi/` and `dr/` should not import from each other. Shared code lives in `http.rs`, `config.rs`, `format.rs`, `compact.rs`.

**Error propagation:** Use `anyhow` in `main.rs` and CLI commands. Use `thiserror` in library code (`dgsi/`, `dr/`, `http.rs`). This follows Rust convention: `thiserror` for libraries, `anyhow` for applications.

**No `unwrap()`/`expect()` in library code.** Propagate errors with `?`. Only use `expect()` in main.rs for truly unrecoverable situations (e.g., "failed to initialize tokio runtime").

**Naming:** snake_case for files/functions, PascalCase for types/enums, SCREAMING_SNAKE for constants. Follow `rustfmt` defaults.

**`HttpClient` is shared by both modules.** DGSI uses `get_latin1()` for HTML fetching. DR uses the same client but accesses `inner()` and `cookie_jar()` for custom OutSystems requests. The `HttpClient` owns the cookie jar — DR sets cookies on it programmatically before each search.

**Compact mode is post-processing, NOT baked into rendering.** The `Renderable::to_markdown()` produces raw markdown. `compact_text()` and `strip_stopwords()` are applied afterward in `format::render()`. This keeps the trait simple and the processing pipeline clear.

**Config layering:** TOML file sets defaults → CLI flags override. Use `Option<T>` for CLI fields and merge: `cli_value.unwrap_or(config_value)`. Don't make config file mandatory.

**No println!/eprintln!** — Use `tracing::info!`, `tracing::warn!`, `tracing::error!` for all output except the actual results. Results go to stdout, logs go to stderr via `tracing-subscriber`.

## CLAUDE.md Template

Create `CLAUDE.md` at project root with:
- [ ] Project overview (what lawyerr does, two modules: DGSI + DR)
- [ ] Tech stack (Rust 2024, tokio, reqwest, scraper, axum, clap)
- [ ] Architecture (module tree matching `src/` structure)
- [ ] Code standards: error handling (anyhow app / thiserror lib), async patterns, no unsafe, no println
- [ ] Common commands:
  ```
  cargo fmt
  cargo clippy --all-targets
  cargo test
  cargo build --release
  ```
- [ ] End-of-task workflow: **Always run `cargo fmt && cargo clippy --all-targets && cargo test` before committing**
- [ ] How to add a new DGSI court / DR content type / act type
- [ ] Reference to `docs/plans/initial.md` for API details

## README.md Template

Create `README.md` matching existing project style:
- [ ] Title with emoji: `# ⚖️ lawyerr`
- [ ] One-line tagline
- [ ] Disclaimer (educational/AI research use)
- [ ] Features section (bullet points with key differentiators)
- [ ] Install section (cargo install from git + release binaries)
- [ ] Usage examples with actual command output
- [ ] Configuration section (TOML example)
- [ ] How It Works (numbered technical pipeline)
- [ ] Related Projects (links to crauler, amz-crawler, olx-tracker)
- [ ] License (MIT)

## CI/CD (`.github/workflows/ci.yml`)

- [ ] Create CI workflow matching existing projects:
  ```yaml
  name: CI
  on:
    push: { branches: [main] }
    pull_request: { branches: [main] }
  concurrency:
    group: ${{ github.workflow }}-${{ github.ref }}
    cancel-in-progress: true
  env:
    CARGO_TERM_COLOR: always
    RUSTFLAGS: -Dwarnings
  jobs:
    check:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - run: cargo check --all-targets
    fmt:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
          with: { components: rustfmt }
        - run: cargo fmt --all -- --check
    clippy:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
          with: { components: clippy }
        - run: cargo clippy --all-targets
    test:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - run: cargo test
  ```

## Testability & Traits

The architecture must support unit testing with mocked HTTP. Use traits so modules don't depend on concrete HTTP implementations.

### HTTP Trait (`src/http.rs`)
- [ ] Define `HttpFetcher` trait:
  ```rust
  #[async_trait]
  trait HttpFetcher: Send + Sync {
      async fn get(&self, url: &str) -> Result<Vec<u8>>;
      async fn get_text(&self, url: &str) -> Result<String>;
      async fn post_json(&self, url: &str, body: &serde_json::Value, headers: &[(String, String)]) -> Result<String>;
  }
  ```
- [ ] `HttpClient` implements `HttpFetcher` (real HTTP)
- [ ] DGSI and DR modules accept `impl HttpFetcher` or `&dyn HttpFetcher`, NOT concrete `HttpClient`
- [ ] For tests, create `MockHttpFetcher` that returns canned HTML/JSON responses from fixtures

### Module Public APIs
- [ ] DGSI public API in `src/dgsi/mod.rs`:
  ```rust
  pub async fn search(fetcher: &dyn HttpFetcher, courts: &[Court], query: &str, opts: &SearchOpts) -> Result<Vec<(Court, Vec<DgsiSearchResult>)>>
  pub async fn fetch_decision(fetcher: &dyn HttpFetcher, url: &str) -> Result<DgsiDecision>
  ```
- [ ] DR public API in `src/dr/mod.rs`:
  ```rust
  pub async fn search(session: &DrSession, params: &DrSearchParams) -> Result<DrSearchResponse>
  ```
- [ ] These are the only public functions — internal parsing/building are `pub(crate)` or private

### Test Fixtures
- [ ] Create `tests/fixtures/` directory:
  - `dgsi_search_results.html` — captured HTML from a real DGSI search
  - `dgsi_decision.html` — captured HTML from a real decision page
  - `dr_search_response.json` — captured JSON from a real DR search
- [ ] Capture fixtures during development using the smoke test commands from `initial.md`
- [ ] Unit tests parse these fixtures without hitting the network

### Test Structure
- [ ] Unit tests in each module file (`#[cfg(test)] mod tests`)
- [ ] Integration tests in `tests/` directory (hit real APIs, run only with `--ignored` flag)
- [ ] `cargo test` runs unit tests only (fast, no network)
- [ ] `cargo test -- --ignored` runs integration tests (requires network)
