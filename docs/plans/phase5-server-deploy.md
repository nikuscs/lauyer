# Phase 5: HTTP Server & Deployment

**Goal:** Add `lawyerr serve` command that exposes both DGSI and DR as REST endpoints via Axum. Add Dockerfile for Unraid/VPS deployment. After this phase, any LLM with web fetch (claude.ai skills, etc.) can search Portuguese law via HTTP.

**Depends on:** Phase 2 (DGSI), Phase 3 (DR), Phase 4 (formatting)

**Reference projects:**
- `~/projects/olx-tracker` — Axum server wrapping CLI commands (`src/server/`). Check for: router setup, handler patterns, AppState, graceful shutdown. This is the closest reference for `lawyerr serve`.
- `~/projects/crauler` — Axum server with proxy routing. Check `crates/crauler/src/` for server patterns.

---

## Checklist

### Server Core (`src/server/mod.rs`)
- [ ] Define `AppState`:
  ```rust
  struct AppState {
      config: Config,
      http_client: HttpClient,    // shared for DGSI
      dr_session: RwLock<DrSession>, // DR session (refreshable)
  }
  ```
- [ ] Implement `lawyerr serve` command:
  - `--port` (default: 3000, env: `LAWYERR_PORT`)
  - `--host` (default: `0.0.0.0`, env: `LAWYERR_HOST`)
- [ ] Set up Axum router with all endpoints
- [ ] Graceful shutdown on SIGTERM/SIGINT (`tokio::signal`)
- [ ] Log startup: `Listening on http://{host}:{port}`

### DGSI Endpoints (`src/server/routes.rs`)
- [ ] `GET /dgsi/search` — query params:
  - `q` — search query (required)
  - `court` — court alias, repeatable (default: all)
  - `since`, `until` — date range (YYYY-MM-DD)
  - `limit` — max results per court (default: 50)
  - `sort` — `relevance` or `date`
  - `format` — `md` or `json` (default: `json` for API)
  - `compact` — `true`/`false` (default: `true`)
  - `fetch_full` — `true`/`false` (default: `false`)
- [ ] `GET /dgsi/fetch` — query params:
  - `url` — full DGSI URL to fetch
  - `format` — `md` or `json`
  - `compact` — `true`/`false`
- [ ] `GET /dgsi/courts` — returns JSON list of courts with aliases
- [ ] Reuse the same search/fetch logic as CLI — handlers map HTTP params to the same pipeline

### DR Endpoints (`src/server/routes.rs`)
- [ ] `GET /dr/search` — query params:
  - `q` — search text (optional)
  - `type` — act type filter, repeatable (e.g., `Portaria`)
  - `content` — content type alias (default: `atos-1`)
  - `since`, `until` — date range
  - `limit` — max results
  - `format` — `md` or `json`
  - `compact` — `true`/`false`
- [ ] `GET /dr/today` — query params:
  - `type` — act type filter (optional)
  - `content` — content type (default: `atos-1`)
  - `format` — `md` or `json`
- [ ] `GET /dr/types` — returns JSON list of available act types (from aggregation)
- [ ] `GET /dr/fetch` — fetch specific DR document (if applicable)
- [ ] Handle DR session refresh automatically on error

### Shared Endpoints
- [ ] `GET /health` — returns `{"status": "ok", "version": "0.1.0"}`
- [ ] CORS headers if needed (for browser-based clients)

### Error Handling
- [ ] Map `LawyerrError` to appropriate HTTP status codes:
  - `Http` → 502 (bad gateway — upstream failed)
  - `Parse` → 500 (internal — our parsing failed)
  - `Session` → 503 (service unavailable — DR session issue, retry)
  - `Config` → 500
- [ ] Return JSON error bodies: `{"error": "message"}`

### Docker / Deployment
- [ ] Create `Dockerfile`:
  ```dockerfile
  FROM rust:1-slim AS builder
  WORKDIR /app
  COPY . .
  RUN cargo build --release

  FROM debian:bookworm-slim
  RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
  COPY --from=builder /app/target/release/lawyerr /usr/local/bin/
  EXPOSE 3000
  CMD ["lawyerr", "serve", "--port", "3000", "--host", "0.0.0.0"]
  ```
- [ ] Create `.dockerignore`: `target/`, `.git/`, `docs/`
- [ ] Test `docker build -t lawyerr .`
- [ ] Test `docker run -p 3000:3000 lawyerr`
- [ ] Verify endpoints work from host: `curl http://localhost:3000/health`

### Verification
- [ ] `lawyerr serve` — starts server, shows listening address
- [ ] `curl http://localhost:3000/health` — returns OK
- [ ] `curl "http://localhost:3000/dgsi/search?q=usucapião&court=stj&limit=3"` — returns JSON results
- [ ] `curl "http://localhost:3000/dgsi/courts"` — returns court list
- [ ] `curl "http://localhost:3000/dr/search?type=Portaria&since=2026-03-14"` — returns Portarias
- [ ] `curl "http://localhost:3000/dr/today"` — returns today's publications
- [ ] `curl "http://localhost:3000/dr/types"` — returns act type list
- [ ] `curl "http://localhost:3000/dgsi/search?q=contrato&format=md"` — returns markdown
- [ ] Docker build + run works
- [ ] Graceful shutdown: `ctrl-c` stops cleanly, in-flight requests complete

---

## Architecture Notes

**State sharing:** Use `Arc<AppState>` passed to Axum via `.with_state()`. The `DrSession` needs `RwLock` because it may need refresh (write lock for refresh, read lock for searches).

**JSON default:** Server endpoints default to JSON (not markdown) since they're an API. CLI defaults to markdown. The `format` param overrides both.

**Keep handlers thin:** Handler functions should parse query params, build the search params struct, call the same pipeline as CLI, and format the response. No business logic in handlers.

**DR session lifecycle:** Initialize `DrSession` at server startup. If a search fails with session error, take write lock, refresh session, retry. Use `tokio::sync::RwLock` (not `std::sync::RwLock`) since refresh is async.

**Quality gate:** `cargo fmt --check && cargo clippy -- -D warnings && cargo test` must pass before this phase is complete.
