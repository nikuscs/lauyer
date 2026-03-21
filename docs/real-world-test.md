# Real-World Test Plan

Manual verification checklist for lawyerr CLI and server. Run against live DGSI and DR APIs.

**Last tested:** 2026-03-21

---

## CLI — DGSI

### `dgsi courts`
```bash
lawyerr dgsi courts
```
- [ ] Lists all 10 courts with aliases
- [ ] Portuguese characters render correctly (Relação, Guimarães, Évora)

### `dgsi search` — Basic
```bash
lawyerr dgsi search "usucapião" --court stj --limit 3
```
- [ ] Returns markdown with results
- [ ] Portuguese characters correct (ã, ç, õ, é)
- [ ] Shows processo, date, relator, descritores per result
- [ ] Total count shown (should be ~1000)

### `dgsi search` — All courts (no --court)
```bash
lawyerr dgsi search "usucapião" --limit 1
```
- [ ] Searches all 10 courts in parallel
- [ ] Returns results from multiple courts
- [ ] Failed courts logged but don't block others

### `dgsi search` — Multiple courts
```bash
lawyerr dgsi search "contrato trabalho" --court stj --court rel-porto --limit 2
```
- [ ] Both courts return results
- [ ] Each court section has its own header

### `dgsi search` — JSON format
```bash
lawyerr --format json dgsi search "usucapião" --court stj --limit 3
```
- [ ] Valid JSON output (pipe to `jq .` to verify)
- [ ] Contains source, query, total, results array
- [ ] Each result has processo, date, relator, relevance, descriptors, url

### `dgsi search` — Table format
```bash
lawyerr --format table dgsi search "usucapião" --court stj --limit 3
```
- [ ] Aligned columns with headers (Date, Processo, Relator, Descritores)
- [ ] Separator row between headers and data
- [ ] Long descriptors truncated with `...`

### `dgsi search` — Date filtering with --since
```bash
lawyerr dgsi search "usucapião" --court stj --since 2024-01-01 --limit 2
```
- [ ] Query contains `AND [DATAAC] > 01/01/2024`
- [ ] Results are from 2024 or later

### `dgsi search` — Date range --since + --until
```bash
lawyerr dgsi search "usucapião" --court stj --since 2024-01-01 --until 2025-01-01 --limit 2
```
- [ ] Results only from 2024
- [ ] Fewer results than without date filter

### `dgsi search` — Recent shorthand
```bash
lawyerr dgsi search "usucapião" --court stj --recent 1y --limit 2
```
- [ ] Results from last year only
- [ ] Query contains `AND [DATAAC] > MM/DD/YYYY` with correct date

### `dgsi search` — Sort by date
```bash
lawyerr dgsi search "usucapião" --court stj --limit 3 --sort date
```
- [ ] Results may differ from default relevance sort

### `dgsi search` — Fetch full decisions
```bash
lawyerr dgsi search "usucapião" --court stj --limit 2 --fetch-full
```
- [ ] Returns full decision text (Sumário, Decisão, Texto Integral)
- [ ] Multiple decisions rendered
- [ ] Progress bar shows fetching progress (stderr)

### `dgsi fetch` — Single decision
```bash
lawyerr dgsi fetch "https://www.dgsi.pt/jstj.nsf/954f0ce6ad9dd8b980256b5f003fa814/adbdc4fb2b666586802568fc003a8daf?OpenDocument"
```
- [ ] Full decision rendered with all fields
- [ ] Sumário, Decisão, Votação, Meio Processual present
- [ ] Portuguese characters correct in decision text

### `dgsi fetch` — JSON format
```bash
lawyerr --format json dgsi fetch "https://www.dgsi.pt/jstj.nsf/954f0ce6ad9dd8b980256b5f003fa814/adbdc4fb2b666586802568fc003a8daf?OpenDocument"
```
- [ ] Valid JSON with all decision fields

---

## CLI — Global Flags

### `--output` — Write to file
```bash
lawyerr --output /tmp/results.json --format json dgsi search "usucapião" --court stj --limit 2
cat /tmp/results.json | jq .
```
- [ ] File created with valid JSON
- [ ] No output to stdout

