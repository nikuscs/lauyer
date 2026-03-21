# Real-World Test Plan

Manual verification checklist for lauyer CLI. Run against live DGSI APIs.

**Last tested:** 2026-03-21

---

## CLI — DGSI

### `dgsi courts`
```bash
lauyer dgsi courts
```
- [ ] Lists all 10 courts with aliases
- [ ] Portuguese characters render correctly (Relação, Guimarães, Évora)

### `dgsi search` — Basic (markdown default)
```bash
lauyer dgsi search "despejo arrendamento" --court stj --limit 3
```
- [ ] Returns markdown with results
- [ ] Portuguese characters correct (ã, ç, õ, é)
- [ ] Shows processo, date, relator, descritores per result
- [ ] Total count shown

### `dgsi search` — All courts (no --court)
```bash
lauyer dgsi search "responsabilidade civil" --limit 1
```
- [ ] Searches all 10 courts in parallel
- [ ] Returns results from multiple courts
- [ ] Failed courts logged but don't block others

### `dgsi search` — Multiple courts
```bash
lauyer dgsi search "contrato trabalho" --court stj --court rel-porto --limit 2
```
- [ ] Both courts return results
- [ ] Each court section has its own header

### `dgsi search` — JSON format
```bash
lauyer --format json dgsi search "direito propriedade" --court rel-lisboa --limit 3
```
- [ ] Valid JSON output (pipe to `jq .` to verify)
- [ ] Contains source, query, total, results array
- [ ] Each result has processo, date, relator, relevance, descriptors, url

### `dgsi search` — Table format
```bash
lauyer --format table dgsi search "insolvência" --court rel-coimbra --limit 3
```
- [ ] Aligned columns with headers (Date, Processo, Relator, Descritores)
- [ ] Separator row between headers and data
- [ ] Long descriptors truncated with `...`

### `dgsi search` — Date filtering with --since
```bash
lauyer dgsi search "recurso de revista" --court stj --since 2024-01-01 --limit 3
```
- [ ] Query contains `AND [DATAAC] > 01/01/2024`
- [ ] Results are from 2024 or later

### `dgsi search` — Date range --since + --until
```bash
lauyer dgsi search "simulação contrato" --court rel-porto --since 2023-01-01 --until 2024-06-01 --limit 3
```
- [ ] Results only from the specified range
- [ ] Fewer results than without date filter

### `dgsi search` — Recent shorthand
```bash
lauyer dgsi search "penhora" --court tca-sul --recent 6m --limit 3
```
- [ ] Results from last 6 months only

### `dgsi search` — Sort by date
```bash
lauyer dgsi search "herança" --court rel-guimaraes --limit 3 --sort date
```
- [ ] Results sorted chronologically (most recent first)

### `dgsi search` — Fetch full decisions
```bash
lauyer dgsi search "abuso de direito" --court stj --limit 2 --fetch-full
```
- [ ] Returns full decision text (Sumário, Decisão, Texto Integral)
- [ ] Multiple decisions rendered
- [ ] Progress bar shows fetching progress (stderr)

### `dgsi search` — Different court types
```bash
# Administrative court
lauyer dgsi search "acto administrativo" --court sta --limit 2

# Conflicts court
lauyer dgsi search "competência" --court conflitos --limit 2

# Northern administrative court
lauyer dgsi search "impugnação" --court tca-norte --limit 2

# Évora appeals court
lauyer dgsi search "divórcio" --court rel-evora --limit 2
```
- [ ] Each court returns results
- [ ] Court-specific fields may vary (STA vs STJ)

### `dgsi fetch` — Single decision (STJ)
```bash
lauyer dgsi fetch "https://www.dgsi.pt/jstj.nsf/954f0ce6ad9dd8b980256b5f003fa814/adbdc4fb2b666586802568fc003a8daf?OpenDocument"
```
- [ ] Full decision rendered with all fields
- [ ] Sumário, Decisão, Votação, Meio Processual present
- [ ] Portuguese characters correct in decision text

### `dgsi fetch` — JSON format
```bash
lauyer --format json dgsi fetch "https://www.dgsi.pt/jstj.nsf/954f0ce6ad9dd8b980256b5f003fa814/adbdc4fb2b666586802568fc003a8daf?OpenDocument"
```
- [ ] Valid JSON with all decision fields

