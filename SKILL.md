---
name: lauyer
description: Search Portuguese court jurisprudence (DGSI) and legislation (Diário da República). Use for finding court decisions, laws, portarias, decretos-lei, and any Portuguese legal information.
metadata: {"openclaw":{"emoji":"⚖️","requires":{"bins":["lauyer"]},"install":[{"id":"binary","kind":"custom","command":"# Download from https://github.com/nikuscs/lauyer/releases/latest\n# macOS: lauyer-macos-arm64.tar.gz\n# Linux x64: lauyer-linux-x64.tar.gz\n# Windows: lauyer-windows-x64.zip\ntar -xzf lauyer-*.tar.gz && chmod +x lauyer && sudo mv lauyer /usr/local/bin/","label":"Download pre-built binary (recommended)"},{"id":"cargo","kind":"cargo","crate":"lauyer","bins":["lauyer"],"label":"Install via Cargo (requires Rust)"}]}}
---

# Lauyer Skill

Search Portuguese court jurisprudence (DGSI) and legislation (Diário da República). Outputs clean markdown optimized for LLM consumption.

## Installation

**No Rust or compilation required.** Download the pre-built binary for your platform from [Releases](https://github.com/nikuscs/lauyer/releases/latest):

### macOS (Apple Silicon)

```bash
curl -L https://github.com/nikuscs/lauyer/releases/latest/download/lauyer-macos-arm64.tar.gz | tar xz
chmod +x lauyer
sudo mv lauyer /usr/local/bin/
```

### Linux (x64)

```bash
curl -L https://github.com/nikuscs/lauyer/releases/latest/download/lauyer-linux-x64.tar.gz | tar xz
chmod +x lauyer
sudo mv lauyer /usr/local/bin/
```

### Windows

Download `lauyer-windows-x64.zip` from [Releases](https://github.com/nikuscs/lauyer/releases/latest), extract, and add the folder to your `PATH`.

### Verify

```bash
lauyer --help
```

## When to use (trigger phrases)

Use this skill when the user asks:

- "search Portuguese law for..."
- "find court decisions about..."
- "what does Portuguese law say about..."
- "search DGSI for..."
- "find jurisprudência about..."
- "search Diário da República..."
- "latest portarias about..."
- "decretos-lei about..."
- "today's legislation"
- "what laws were published today?"
- Any mention of Portuguese law, court decisions, jurisprudence, DGSI, Diário da República, portarias, decretos-lei, leis, despachos, or legal research in Portugal

## DGSI — Court Jurisprudence

Search court decisions across 10 Portuguese courts in parallel.

```bash
# Search all courts
lauyer dgsi search "usucapião"

# Search specific court
lauyer dgsi search "contrato trabalho" --court stj

# Multiple courts
lauyer dgsi search "responsabilidade civil" --court stj --court rel-porto

# Date filtering
lauyer dgsi search "despejo" --since 2024-01-01 --until 2024-12-31
lauyer dgsi search "arrendamento" --recent 1y

# Sort and limit
lauyer dgsi search "herança" --limit 10 --sort date

# Fetch full decision text
lauyer dgsi search "abuso de direito" --court stj --limit 3 --fetch-full

# Fetch a single decision by URL
lauyer dgsi fetch "https://www.dgsi.pt/jstj.nsf/..."

# List all courts
lauyer dgsi courts
```

**Courts:** `stj`, `sta`, `conflitos`, `rel-porto`, `rel-lisboa`, `rel-coimbra`, `rel-guimaraes`, `rel-evora`, `tca-sul`, `tca-norte`

### DGSI Search Options

| Flag | Description | Example |
|------|-------------|---------|
| `--court` | Filter by court (repeatable) | `--court stj --court sta` |
| `--since` | Earliest date (YYYY-MM-DD) | `--since 2024-01-01` |
| `--until` | Latest date (YYYY-MM-DD) | `--until 2024-12-31` |
| `--recent` | Relative window (30d, 6m, 1y) | `--recent 1y` |
| `--limit` | Max results (default: 50) | `--limit 10` |
| `--sort` | Sort order: relevance, date | `--sort date` |
| `--fetch-full` | Fetch full decision text | `--fetch-full` |
| `--field` | Structured search field name | `--field relator` |
| `--value` | Structured search field value | `--value "Santos Cabral"` |

## DR — Diário da República

Search legislation, portarias, decretos-lei, and other official acts.

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

# List available act types
lauyer dr types
```

**Content types:** `atos-1` (1st series), `atos-2` (2nd series), `dr` (whole DR issues), `decisoes` (judicial decisions)

**Act types:** `portaria`, `decreto-lei`, `lei`, `despacho`, `decreto`, `aviso`, `resolucao`, `retificacao`, `decreto-regulamentar`, `lei-organica`

### DR Search Options

| Flag | Description | Example |
|------|-------------|---------|
| `--type` | Act type filter (repeatable) | `--type portaria --type lei` |
| `--content` | Content type filter (repeatable) | `--content atos-1` |
| `--since` | Earliest date (YYYY-MM-DD) | `--since 2026-01-01` |
| `--until` | Latest date (YYYY-MM-DD) | `--until 2026-03-21` |
| `--recent` | Relative window (1w, 1m, 1y) | `--recent 1w` |
| `--limit` | Max results (default: 50) | `--limit 10` |

## Output Formats

```bash
lauyer dgsi search "insolvência" --court stj --format json       # structured JSON
lauyer dgsi search "insolvência" --court stj --format table      # terminal table
lauyer dr search --type lei --recent 1m --output resultados.json  # write to file (format auto-detected)
```

### Compact Mode

```bash
lauyer dgsi search "penhora" --no-compact       # full output, no boilerplate stripping
lauyer dr search "IRS" --recent 1w --strip-stopwords  # remove Portuguese stop words
```

## Global Options

| Flag | Description | Example |
|------|-------------|---------|
| `--format` | Output: markdown, json, table | `--format json` |
| `--output` | Write to file | `--output results.md` |
| `--no-compact` | Disable boilerplate stripping | `--no-compact` |
| `--strip-stopwords` | Remove Portuguese stop words | `--strip-stopwords` |
| `--proxy` | Proxy URL (socks5/http) | `--proxy socks5://host:port` |
| `--config` | Path to config file | `--config ~/lauyer.toml` |
| `--quiet` | Suppress progress bars | `--quiet` |

## Common Workflows

### Research a legal topic

```bash
# 1. Get recent STJ decisions
lauyer dgsi search "usucapião" --court stj --recent 2y --sort date --limit 20

# 2. Get full text of the most relevant ones
lauyer dgsi search "usucapião" --court stj --recent 2y --sort date --limit 3 --fetch-full

# 3. Check for related legislation
lauyer dr search "usucapião" --content atos-1 --recent 1y
```

### Monitor new legislation

```bash
# Today's portarias
lauyer dr today --type portaria

# This week's decretos-lei
lauyer dr search --type decreto-lei --recent 1w

# All publications from a date range
lauyer dr search --since 2026-03-01 --until 2026-03-21
```

### Cross-reference courts

```bash
# Same topic across multiple courts
lauyer dgsi search "responsabilidade civil" --court stj --limit 5
lauyer dgsi search "responsabilidade civil" --court sta --limit 5
lauyer dgsi search "responsabilidade civil" --court rel-porto --limit 5
```

### JSON pipeline

```bash
# Extract just case references
lauyer dgsi search "insolvência" --court stj --format json | jq '.[].processo'

# Count results per court
lauyer dgsi search "trabalho" --format json | jq 'group_by(.court) | map({court: .[0].court, count: length})'
```

## HTTP API

If the user has a lauyer server running, they must provide the base URL (e.g. `http://localhost:3000` or `https://lauyer.example.com`). **Do not guess the URL** — ask the user for it. To start a local server:

```bash
lauyer serve --port 3000
# Env vars: LAUYER_PORT, LAUYER_HOST
```

All endpoints return markdown by default. Add `?format=json` for JSON, `?format=table` for table. Examples below use `$LAUYER_URL` as a placeholder — replace with the actual server URL.

### `GET /health`

Returns `{"status":"ok","version":"0.1.0"}`.

### `GET /dgsi/search`

| Param | Required | Description |
|-------|----------|-------------|
| `q` | yes | Search query |
| `court` | no | Comma-separated court aliases (default: all) |
| `since` | no | Start date `YYYY-MM-DD` |
| `until` | no | End date `YYYY-MM-DD` |
| `limit` | no | Max results (default: 50) |
| `sort` | no | `relevance` (default) or `date` |
| `fetch_full` | no | `true` to fetch full decision text |
| `compact` | no | `true`/`false` override compact mode |
| `format` | no | `markdown`, `json`, `table` |

```bash
curl "$LAUYER_URL/dgsi/search?q=usucapiao&court=stj,sta&limit=5&sort=date&format=json"
```

### `GET /dgsi/fetch`

| Param | Required | Description |
|-------|----------|-------------|
| `url` | yes | Full DGSI decision URL |
| `format` | no | `markdown`, `json`, `table` |
| `compact` | no | `true`/`false` |

```bash
curl "$LAUYER_URL/dgsi/fetch?url=https://www.dgsi.pt/jstj.nsf/...&format=json"
```

### `GET /dgsi/courts`

| Param | Required | Description |
|-------|----------|-------------|
| `format` | no | `markdown` (default) or `json` |

### `GET /dr/search`

| Param | Required | Description |
|-------|----------|-------------|
| `q` | no | Full-text search query |
| `type` | no | Comma-separated act type aliases (`portaria`, `lei`, etc.) |
| `content` | no | Comma-separated content types (default: `atos-1`) |
| `since` | no | Start date `YYYY-MM-DD` |
| `until` | no | End date `YYYY-MM-DD` |
| `limit` | no | Max results (default: 50) |
| `compact` | no | `true`/`false` |
| `format` | no | `markdown`, `json`, `table` |

```bash
curl "$LAUYER_URL/dr/search?q=trabalho&type=portaria,lei&content=atos-1&since=2026-03-01&format=json"
```

### `GET /dr/today`

| Param | Required | Description |
|-------|----------|-------------|
| `type` | no | Comma-separated act type aliases |
| `compact` | no | `true`/`false` |
| `format` | no | `markdown`, `json`, `table` |

```bash
curl "$LAUYER_URL/dr/today?type=portaria&format=json"
```

### `GET /dr/types`

| Param | Required | Description |
|-------|----------|-------------|
| `format` | no | `markdown` (default) or `json` |

### `GET /dr/fetch`

Returns `501 Not Implemented` — individual DR document fetching is not yet supported.

## Agent Guidelines

### Best Practices

1. **Default to markdown** output when presenting to users — it's pre-compacted for LLM context
2. **Use `--format json`** when you need to process or filter results programmatically
3. **Limit results** with `--limit 5-10` for quick answers, `--limit 20-50` for thorough research
4. **Use `--fetch-full`** only when the user needs actual decision text, not just metadata
5. **Use `--recent`** to scope searches temporally — `1w`, `1m`, `6m`, `1y`
6. **Combine `--court` with `--sort date`** for targeted, chronological results

### Response Formatting

When presenting results to users:

- Show the most relevant 3-5 results with case number, date, court, and summary
- Include the DGSI URL for each result so users can read the full decision
- Mention total results found and suggest narrowing if too many
- For DR results, highlight the act type, number, and publication date
- Offer to fetch full text if the user wants to read a specific decision

### Understanding Courts

- **STJ** (Supremo Tribunal de Justiça) — Supreme Court, highest civil/criminal court
- **STA** (Supremo Tribunal Administrativo) — Supreme Administrative Court
- **Relações** (rel-porto, rel-lisboa, etc.) — Appeals courts by region
- **TCA** (tca-sul, tca-norte) — Central Administrative Courts
- **Conflitos** — Conflicts tribunal (jurisdiction disputes)

Start with STJ for authoritative precedent, use Relações for regional case law.

## Proxy Support

```bash
# SOCKS5 proxy
lauyer --proxy "socks5://127.0.0.1:1080" dgsi search "trabalho"

# HTTP proxy
lauyer --proxy "http://proxy:8080" dgsi search "trabalho"
```

## Tips

- DGSI uses Latin-1 encoding — lauyer handles the conversion automatically
- DR requires session initialization (CSRF + cookies) — lauyer handles this transparently
- Compact mode strips legal boilerplate headers — use `--no-compact` for raw output
- `--strip-stopwords` removes common Portuguese words for minimal token usage
- Configuration can be set via `lauyer.toml` in the working directory or `~/.config/lauyer/lauyer.toml`
