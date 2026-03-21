# Phase 2: DGSI Module

**Goal:** Implement the full DGSI search and parsing pipeline. After this phase, `lawyerr dgsi search "usucapião"` should return real results as markdown/JSON.

**Depends on:** Phase 1 (core infrastructure)

**Ref:** See `docs/plans/initial.md` — DGSI sections for HTML structure, encoding, courts table, query syntax.

---

## Checklist

### Courts Registry (`src/dgsi/courts.rs`)
- [ ] Define `Court` enum with all 10 courts:
  ```rust
  enum Court {
      Stj, Sta, Conflitos,
      RelPorto, RelLisboa, RelCoimbra, RelGuimaraes, RelEvora,
      TcaSul, TcaNorte,
  }
  ```
- [ ] Implement `Court::db(&self) -> &str` — returns `"jstj.nsf"`, `"jsta.nsf"`, etc.
- [ ] Implement `Court::view_unid(&self) -> &str` — returns the view UNID hash
- [ ] Implement `Court::alias(&self) -> &str` — returns `"stj"`, `"rel-porto"`, etc.
- [ ] Implement `Court::from_alias(alias: &str) -> Option<Court>` — case-insensitive
- [ ] Implement `Court::display_name(&self) -> &str` — full Portuguese name
- [ ] Implement `Court::all() -> Vec<Court>` — returns all courts
- [ ] Implement `Court::search_url(&self, query: &str, count: u32, start: u32, sort_by_date: bool) -> String`
  - Format: `https://www.dgsi.pt/{db}.nsf/{view_unid}?SearchView&Query={query}&SearchMax=0&Count={count}&Start={start}`
  - If `sort_by_date`: append `&SearchOrder=1`

### Query Builder (`src/dgsi/search.rs` — query part)
- [ ] Implement `build_query(text: &str, since: Option<NaiveDate>, until: Option<NaiveDate>, field: Option<(&str, &str)>) -> String`
  - Base query from `text`
  - If `since`: append ` AND [DATAAC] > MM/DD/YYYY` (Domino date format!)
  - If `until`: append ` AND [DATAAC] < MM/DD/YYYY`
  - If `field`: use `FIELD {name} contains {value}` format
- [ ] Implement `--recent` duration parsing: `6m` → 6 months ago, `1y` → 1 year ago, `2w` → 2 weeks

### Search Results Parser (`src/dgsi/search.rs` — parser part)
- [ ] Parse search results HTML (fetched via `HttpClient::get_latin1`)
- [ ] Extract total count from `<h4>` tag: parse `"N documents found"` or `"N documents returned; M found"`
- [ ] Parse results table rows, extracting per result:
  ```rust
  struct DgsiSearchResult {
      relevance: u8,          // from <img alt="83%">
      date: NaiveDate,        // from MM/DD/YYYY in <font>
      processo: String,       // case number text
      doc_url: String,        // href from <a> tag
      doc_unid: String,       // extracted from URL
      relator: String,        // judge name
      descriptors: Vec<String>, // split by <br>
  }
  ```
- [ ] Handle edge cases: 0 results page, malformed rows
- [ ] Use `scraper` crate with CSS selectors: `table tr[valign='top']`, `img[alt]`, `font a[href]`

### Decision Page Parser (`src/dgsi/decision.rs`)
- [ ] Fetch individual decision page via `HttpClient::get_latin1`
- [ ] Parse the two-column `<table>` with `bgcolor="#71B2CF"` label cells and `bgcolor="#E0F1FF"` value cells
- [ ] Extract into struct:
  ```rust
  struct DgsiDecision {
      processo: String,
      relator: String,
      descritores: Vec<String>,
      data_acordao: Option<NaiveDate>,
      votacao: String,
      meio_processual: String,
      decisao: String,
      sumario: String,
      texto_integral: String,     // full decision text
      legislacao_nacional: String,
      jurisprudencia_nacional: String,
      doutrina: String,
      url: String,
      court: Court,
      // ... other fields as available
  }
  ```
- [ ] Handle different field sets per court (STJ vs STA have different fields)
- [ ] Strip HTML from field values (basic tag removal)

### Markdown Output (`src/dgsi/markdown.rs`)
- [ ] Implement `SearchResult` trait for `DgsiSearchResult` (listing view)
- [ ] Implement `SearchResult` trait for `DgsiDecision` (full view)
- [ ] Markdown format for search listing:
  ```markdown
  ## STJ — 42 results for "usucapião"

  ### 1. Processo 08A3210 (2008-10-14) — Rel. Azevedo Ramos
  **Relevance:** 83%
  **Descritores:** Usucapião, Posse, Boa Fé
  ```
- [ ] Markdown format for full decision:
  ```markdown
  # Processo 08A3210 — STJ
  **Data:** 2008-10-14 | **Relator:** Azevedo Ramos | **Votação:** Unanimidade

  ## Sumário
  [sumário text]

  ## Decisão
  [decisão text]

  ## Texto Integral
  [full text]
  ```
- [ ] Apply compact mode if enabled (via `compact.rs`)

### Parallel Search (`src/dgsi/mod.rs`)
- [ ] Implement `search_all_courts(query, courts, config) -> Vec<(Court, Result<Vec<DgsiSearchResult>>)>`
  - Spawn one `tokio::spawn` per court
  - Use `tokio::sync::Semaphore` to limit concurrency (`--max-concurrent`)
  - Add delay between requests (`--delay-ms`)
  - Collect results, log errors for failed courts, return successes
- [ ] Implement `search_court(court, query, limit, config) -> Result<Vec<DgsiSearchResult>>`
  - Auto-paginate: fetch pages until `limit` reached or no more results
  - Each page: `Count=50`, `Start` incremented
- [ ] Implement `fetch_decision(url, court, config) -> Result<DgsiDecision>`
- [ ] Implement `--fetch-full` mode: for each search result, fetch full decision in parallel

### Wire Up CLI
- [ ] Connect `lawyerr dgsi search` command to search pipeline
- [ ] Connect `lawyerr dgsi fetch` command to decision fetcher
- [ ] Connect `lawyerr dgsi courts` command to list all courts with aliases
- [ ] Progress bars via `indicatif` for multi-court search and `--fetch-full`

### Verification
- [ ] `lawyerr dgsi courts` — lists all 10 courts with aliases
- [ ] `lawyerr dgsi search "usucapião" --court stj --limit 5` — returns 5 results from STJ
- [ ] `lawyerr dgsi search "usucapião" --court stj --format json` — valid JSON output
- [ ] `lawyerr dgsi search "usucapião" --court stj --since 2020-01-01` — date filtering works
- [ ] `lawyerr dgsi search "usucapião"` (no court) — searches all courts in parallel
- [ ] `lawyerr dgsi fetch <url>` — fetches and renders a full decision as markdown
- [ ] `lawyerr dgsi search "usucapião" --court stj --fetch-full --limit 3` — fetches full text for 3 results
- [ ] Verify Portuguese characters render correctly (ã, ç, õ, é, etc.)

---

## Architecture Notes

**Parsing strategy:** Use `scraper` crate with CSS selectors. Don't regex HTML. Parse once, extract into typed structs.

**Latin-1 decoding:** Always use `HttpClient::get_latin1()` for DGSI requests. The `encoding_rs` crate handles this. Decode BEFORE parsing with `scraper`.

**URL construction:** All URLs must use HTTPS. The base is always `https://www.dgsi.pt/`.

**Date format gotcha:** DGSI uses `MM/DD/YYYY` (American format), not `DD/MM/YYYY`. Parse accordingly with `chrono::NaiveDate::parse_from_str(s, "%m/%d/%Y")`.