---

## CLI — Global Flags

### `--output` — Write to file
```bash
lauyer --output /tmp/resultados.json --format json dgsi search "dano moral" --court rel-lisboa --limit 3
cat /tmp/resultados.json | jq .
```
- [ ] File created with valid JSON
- [ ] No output to stdout

### `--output` — Auto-detect format from extension
```bash
lauyer --output /tmp/jurisprudencia.json dgsi search "negligência médica" --court stj --limit 2
cat /tmp/jurisprudencia.json | jq .
```
- [ ] `.json` extension → JSON format auto-detected

### `--no-compact` — Disable compact mode
```bash
lauyer --no-compact dgsi search "servidão predial" --court rel-coimbra --limit 2
```
- [ ] Output may have more whitespace/formatting than default

### `--strip-stopwords` — Remove Portuguese stop words
```bash
lauyer --strip-stopwords dgsi search "posse boa fé" --court stj --limit 2
```
- [ ] Articles (o, a, os, as, de) removed from text
- [ ] Legal-critical words preserved (não, sem, nunca)

### `--quiet` — Suppress progress bars
```bash
lauyer --quiet dgsi search "ónus da prova" --court stj --limit 3 2>/dev/null | wc -l
```
- [ ] No progress output on stderr
- [ ] Results still printed to stdout

### `--quiet` + pipe to jq
```bash
lauyer --quiet --format json dgsi search "hipoteca" --court rel-porto --limit 5 | jq '.results | length'
```
- [ ] Clean pipe output with no progress bar interference
- [ ] `jq` parses successfully

### `--config` — Custom config file
```bash
printf '[http]\ntimeout_secs = 5\n' > /tmp/lauyer_test.toml
RUST_LOG=debug lauyer --config /tmp/lauyer_test.toml dgsi search "fiança" --court stj --limit 1
```
- [ ] Config loaded (debug logs show path)

### `--proxy` — Proxy support
```bash
lauyer --proxy "socks5://127.0.0.1:9999" dgsi search "teste" --court stj --limit 1
```
- [ ] Connection error referencing proxy (confirms proxy is being used)
- [ ] Does not fall back to direct connection

---

## CLI — DGSI (additional params)

### `dgsi search` — `--max-concurrent`
```bash
lauyer dgsi search "contrato" --court stj --limit 3 --fetch-full --max-concurrent 1
```
- [ ] Fetches full decisions sequentially (one at a time)
- [ ] Slower than default concurrency

### `dgsi search` — `--delay-ms`
```bash
lauyer dgsi search "contrato" --court stj --limit 2 --fetch-full --delay-ms 500
```
- [ ] Visible delay between fetches
- [ ] Results still returned correctly

### `dgsi search` — Default limit (50)
```bash
lauyer dgsi search "contrato" --court stj
```
- [ ] Returns up to 50 results (default)
- [ ] Total count may be higher than 50

### `dgsi courts` — JSON format
```bash
lauyer --format json dgsi courts
```
- [ ] Valid JSON array
- [ ] Each entry has `alias` and `name` fields
- [ ] 10 courts listed

---

## CLI — DR (Diário da República)

### `dr types`
```bash
lauyer dr types
```
- [ ] Lists all act type aliases as markdown table
- [ ] Includes portaria, decreto-lei, lei, despacho, etc.

### `dr search` — Portarias (past week)
```bash
lauyer dr search --type portaria --recent 1w
```
- [ ] Returns real Portarias from Diário da República
- [ ] Each result has tipo, número, emissor, sumário
- [ ] Dates within the last week

### `dr search` — Decretos-Lei (past month)
```bash
lauyer dr search --type decreto-lei --recent 1m
```
- [ ] Returns Decretos-Lei
- [ ] Different results from portaria search

### `dr search` — Full-text search
```bash
lauyer dr search "trabalho" --recent 1m
```
- [ ] Text search across all act types
- [ ] Returns results containing "trabalho" in content

