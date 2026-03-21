# ⚖️ lawyerr

![CI](https://github.com/nikuscs/lawyerr/actions/workflows/ci.yml/badge.svg)
![Release](https://img.shields.io/github/v/release/nikuscs/lawyerr)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

**Fast Rust CLI for searching Portuguese court jurisprudence (DGSI), optimized for LLM consumption.**

> **Disclaimer:** This project is for **educational purposes and AI automation research only**.
> The authors are not responsible for any misuse or for any damages resulting from the use of this tool.
> Users are solely responsible for ensuring compliance with applicable laws and the terms of service
> of any websites accessed. This software is provided "as-is" without warranty of any kind.
>
> If you are a rights holder and wish to have this project removed, please [contact me](https://github.com/nikuscs).

> **Note:** This project was partially developed with AI assistance and may contain bugs or unexpected behavior. Use at your own risk.

Search court decisions across all 10 Portuguese DGSI courts in parallel, with clean markdown output ready for LLM pipelines.

## Why?

- **Parallel search** — queries all 10 DGSI courts simultaneously, results in seconds
- **LLM-ready output** — markdown by default with compact mode that strips boilerplate
- **Latin-1 handling** — automatic ISO-8859-1 to UTF-8 decoding for DGSI's legacy encoding
- **Flexible output** — Markdown, JSON, Table. Pipe to `jq`, feed to scripts, or read in terminal
- **Server mode** — deploy as a REST API on Unraid, VPS, or any server
- **Smart retry** — exponential backoff with retryable vs fatal error distinction

## Install

```bash
# From source (requires Rust 1.85+)
cargo install --git https://github.com/nikuscs/lawyerr

# Or clone and build
git clone https://github.com/nikuscs/lawyerr
cd lawyerr
cargo build --release
```

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
```

## Docker

```bash
docker build -t lawyerr .
docker run -p 3000:3000 lawyerr serve
```

## Roadmap

- **Diario da Republica (DR)** — legislation search is planned but not yet implemented

## Related Projects

- [crauler](https://github.com/nikuscs/crauler) — Web crawler with proxy routing and HTML-to-Markdown
- [amz-crawler](https://github.com/nikuscs/amz-crawler) — Amazon product crawler with TLS fingerprinting

## License

MIT
