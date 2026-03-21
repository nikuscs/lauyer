# ⚖️ lauyer

![CI](https://github.com/nikuscs/lauyer/actions/workflows/ci.yml/badge.svg)
![Release](https://img.shields.io/github/v/release/nikuscs/lauyer)
![License](https://img.shields.io/badge/license-PolyForm%20Noncommercial-red.svg)

**Fast Rust CLI for searching Portuguese court jurisprudence (DGSI) and legislation (Diário da República), optimized for LLM consumption.**

> **Disclaimer:** This project is for **educational purposes and AI automation research only**.
> The authors are not responsible for any misuse or for any damages resulting from the use of this tool.
> Users are solely responsible for ensuring compliance with applicable laws and the terms of service
> of any websites accessed. This software is provided "as-is" without warranty of any kind.
>
> If you are a rights holder and wish to have this project removed, please [contact me](https://github.com/nikuscs).

> **Note:** This project was partially developed with AI assistance and may contain bugs or unexpected behavior. Use at your own risk.

## Why?

- **Parallel search** — queries all 10 DGSI courts simultaneously
- **DR legislation** — searches Diário da República acts (Portarias, Decretos-Lei, Leis, Despachos)
- **LLM-ready** — markdown output by default with compact mode that strips legal boilerplate
- **Latin-1 handling** — automatic ISO-8859-1 → UTF-8 decoding for DGSI
- **Flexible output** — Markdown, JSON, Table — pipe to `jq`, feed to scripts, or read in terminal
- **Server mode** — REST API via Axum for remote deployment
- **Smart retry** — exponential backoff, retryable vs fatal error distinction

## Install

```bash
# From source (requires Rust 1.85+)
cargo install --git https://github.com/nikuscs/lauyer

# Or clone and build
git clone https://github.com/nikuscs/lauyer
cd lauyer
cargo build --release
```

Pre-built binaries available in [Releases](https://github.com/nikuscs/lauyer/releases).

## DGSI — Court Jurisprudence

```bash
# Search all 10 courts in parallel
lauyer dgsi search "usucapião"

# Search specific court
lauyer dgsi search "contrato trabalho" --court stj

# Multiple courts
lauyer dgsi search "responsabilidade civil" --court stj --court rel-porto

# Date filtering
lauyer dgsi search "despejo" --since 2024-01-01 --until 2024-12-31
lauyer dgsi search "arrendamento" --recent 1y

# Sort by date, limit results
lauyer dgsi search "herança" --limit 10 --sort date

# Fetch full decision text for each result
lauyer dgsi search "abuso de direito" --court stj --limit 3 --fetch-full

# Fetch a single decision by URL
lauyer dgsi fetch "https://www.dgsi.pt/jstj.nsf/..."

# List all courts
lauyer dgsi courts
```

**Courts:** `stj`, `sta`, `conflitos`, `rel-porto`, `rel-lisboa`, `rel-coimbra`, `rel-guimaraes`, `rel-evora`, `tca-sul`, `tca-norte`

## DR — Diário da República

```bash
# Search Portarias from the past week
lauyer dr search --type portaria --recent 1w

# Full-text search
lauyer dr search "trabalho" --type decreto-lei --recent 1m

# Search 2nd series (Despachos, Avisos)
lauyer dr search --content atos-2 --type despacho --recent 1w

# Judicial decisions published in DR
lauyer dr search --content decisoes --recent 1m

# Date range
lauyer dr search --type portaria --since 2026-03-01 --until 2026-03-21

# Today's publications
lauyer dr today
lauyer dr today --type portaria

# List act types
lauyer dr types
```

**Content types:** `atos-1` (1st series), `atos-2` (2nd series), `dr` (whole DR issues), `decisoes` (judicial decisions)

**Act types:** `portaria`, `decreto-lei`, `lei`, `despacho`, `decreto`, `aviso`, `resolucao`, `retificacao`, `decreto-regulamentar`, `lei-organica`

## Output Formats

```bash
lauyer dgsi search "insolvência" --court stj --format markdown  # default
lauyer dgsi search "insolvência" --court stj --format json      # structured
lauyer dgsi search "insolvência" --court stj --format table     # terminal
```

## Global Options

| Flag | Description |
|------|-------------|
| `--format` | Output format: `markdown` (default), `json`, `table` |
| `--output` | Write to file (format auto-detected from extension) |
| `--no-compact` | Disable compact mode |
| `--strip-stopwords` | Remove Portuguese stop words |
| `--proxy` | Proxy URL (`socks5://`, `http://`) |
| `--config` | Path to config file |
| `--quiet` | Suppress progress bars |

## Configuration

Create `lauyer.toml` in the working directory or `~/.config/lauyer/lauyer.toml`:

```toml
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
lauyer serve --port 3000
```

**`GET /dgsi/search`** `?q=usucapiao&court=stj&limit=5&since=2024-01-01&sort=date&format=json&compact=true&fetch_full=false`

**`GET /dgsi/fetch`** `?url=https://www.dgsi.pt/...&format=md`

**`GET /dgsi/courts`** `?format=json`

**`GET /dr/search`** `?q=trabalho&type=portaria&content=atos-1&since=2026-03-01&limit=10&format=json`

**`GET /dr/today`** `?type=portaria&format=json`

**`GET /dr/types`** `?format=json`

**`GET /health`**

All endpoints default to markdown. Add `?format=json` for JSON. Env vars: `LAUYER_PORT`, `LAUYER_HOST`.

### Docker

```bash
docker build -t lauyer .
docker run -p 3000:3000 lauyer serve
```

## AI Agents

If you are an AI agent, you can use `lauyer` as a skill to search Portuguese legal databases. Install the binary and call it directly from your tool/shell integration.

### Quick setup

Download the pre-compiled binary for your platform from [Releases](https://github.com/nikuscs/lauyer/releases) and place it in your `PATH`.

```bash
# Search jurisprudence (returns markdown by default)
lauyer dgsi search "usucapião" --court stj --limit 5 --format json

# Search legislation
lauyer dr search "trabalho" --type decreto-lei --recent 1m --format json

# Fetch a full court decision
lauyer dgsi fetch "https://www.dgsi.pt/jstj.nsf/..." --format json
```

### As a REST API

```bash
lauyer serve --port 3000
# Then call endpoints: GET /dgsi/search?q=usucapiao&court=stj&limit=5&format=json
```

### Tips for agents

- Use `--format json` for structured output you can parse
- Use `--limit` to control result count and stay within context limits
- Use `--fetch-full` on searches to get full decision text inline
- Use `--no-compact` if you need the full unprocessed legal text
- Pipe through `jq` for field extraction: `lauyer dgsi search "dano" --format json | jq '.[].summary'`

Feel free to copy and adapt this tool's interface into your own skill definitions or MCP server configurations.

## Related Projects

- [🕷️ crauler](https://github.com/nikuscs/crauler) — Web crawler with proxy routing and HTML→Markdown
- [🦎 amz-crawler](https://github.com/nikuscs/amz-crawler) — Amazon product crawler with TLS fingerprinting
- [🕹️ scrauper](https://github.com/nikuscs/scrauper) — Multi-threaded ScreenScraper.fr scraper for ES-DE

## License

PolyForm Noncommercial 1.0.0 with AI Restriction — see `LICENSE`.

Personal and non-commercial use only. Commercial use, AI training, and AI crawling are prohibited.
