# ⚖️ lawyerr

![CI](https://img.shields.io/github/actions/workflow/status/nikuscs/lawyerr/ci.yml?branch=main&label=CI)
![Release](https://img.shields.io/github/v/release/nikuscs/lawyerr)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

Fast Rust CLI for searching Portuguese court jurisprudence (DGSI) and legislation (Diário da República), optimized for LLM consumption. Searches 10 courts in parallel, outputs clean markdown, and includes a REST API server mode.

## Quick start

Search all Portuguese courts for a term and get markdown output:

```bash
lawyerr dgsi search "usucapiao"
```

```markdown
# Search Results: "usucapiao"

## STJ — Supremo Tribunal de Justiça (12 results)

### 1. Acórdão 1234/20.0T8LSB.L1.S1
- **Date:** 2025-11-15
- **Summary:** Usucapião sobre prédio rústico não demarcado...
- **URL:** https://www.dgsi.pt/jstj.nsf/...

## Relação de Lisboa (8 results)
...
```

Search today's legislation:

```bash
lawyerr dr today
```

## Why?

- **Parallel search** — queries all 10 DGSI courts simultaneously, results in seconds
- **DR legislation search** — search Diário da República acts (Portarias, Decretos-Lei, etc.)
- **LLM-ready output** — markdown by default with compact mode that strips boilerplate
- **Latin-1 handling** — automatic ISO-8859-1 to UTF-8 decoding for DGSI's legacy encoding
- **Flexible output** — Markdown, JSON, Table. Pipe to `jq`, feed to scripts, or read in terminal
- **Server mode** — deploy as a REST API on Unraid, VPS, or any server
- **Smart retry** — exponential backoff with retryable vs fatal error distinction

## Install

### Binary

```bash
# From source (requires Rust 1.85+)
cargo install --git https://github.com/nikuscs/lawyerr

# Or clone and build
git clone https://github.com/nikuscs/lawyerr
cd lawyerr
cargo build --release
# Binary at target/release/lawyerr
```

Pre-built binaries available in [Releases](https://github.com/nikuscs/lawyerr/releases).

## Usage

### List available courts

```bash
lawyerr dgsi courts
```

**Supported courts:** STJ, STA, Conflitos, Relacao do Porto/Lisboa/Coimbra/Guimaraes/Evora, TCA Sul/Norte

### Search court decisions

```bash
# Search all courts
lawyerr dgsi search "usucapiao"

# Search specific court
lawyerr dgsi search "contrato trabalho" --court stj

# Date filtering
lawyerr dgsi search "despejo" --since 2024-01-01 --until 2024-12-31

# Relative date window
lawyerr dgsi search "arrendamento" --recent 1y

# Multiple courts
lawyerr dgsi search "responsabilidade civil" --court stj --court rel-porto

# Fetch full decision text for each result
lawyerr dgsi search "usucapiao" --court stj --fetch-full

# Limit results and sort by date
lawyerr dgsi search "trabalho" --limit 10 --sort date
```

### Fetch a single decision

```bash
lawyerr dgsi fetch "https://www.dgsi.pt/jstj.nsf/..."
```

### Search Diário da República

```bash
# Search recent acts (1st series)
lawyerr dr search --content atos-1 --recent 1w

# Search for Portarias only
lawyerr dr search --content atos-1 --type portaria --recent 1w

# Full-text search
lawyerr dr search "trabalho" --content atos-1 --recent 1m

# Search 2nd series
lawyerr dr search --content atos-2 --recent 1w

# Search judicial decisions published in DR
lawyerr dr search --content decisoes --recent 1m

# Date range
lawyerr dr search --since 2026-01-01 --until 2026-03-21

# Multiple content types
lawyerr dr search --content atos-1 --content atos-2 --recent 1w
```

**Content type aliases:** `atos-1` (1st series acts), `atos-2` (2nd series acts), `dr` (whole DR issues), `decisoes`/`jurisprudencia` (judicial decisions)

### Today's publications

```bash
lawyerr dr today
lawyerr dr today --type portaria
```

### List available act types

```bash
lawyerr dr types
```

**Act type aliases:** `portaria`, `lei`, `decreto-lei`, `despacho`, `decreto`, `aviso`, `resolucao` (Resolucao do Conselho de Ministros), `retificacao`, `decreto-regulamentar`, `lei-organica`

### Output formats

```bash
lawyerr dgsi search "usucapiao" --format json       # JSON (for scripts)
lawyerr dgsi search "usucapiao" --format markdown   # Markdown (default)
lawyerr dgsi search "usucapiao" --format table      # Table (for terminal)
```

### Global options

| Flag | Description |
|------|-------------|
| `--format` | Output format: markdown, json, table |
| `--output` | Write output to file (format auto-detected from extension) |
| `--no-compact` | Disable compact mode (include full boilerplate) |
| `--strip-stopwords` | Remove common stop words from output |
| `--proxy` | Proxy URL (socks5/http) |
| `--config` | Path to config file |
| `--quiet` | Suppress progress output |

## Configuration

Create `lawyerr.toml` in the current directory or `~/.config/lawyerr/lawyerr.toml`:

```toml
[dgsi]
courts = ["stj", "sta", "rel-porto", "rel-lisboa"]

[dr]
content_types = ["atos-1", "atos-2", "decisoes"]
act_types = []

[http]
delay_ms = 100
max_concurrent = 10
timeout_secs = 30
retries = 3

[output]
format = "markdown"
compact = true
strip_stopwords = false

[server]
host = "0.0.0.0"
port = 3000
```

## Server Mode

```bash
lawyerr serve --host 0.0.0.0 --port 3000
```

```bash
# Health check
curl http://localhost:3000/health

# Search DGSI
curl "http://localhost:3000/dgsi/search?q=usucapiao&court=stj&limit=10"

# Fetch a single decision
curl "http://localhost:3000/dgsi/fetch?url=https://www.dgsi.pt/jstj.nsf/..."

# List courts
curl http://localhost:3000/dgsi/courts

# JSON output
curl "http://localhost:3000/dgsi/search?q=usucapiao&format=json"

# Search DR
curl "http://localhost:3000/dr/search?content=atos-1&since=2026-03-01"

# DR with act type filter
curl "http://localhost:3000/dr/search?content=atos-1&type=portaria&limit=10"

# Today's DR publications
curl "http://localhost:3000/dr/today"

# List DR act types
curl "http://localhost:3000/dr/types?format=json"
```

## Docker

```bash
docker build -t lawyerr .
docker run -p 3000:3000 lawyerr serve
```

## How It Works

1. **DGSI** — constructs Lotus Domino FT search queries, fetches HTML tables from dgsi.pt, parses Latin-1 encoded responses, extracts structured decision metadata, and converts HTML to clean markdown
2. **DR** — initializes an OutSystems session (CSRF token + cookies), builds a ~30KB typed POST body with base64-encoded search params, queries the ElasticSearch backend, and renders results as markdown
3. **Compact mode** — post-processes markdown output to strip legal boilerplate, normalize whitespace, and optionally remove Portuguese stop words for minimal token usage
4. **Server mode** — wraps both modules in an Axum HTTP server with shared state, exposing REST endpoints that mirror the CLI interface

## Roadmap

- **DR document fetching** — individual document fetching from Diario da Republica (complex, low priority)

## Related Projects

- [🕷️ crauler](https://github.com/nikuscs/crauler) — Web crawler with proxy routing and HTML-to-Markdown
- [🦎 amz-crawler](https://github.com/nikuscs/amz-crawler) — Amazon product crawler with TLS fingerprinting
- [🕹️ scrauper](https://github.com/nikuscs/scrauper) — Multi-threaded ScreenScraper.fr scraper for ES-DE
- [⚖️ kante-kusta](https://github.com/nikuscs/kante-kusta) — KuantoKusta.pt price comparison CLI
- [🕵️ olx-tracker](https://github.com/nikuscs/olx-tracker) — Track OLX.pt listings and get alerts on deals

## Disclaimer

> This project is for **educational purposes and AI automation research only**.
> The authors are not responsible for any misuse or for any damages resulting from the use of this tool.
> Users are solely responsible for ensuring compliance with applicable laws and the terms of service
> of any websites accessed. This software is provided "as-is" without warranty of any kind.
>
> If you are a rights holder and wish to have this project removed, please [contact me](https://github.com/nikuscs).

> **Note:** This project was partially developed with AI assistance and may contain bugs or unexpected behavior. Use at your own risk.

## License

MIT — see `LICENSE`.