### `--output` — Auto-detect format from extension
```bash
lawyerr --output /tmp/results.json dgsi search "usucapião" --court stj --limit 2
cat /tmp/results.json | jq .
```
- [ ] `.json` extension → JSON format auto-detected

### `--no-compact` — Disable compact mode
```bash
lawyerr --no-compact dgsi search "usucapião" --court stj --limit 2
```
- [ ] Output may have more whitespace/formatting than default

### `--strip-stopwords` — Remove Portuguese stop words
```bash
lawyerr --strip-stopwords dgsi search "usucapião" --court stj --limit 2
```
- [ ] Articles (o, a, os, as) removed from descriptors
- [ ] Legal-critical words preserved (não, sem, nunca)

### `--quiet` — Suppress progress bars
```bash
lawyerr --quiet dgsi search "usucapião" --court stj --limit 2 2>/dev/null | wc -l
```
- [ ] No progress output on stderr
- [ ] Results still printed to stdout

### `--proxy` — Proxy support
```bash
lawyerr --proxy socks5://host:port dgsi search "usucapião" --court stj --limit 1
```
- [ ] Works through proxy (if proxy available)
- [ ] Fails gracefully if proxy unreachable

### `--config` — Custom config file
```bash
echo '[http]\ntimeout_secs = 5' > /tmp/lawyerr_test.toml
lawyerr --config /tmp/lawyerr_test.toml dgsi search "usucapião" --court stj --limit 1
```
- [ ] Config loaded (check logs with `RUST_LOG=debug`)

---

## CLI — DR Module (stubs)

### `dr search` — Not implemented
```bash
lawyerr dr search "portaria"
```
- [ ] Prints "not fully implemented yet" info message
- [ ] Exits cleanly (no panic)

### `dr today` — Not implemented
```bash
lawyerr dr today
```
- [ ] Prints info message, exits cleanly

### `dr types` — Not implemented
```bash
lawyerr dr types
```
- [ ] Prints info message, exits cleanly

### `dr fetch` — Not implemented
```bash
lawyerr dr fetch "https://example.com"
```
- [ ] Prints info message, exits cleanly

---

## HTTP Server

### Start server
```bash
lawyerr serve --port 9876
```
- [ ] Prints `Listening on http://0.0.0.0:9876`
- [ ] Ctrl-C stops gracefully

### `GET /health`
```bash
curl http://localhost:9876/health
```
- [ ] Returns `{"status":"ok","version":"0.1.0"}`
- [ ] Status 200

### `GET /dgsi/courts`
```bash
curl http://localhost:9876/dgsi/courts
```
- [ ] Returns JSON array of 10 courts
- [ ] Each has `alias` and `name` fields

### `GET /dgsi/search` — JSON (default)
```bash
curl "http://localhost:9876/dgsi/search?q=usucapi%C3%A3o&court=stj&limit=3"
```
- [ ] Returns JSON with source, query, total, results
- [ ] Content-Type: application/json
- [ ] Portuguese characters correct

### `GET /dgsi/search` — Markdown
```bash
curl "http://localhost:9876/dgsi/search?q=usucapi%C3%A3o&court=stj&limit=2&format=md"
```
- [ ] Returns markdown output
- [ ] Content-Type: text/markdown; charset=utf-8

### `GET /dgsi/search` — With dates
```bash
curl "http://localhost:9876/dgsi/search?q=usucapi%C3%A3o&court=stj&since=2024-01-01&until=2025-01-01&limit=2"
```
- [ ] Filtered results from date range

### `GET /dgsi/search` — Sort by date
```bash
curl "http://localhost:9876/dgsi/search?q=usucapi%C3%A3o&court=stj&limit=3&sort=date"
```
- [ ] Results sorted by date

### `GET /dgsi/search` — All courts (no court param)
```bash
curl "http://localhost:9876/dgsi/search?q=usucapi%C3%A3o&limit=1"
```
- [ ] Searches all courts
- [ ] Returns combined results

### `GET /dgsi/search` — Missing query
```bash
curl -s -o /dev/null -w "%{http_code}" "http://localhost:9876/dgsi/search"
```
- [ ] Returns 400

