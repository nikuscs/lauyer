# ⚖️ lawyerr

Fast CLI for searching Portuguese legal jurisprudence (DGSI) and legislation (Diário da República), optimized for LLM consumption.

> **Disclaimer:** This tool is intended for educational and AI research purposes. Always verify legal information through official sources.

## Features

- **Dual-source search** — query DGSI court decisions and Diário da República legislation from one tool
- **Parallel court search** — searches all 10+ DGSI courts simultaneously
- **Clean markdown output** — structured for LLM consumption with compact mode (default)
- **Multiple output formats** — Markdown, JSON, Table
- **HTTP server mode** — deploy as a REST API on Unraid, VPS, or any server
- **Latin-1 handling** — automatic ISO-8859-1 → UTF-8 decoding for DGSI
- **Smart retry** — exponential backoff with retryable vs fatal error distinction
- **Configurable** — TOML config file with CLI flag overrides

## Install

```bash
# From source
cargo install --git https://github.com/nikuscs/lawyerr

# Or clone and build
git clone https://github.com/nikuscs/lawyerr
cd lawyerr
cargo build --release
```

## Usage

```bash
# Search DGSI (all courts)
lawyerr dgsi search "usucapião"

# Search specific court with date filter
lawyerr dgsi search "contrato trabalho" --court stj --since 2024-01-01

# Search Diário da República for portarias
lawyerr dr search "arrendamento" --type portaria --recent 1y

# Today's publications
lawyerr dr today

# JSON output
lawyerr dgsi search "usucapião" --format json

# Start HTTP server
lawyerr serve --port 3000
```

## Configuration

Create `lawyerr.toml` in the current directory or `~/.config/lawyerr/lawyerr.toml`:

```toml
[dgsi]
courts = ["stj", "sta", "rel-porto", "rel-lisboa"]

[dr]
content_types = ["atos-1", "atos-2", "decisoes"]

[http]
delay_ms = 100
max_concurrent = 10
timeout_secs = 30

[output]
format = "markdown"
compact = true
```

## How It Works

1. **DGSI**: Queries IBM Lotus Domino `.nsf` databases via GET requests, parses HTML tables, decodes Latin-1 responses
2. **DR**: Initializes OutSystems session (CSRF + cookies), builds typed search parameters, POSTs to screen service endpoints, parses ElasticSearch JSON responses
3. **Output**: Results implement `Renderable` trait → rendered to chosen format → compact mode strips boilerplate → written to stdout or file

## Related Projects

- [crauler](https://github.com/nikuscs/crauler) — Web crawler with proxy routing and HTML→Markdown
- [amz-crawler](https://github.com/nikuscs/amz-crawler) — Amazon product crawler with TLS fingerprinting
- [olx-tracker](https://github.com/nikuscs/olx-tracker) — OLX listing tracker with Axum server

## License

MIT
