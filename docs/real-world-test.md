# Real-World Test Plan

Manual verification checklist for lawyerr CLI. Run against live DGSI APIs.

**Last tested:** 2026-03-21

---

## CLI — DGSI

### `dgsi courts`
```bash
lawyerr dgsi courts
```
- [ ] Lists all 10 courts with aliases
- [ ] Portuguese characters render correctly (Relação, Guimarães, Évora)

### `dgsi search` — Basic (markdown default)
```bash
lawyerr dgsi search "despejo arrendamento" --court stj --limit 3
```
- [ ] Returns markdown with results
- [ ] Portuguese characters correct (ã, ç, õ, é)
- [ ] Shows processo, date, relator, descritores per result
- [ ] Total count shown

### `dgsi search` — All courts (no --court)
```bash
lawyerr dgsi search "responsabilidade civil" --limit 1
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
lawyerr --format json dgsi search "direito propriedade" --court rel-lisboa --limit 3
```
- [ ] Valid JSON output (pipe to `jq .` to verify)
- [ ] Contains source, query, total, results array
- [ ] Each result has processo, date, relator, relevance, descriptors, url

### `dgsi search` — Table format
```bash
lawyerr --format table dgsi search "insolvência" --court rel-coimbra --limit 3
```
- [ ] Aligned columns with headers (Date, Processo, Relator, Descritores)
- [ ] Separator row between headers and data
- [ ] Long descriptors truncated with `...`

### `dgsi search` — Date filtering with --since
```bash
lawyerr dgsi search "recurso de revista" --court stj --since 2024-01-01 --limit 3
```
- [ ] Query contains `AND [DATAAC] > 01/01/2024`
- [ ] Results are from 2024 or later

### `dgsi search` — Date range --since + --until
```bash
lawyerr dgsi search "simulação contrato" --court rel-porto --since 2023-01-01 --until 2024-06-01 --limit 3
```
- [ ] Results only from the specified range
- [ ] Fewer results than without date filter

### `dgsi search` — Recent shorthand
```bash
lawyerr dgsi search "penhora" --court tca-sul --recent 6m --limit 3
```
- [ ] Results from last 6 months only

### `dgsi search` — Sort by date
```bash
lawyerr dgsi search "herança" --court rel-guimaraes --limit 3 --sort date
```
- [ ] Results sorted chronologically (most recent first)

### `dgsi search` — Fetch full decisions
```bash
lawyerr dgsi search "abuso de direito" --court stj --limit 2 --fetch-full
```
- [ ] Returns full decision text (Sumário, Decisão, Texto Integral)
- [ ] Multiple decisions rendered
- [ ] Progress bar shows fetching progress (stderr)

### `dgsi search` — Different court types
```bash
# Administrative court
lawyerr dgsi search "acto administrativo" --court sta --limit 2

# Conflicts court
lawyerr dgsi search "competência" --court conflitos --limit 2

# Northern administrative court
lawyerr dgsi search "impugnação" --court tca-norte --limit 2

# Évora appeals court
lawyerr dgsi search "divórcio" --court rel-evora --limit 2
```
- [ ] Each court returns results
- [ ] Court-specific fields may vary (STA vs STJ)

### `dgsi fetch` — Single decision (STJ)
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
lawyerr --output /tmp/resultados.json --format json dgsi search "dano moral" --court rel-lisboa --limit 3
cat /tmp/resultados.json | jq .
```
- [ ] File created with valid JSON
- [ ] No output to stdout

### `--output` — Auto-detect format from extension
```bash
lawyerr --output /tmp/jurisprudencia.json dgsi search "negligência médica" --court stj --limit 2
cat /tmp/jurisprudencia.json | jq .
```
- [ ] `.json` extension → JSON format auto-detected

### `--no-compact` — Disable compact mode
```bash
lawyerr --no-compact dgsi search "servidão predial" --court rel-coimbra --limit 2
```
- [ ] Output may have more whitespace/formatting than default

### `--strip-stopwords` — Remove Portuguese stop words
```bash
lawyerr --strip-stopwords dgsi search "posse boa fé" --court stj --limit 2
```
- [ ] Articles (o, a, os, as, de) removed from text
- [ ] Legal-critical words preserved (não, sem, nunca)

### `--quiet` — Suppress progress bars
```bash
lawyerr --quiet dgsi search "ónus da prova" --court stj --limit 3 2>/dev/null | wc -l
```
- [ ] No progress output on stderr
- [ ] Results still printed to stdout

### `--quiet` + pipe to jq
```bash
lawyerr --quiet --format json dgsi search "hipoteca" --court rel-porto --limit 5 | jq '.results | length'
```
- [ ] Clean pipe output with no progress bar interference
- [ ] `jq` parses successfully

### `--config` — Custom config file
```bash
printf '[http]\ntimeout_secs = 5\n' > /tmp/lawyerr_test.toml
RUST_LOG=debug lawyerr --config /tmp/lawyerr_test.toml dgsi search "fiança" --court stj --limit 1
```
- [ ] Config loaded (debug logs show path)

---

## CLI — DR Module (stubs)

### `dr search` — Not implemented
```bash
lawyerr dr search "portaria"
```
- [ ] Prints info message, exits cleanly (no panic)

### `dr today` / `dr types` / `dr fetch`
```bash
lawyerr dr today
lawyerr dr types
lawyerr dr fetch "https://example.com"
```
- [ ] All print info message, exit cleanly

---

## Edge Cases

### Empty search results
```bash
lawyerr dgsi search "xyztermoqueNaoExiste999" --court stj --limit 5
```
- [ ] Returns 0 results without error
- [ ] Proper "0 results" message

### Very long query (boolean operators)
```bash
lawyerr dgsi search "contrato AND trabalho AND termo AND certo" --court stj --limit 2
```
- [ ] Domino boolean operators work
- [ ] Query URL-encoded correctly

### Proximity search
```bash
lawyerr dgsi search "usucapião NEAR posse" --court stj --limit 2
```
- [ ] Domino proximity operator works
- [ ] Returns results

### Special characters in query
```bash
lawyerr dgsi search "artigo 1292º do Código Civil" --court stj --limit 2
```
- [ ] Handles `º` and accented characters
- [ ] Returns results

### Pagination (more than one page)
```bash
lawyerr dgsi search "contrato" --court rel-lisboa --limit 60
```
- [ ] Returns up to 60 results (requires 2 pages of 50+10)
- [ ] No duplicate results

### Field search (DESCRITORES)
```bash
lawyerr dgsi search "" --court stj --field DESCRITORES --value "usucapião" --limit 3
```
- [ ] Uses `FIELD DESCRITORES contains usucapião`
- [ ] Returns results with matching descriptors

### Concurrent all-courts with fetch-full
```bash
lawyerr dgsi search "mandato" --limit 1 --fetch-full
```
- [ ] Searches all courts + fetches full text for each result
- [ ] Progress bars for both stages
- [ ] Full decision text in output

### Multiple format outputs for same query
```bash
QUERY="locação financeira"
lawyerr dgsi search "$QUERY" --court stj --limit 2
lawyerr --format json dgsi search "$QUERY" --court stj --limit 2 | jq .total
lawyerr --format table dgsi search "$QUERY" --court stj --limit 2
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
| DR stubs | ✅ | All exit cleanly |