### `GET /dgsi/search` — Invalid date
```bash
curl "http://localhost:9876/dgsi/search?q=test&since=not-a-date"
```
- [ ] Returns error with message about invalid date

### `GET /dgsi/fetch` — Single decision
```bash
curl "http://localhost:9876/dgsi/fetch?url=https://www.dgsi.pt/jstj.nsf/954f0ce6ad9dd8b980256b5f003fa814/adbdc4fb2b666586802568fc003a8daf%3FOpenDocument"
```
- [ ] Returns full decision as JSON

### `GET /dgsi/fetch` — Markdown format
```bash
curl "http://localhost:9876/dgsi/fetch?url=https://www.dgsi.pt/jstj.nsf/954f0ce6ad9dd8b980256b5f003fa814/adbdc4fb2b666586802568fc003a8daf%3FOpenDocument&format=md"
```
- [ ] Returns markdown decision

### `GET /dgsi/fetch` — Missing URL
```bash
curl -s -o /dev/null -w "%{http_code}" "http://localhost:9876/dgsi/fetch"
```
- [ ] Returns 400

### DR endpoints — 501
```bash
curl -s -o /dev/null -w "%{http_code}" http://localhost:9876/dr/search
curl -s -o /dev/null -w "%{http_code}" http://localhost:9876/dr/today
curl -s -o /dev/null -w "%{http_code}" http://localhost:9876/dr/types
curl -s -o /dev/null -w "%{http_code}" http://localhost:9876/dr/fetch
```
- [ ] All return 501

---

## Edge Cases

### Empty search results
```bash
lawyerr dgsi search "xyznonexistentterm12345" --court stj --limit 5
```
- [ ] Returns 0 results without error
- [ ] Proper "0 results" message

### Very long query
```bash
lawyerr dgsi search "contrato de trabalho a termo certo com duração determinada" --court stj --limit 2
```
- [ ] Query URL-encoded correctly
- [ ] Returns results

### Pagination
```bash
lawyerr dgsi search "contrato" --court stj --limit 60
```
- [ ] Returns up to 60 results (requires 2 pages of 50+10)
- [ ] Results are deduplicated

### Special characters in query
```bash
lawyerr dgsi search "artigo 1292º" --court stj --limit 2
```
- [ ] Handles `º` character
- [ ] Returns results

### Concurrent piping
```bash
lawyerr --quiet --format json dgsi search "contrato" --court stj --limit 5 | jq '.results | length'
```
- [ ] Clean pipe output with no progress bar interference
- [ ] `jq` parses successfully

---

## Results from 2026-03-21 testing

| Test | Status | Notes |
|---|---|---|
| dgsi courts | ✅ | 10 courts, correct Portuguese chars |
| dgsi search markdown | ✅ | Clean output, correct encoding |
| dgsi search JSON | ✅ | Valid JSON, all fields present |
| dgsi search table | ✅ | Aligned columns, truncation works |
| dgsi search all courts | ✅ | 10 courts parallel, results from all |
| dgsi search multi-court | ✅ | Both courts return results |
| dgsi search --since | ✅ | Date filter works |
| dgsi search --since --until | ✅ | Date range filter works |
| dgsi search --recent 1y | ✅ | Recent shorthand works |
| dgsi search --sort date | ✅ | Sort parameter passed |
| dgsi search --fetch-full | ✅ | Full decisions fetched |
| dgsi fetch | ✅ | Full decision with all fields |
| --output file | ✅ | File created with content |
| --strip-stopwords | ✅ | Stop words removed, legal words kept |
| --no-compact | ✅ | Works |
| --quiet | ✅ | No stderr output |
| Server /health | ✅ | {"status":"ok","version":"0.1.0"} |
| Server /dgsi/courts | ✅ | JSON array, 10 courts |
| Server /dgsi/search JSON | ✅ | Valid JSON results |
| Server /dgsi/search md | ✅ | Markdown output |
| Server /dgsi/fetch | ✅ | Full decision |
| Server DR endpoints | ✅ | All return 501 |
| DR CLI stubs | ✅ | All exit cleanly |
