# Phase 1: Project Scaffolding & Core Infrastructure

**Goal:** Set up the Rust project with clean architecture, shared types, HTTP client abstraction, config system, and error handling. No DGSI/DR logic yet — just the bones.

**Ref:** See `docs/plans/initial.md` for full context. See `docs/dr_request_template.json` for DR body template.

---

## Checklist

### Project Setup
- [ ] Initialize Cargo project: `cargo init --name lawyerr`
- [ ] Set up `Cargo.toml` with all dependencies (see initial plan Dependencies section)
- [ ] Set Rust edition 2024, minimum Rust version
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
  - Cookie jar (`reqwest::cookie::Jar` via `Arc`)
  - Optional proxy configuration
  - Default headers (User-Agent, Accept-Encoding)
  - Configurable timeout
- [ ] Implement `HttpClient::new(config: &HttpConfig) -> Result<Self>`
- [ ] Implement `HttpClient::get(url) -> Result<Response>` with retry logic
- [ ] Implement `HttpClient::post_json(url, body, headers) -> Result<Response>` with retry logic
- [ ] Implement retry with exponential backoff: `2^(attempt-1) * 500ms`, max 3 retries
- [ ] Distinguish retryable (timeout, 503, connection reset) vs fatal (404, 400) errors
- [ ] Add `HttpClient::get_latin1(url) -> Result<String>` — fetches and decodes Latin-1 to UTF-8 using `encoding_rs`

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
- [ ] Define sub-configs matching `lawyerr.toml` fields from the plan
- [ ] Implement config loading: `./lawyerr.toml` → `~/.config/lawyerr/lawyerr.toml` → defaults
- [ ] All config fields should have sensible defaults (don't require a config file)
- [ ] `HttpConfig`: proxy, delay_ms, max_concurrent, timeout_secs, retries
- [ ] `OutputConfig`: format (markdown/json/table), compact (default true), strip_stopwords (default false)

### Shared Types (`src/format.rs`)
- [ ] Define `OutputFormat` enum: `Markdown`, `Json`, `Table`
- [ ] Define `SearchResult` trait that DGSI and DR results both implement:
  ```rust
  trait SearchResult {
      fn to_markdown(&self, compact: bool) -> String;
      fn to_json(&self) -> serde_json::Value;
      fn date(&self) -> Option<chrono::NaiveDate>;
      fn title(&self) -> &str;
      fn summary(&self) -> &str;
  }
  ```
- [ ] Implement output writer: `write_results(results, format, output_path_or_stdout)`

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
- [ ] `cargo build` — compiles without errors
- [ ] `cargo clippy` — no warnings
- [ ] `cargo run -- --help` — shows help with all subcommands
- [ ] `cargo run -- dgsi search "test" --help` — shows search flags
- [ ] `cargo run -- dr search "test" --help` — shows DR search flags

---

## Architecture Notes

**Keep modules decoupled:** `dgsi/` and `dr/` should not import from each other. Shared code lives in `http.rs`, `config.rs`, `format.rs`, `compact.rs`.

**Error propagation:** Use `anyhow` in `main.rs` and CLI commands. Use `thiserror` in library code (`dgsi/`, `dr/`, `http.rs`). This follows Rust convention: `thiserror` for libraries, `anyhow` for applications.

**No `unwrap()`/`expect()` in library code.** Propagate errors with `?`. Only use `expect()` in main.rs for truly unrecoverable situations (e.g., "failed to initialize tokio runtime").

**Naming:** snake_case for files/functions, PascalCase for types/enums, SCREAMING_SNAKE for constants. Follow `rustfmt` defaults.