### `dr search` — JSON output
```bash
lauyer --format json dr search --type portaria --recent 1w --limit 3
```
- [ ] Valid JSON (pipe to `jq .`)
- [ ] Each result has tipo, numero, emissor, sumario, data_publicacao, serie

### `dr search` — Table output
```bash
lauyer --format table dr search --type lei --recent 6m --limit 5
```
- [ ] Aligned table with Date, Tipo, Número, Emissor columns

### `dr search` — Date range
```bash
lauyer dr search --type portaria --since 2026-03-01 --until 2026-03-15
```
- [ ] Results only from the specified date range

### `dr search` — Multiple act types
```bash
lauyer dr search --type portaria --type decreto-lei --recent 2w
```
- [ ] Returns both Portarias and Decretos-Lei

### `dr search` — Content type: Atos 2ª Série
```bash
lauyer dr search --content atos-2 --type despacho --recent 1w
```
- [ ] Returns Despachos from 2nd series
- [ ] Different content from atos-1

### `dr search` — Content type: Decisões Judiciais
```bash
lauyer dr search --content decisoes --recent 1m --limit 3
```
- [ ] Returns judicial decisions published in DR

### `dr today`
```bash
lauyer dr today
```
- [ ] Returns today's publications (may be empty on weekends)
- [ ] Session init visible in logs

### `dr today` — With type filter
```bash
lauyer dr today --type portaria
```
- [ ] Only Portarias from today

### `dr search` — With --quiet and pipe
```bash
lauyer --quiet --format json dr search --type portaria --recent 1w --limit 3 | jq '.results | length'
```
- [ ] Clean JSON output, no progress bars
- [ ] `jq` parses successfully

### `dr search` — Output to file
```bash
lauyer --output /tmp/dr_results.json dr search --type portaria --recent 1w --limit 5
cat /tmp/dr_results.json | jq .total
```
- [ ] File created with valid JSON

### `dr search` — Strip stopwords
```bash
lauyer --strip-stopwords dr search "regulamento" --type portaria --recent 1m --limit 2
```
- [ ] Stop words removed from sumário text

### `dr search` — Multiple content types
```bash
lauyer dr search --content atos-1 --content atos-2 --recent 1w --limit 5
```
- [ ] Returns results from both 1st and 2nd series
- [ ] Mixed act types in output

### `dr search` — With --limit
```bash
lauyer dr search --type portaria --recent 1m --limit 3
```
- [ ] Returns exactly 3 results (or fewer if less available)

### `dr types` — JSON format
```bash
lauyer --format json dr types
```
- [ ] Valid JSON array
- [ ] Each entry has `alias` and `name` fields
- [ ] 10 act types listed

### `dr fetch` — Not implemented
```bash
lauyer dr fetch "https://example.com"
```
- [ ] Returns error (not implemented yet)

---

## Error Handling

### `--recent` + `--since` mutual exclusion
```bash
lauyer dgsi search "teste" --court stj --recent 1m --since 2024-01-01
```
- [ ] Returns error: mutually exclusive
- [ ] Non-zero exit code

### Invalid `--court` alias
```bash
lauyer dgsi search "teste" --court invalid-court
```
- [ ] Returns error about unknown court alias
- [ ] Non-zero exit code

### Invalid `--type` alias
```bash
lauyer dr search --type invalid-type --recent 1w
```
- [ ] Returns error about unknown act type
- [ ] Non-zero exit code

### Invalid date format
```bash
lauyer dgsi search "teste" --court stj --since "not-a-date"
```
- [ ] Returns error about invalid date
- [ ] Non-zero exit code

### Invalid `--config` path
```bash
lauyer --config /tmp/nonexistent.toml dgsi courts
```
- [ ] Returns error: config file not found
- [ ] Non-zero exit code

---

## Edge Cases

### Empty search results
```bash
lauyer dgsi search "xyztermoqueNaoExiste999" --court stj --limit 5
```
- [ ] Returns 0 results without error
- [ ] Proper "0 results" message

### Very long query (boolean operators)
```bash
lauyer dgsi search "contrato AND trabalho AND termo AND certo" --court stj --limit 2
```
- [ ] Domino boolean operators work
- [ ] Query URL-encoded correctly

### Proximity search
```bash
lauyer dgsi search "usucapião NEAR posse" --court stj --limit 2
```
- [ ] Domino proximity operator works
- [ ] Returns results

### Special characters in query
```bash
lauyer dgsi search "artigo 1292º do Código Civil" --court stj --limit 2
```
- [ ] Handles `º` and accented characters
- [ ] Returns results

### Pagination (more than one page)
```bash
lauyer dgsi search "contrato" --court rel-lisboa --limit 60
```
- [ ] Returns up to 60 results (requires 2 pages of 50+10)
- [ ] No duplicate results

### Field search (DESCRITORES)
```bash
lauyer dgsi search "" --court stj --field DESCRITORES --value "usucapião" --limit 3
```
- [ ] Uses `FIELD DESCRITORES contains usucapião`
- [ ] Returns results with matching descriptors

### Concurrent all-courts with fetch-full
```bash
lauyer dgsi search "mandato" --limit 1 --fetch-full
```
- [ ] Searches all courts + fetches full text for each result
- [ ] Progress bars for both stages
- [ ] Full decision text in output

### Multiple format outputs for same query
```bash
QUERY="locação financeira"
lauyer dgsi search "$QUERY" --court stj --limit 2
lauyer --format json dgsi search "$QUERY" --court stj --limit 2 | jq .total
lauyer --format table dgsi search "$QUERY" --court stj --limit 2
```
- [ ] All three formats produce consistent data
- [ ] Same total count across formats

---

## Results from 2026-03-21 testing

| Test | Status | Notes |
|---|---|---|
| dgsi courts | ✅ | 10 courts, correct Portuguese chars |
| dgsi search markdown (despejo) | ✅ | Clean output, correct encoding |
| dgsi search all courts (responsabilidade) | ✅ | 10 courts parallel, results from all |
| dgsi search multi-court (contrato trabalho) | ✅ | Both STJ+rel-porto return results |
| dgsi search JSON (direito propriedade) | ✅ | Valid JSON, all fields |
| dgsi search table (insolvência) | ✅ | Aligned columns, truncation works |
| dgsi search --since (recurso de revista) | ✅ | Date filter works |
| dgsi search --since --until (simulação) | ✅ | Date range filter works |
| dgsi search --recent 6m (penhora) | ✅ | Recent shorthand works |
| dgsi search --sort date (herança) | ✅ | Sort parameter passed |
| dgsi search --fetch-full (abuso de direito) | ✅ | Full decisions fetched |
| dgsi search STA (acto administrativo) | ✅ | Administrative court works |
| dgsi search conflitos (competência) | ✅ | Conflicts court works |
| dgsi search rel-evora (divórcio) | ✅ | Évora court works |
| dgsi fetch STJ decision | ✅ | Full decision with all fields |
| dgsi fetch JSON | ✅ | Valid JSON decision |
| --output file (dano moral) | ✅ | File created with content |
| --output auto-detect (negligência) | ✅ | .json → JSON format |
| --strip-stopwords (posse boa fé) | ✅ | Stop words removed, legal words kept |
| --no-compact (servidão) | ✅ | Works |
| --quiet (ónus da prova) | ✅ | No stderr output |
| --quiet pipe to jq (hipoteca) | ✅ | Clean pipe, jq parses |
| Empty results | ✅ | 0 results, no error |
| Boolean operators (AND) | ✅ | Domino boolean works |
| Special chars (artigo 1292º) | ✅ | Handles º correctly |
| Pagination 60 results | ✅ | 2-page fetch works |
| dr types | ✅ | 10 act types with aliases |
| dr search portaria 1w | ✅ | 25 real Portarias returned |
| dr search decreto-lei 1m | ✅ | Decretos-Lei returned |
| dr search full-text "trabalho" | ✅ | Text search returns results |
| dr search JSON output | ✅ | Valid JSON, jq parses (25 results) |
| dr search table output | ✅ | Aligned table with Date/Tipo/Número/Emissor |
| dr today | ✅ | Returns 0 on weekend (expected) |
| dr search --quiet pipe jq | ✅ | Clean pipe, jq parses |
| dr fetch | ✅ | Returns error with exit 1 (not implemented) |
