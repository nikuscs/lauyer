# Phase 3: DR (Diário da República) Module

**Goal:** Implement the full DR search pipeline via OutSystems screen services. After this phase, `lawyerr dr search --type portaria --recent 1w` should return real Portarias as markdown/JSON.

**Depends on:** Phase 1 (core infrastructure)

**Ref:** See `docs/plans/initial.md` — DR sections for session flow, content types, field mapping, test results. See `docs/dr_request_template.json` for the ~30KB POST body template.

---

## Checklist

### Content Types & Constants (`src/dr/content_types.rs`)
- [ ] Define `DrContentType` enum:
  ```rust
  enum DrContentType {
      AtosSerie1,     // Individual acts, 1st series (Portarias, Decretos-Lei, etc.)
      AtosSerie2,     // Individual acts, 2nd series (Despachos, Avisos, etc.)
      DiarioRepublica, // Whole DR issues (PDFs)
      Jurisprudencia,  // Judicial decisions published in DR
  }
  ```
- [ ] Implement `DrContentType::tipo_conteudo(&self) -> &str` — returns PascalCase value for search:
  - `AtosSerie1` → `"AtosSerie1"` (NOT `"atosSerie1"` or `"Atos1"` — those return wrong results!)
  - `AtosSerie2` → `"AtosSerie2"`
  - `DiarioRepublica` → `"DiarioRepublica"`
  - `Jurisprudencia` → `"Jurisprudencia"`
- [ ] Implement `DrContentType::bools_key(&self) -> &str` — returns key for PesquisaAvancadaBools:
  - `AtosSerie1` → `"Atos1"`
  - `AtosSerie2` → `"Atos2"`
  - `DiarioRepublica` → `"DiarioRepublica"`
  - `Jurisprudencia` → `"Jurisprudencia"`
- [ ] Implement `DrContentType::from_alias(alias: &str) -> Option<Self>` — maps CLI aliases:
  - `"atos-1"` → `AtosSerie1`, `"atos-2"` → `AtosSerie2`, etc.
- [ ] Define `DrActType` — known act type strings for `--type` filter:
  - `"Portaria"`, `"Decreto-Lei"`, `"Lei"`, `"Despacho"`, `"Decreto"`, `"Aviso"`, `"Resolução do Conselho de Ministros"`, `"Declaração de Retificação"`, `"Decreto Regulamentar"`, `"Lei Orgânica"`
- [ ] Implement `DrActType::from_alias(alias: &str) -> Option<String>` — maps CLI-friendly aliases to exact strings

### Session Manager (`src/dr/client.rs`)
- [ ] Define `DrSession` struct:
  ```rust
  struct DrSession {
      http_client: reqwest::Client,  // with cookie jar
      module_version: String,
      csrf_token: String,
      api_version: String,  // "6Bnghy+TVcnOZSN2FpzXbQ"
  }
  ```
- [ ] Implement `DrSession::new(config: &HttpConfig) -> Result<Self>`:
  1. Create reqwest client with cookie jar (`Arc<reqwest::cookie::Jar>`)
  2. `GET /dr/moduleservices/moduleversioninfo` → extract `versionToken`
  3. `GET /dr/moduleservices/roles` with header `X-CSRFToken: T6C+9iB49TLra4jEsMeSckDMNhQ=` → cookies get set
  4. Store CSRF token (hardcoded `T6C+9iB49TLra4jEsMeSckDMNhQ=` for anonymous)
  5. Store API version (hardcoded `6Bnghy+TVcnOZSN2FpzXbQ` — may need refresh logic)
- [ ] Implement `DrSession::refresh(&mut self) -> Result<()>` — re-init if session expires
- [ ] Implement version staleness detection: if response has `hasApiVersionChanged: true`, log warning

### Search Parameters Builder (`src/dr/search.rs` — builder part)
- [ ] Define `DrSearchParams`:
  ```rust
  struct DrSearchParams {
      content_type: DrContentType,
      query: String,           // goes in `texto` field
      act_types: Vec<String>,  // goes in `tipo` field
      since: Option<NaiveDate>,
      until: Option<NaiveDate>,
      series: Vec<String>,     // ["I"], ["II"], or []
      limit: u32,
  }
  ```
- [ ] Implement `build_cookie_filtros(params: &DrSearchParams) -> String`:
  - Build plain JSON with only non-empty fields
  - Use **compact JSON** (`serde_json::to_string` then strip spaces, or use a custom serializer)
  - Base64-encode the JSON string
  - Return the base64 string
- [ ] Implement `build_pesquisa_cookie(params: &DrSearchParams) -> String`:
  - Build wrapper: `{"PesquisaAvancadaFiltros": "<base64>", "PesquisaAvancadaBools": "<json-string>", "SortFields": "<json-string>"}`
  - PesquisaAvancadaBools: `{"Atos1": true}` (only the selected content type is true)
  - SortFields: `[{"Field":"dataPublicacao","Order":"desc"},...]`
  - URL-encode the whole wrapper JSON
- [ ] Implement `build_body_filtros(params: &DrSearchParams) -> serde_json::Value`:
  - Same fields as cookie but in OutSystems format: lists become `{"List": [...], "EmptyListItem": ""}`
  - Numbers become strings `"0"`
  - All fields must be present (empty ones too)

### Request Body Builder (`src/dr/search.rs` — body part)
- [ ] Load/embed the template from `docs/dr_request_template.json` at compile time or runtime
  - **Recommended:** Embed via `include_str!` and parse at startup — avoids file path issues
- [ ] Implement `build_search_body(session: &DrSession, params: &DrSearchParams) -> serde_json::Value`:
  - Start from template
  - Remove `_comment` fields
  - Set `versionInfo.moduleVersion` from session
  - Set `versionInfo.apiVersion` from session
  - Set `screenData.variables.FiltrosDePesquisa` from `build_body_filtros()`
  - Set `screenData.variables.PesquisaAvancadaFiltros` (base64 string)
  - Set `screenData.variables.PesquisaAvancadaBools` (JSON string)
  - Set `screenData.variables.DataDe` and `DataAte` (same as date params)
  - Set `screenData.variables.GetCookiePesquisas.Pesquisas.Avancada` (URL-encoded cookie value)
  - Set `screenData.variables.GetDecodeURLPesquisaAvancada.PesquisaAvancada_URL_Decoded`
  - Set `clientVariables.Data` (today), `Session_GUID` (uuid v4), `DateTime` (ISO 8601 now)
- [ ] **All 3 places must be consistent** (see "How Variables Flow" in initial plan)

### Search Execution & Response Parser (`src/dr/search.rs` — execution part)
- [ ] Implement `search(session: &DrSession, params: &DrSearchParams) -> Result<DrSearchResponse>`:
  1. Build PesquisaAvancada cookie value
  2. Set cookies on the session's cookie jar: `PesquisaAvancada`, `sort=8`, `ComesFrom=PA`
  3. Build request body
  4. POST to `/dr/screenservices/dr/Pesquisas/PesquisaResultado/DataActionGetPesquisas`
  5. Required headers: `Content-Type: application/json; charset=UTF-8`, `X-CSRFToken`, `Accept: application/json`, `outsystems-locale: pt-PT`
  6. Parse response
- [ ] Implement response parsing:
  - Check for `exception` field → return error
  - Check `versionInfo.hasApiVersionChanged` → log warning if true
  - Extract `data.Resultado` — this is a **JSON string**, not an object!
  - `serde_json::from_str(data.Resultado)` → ElasticSearch results
  - Extract `data.ResultsCount` for total
- [ ] Define `DrSearchResult`:
  ```rust
  struct DrSearchResult {
      id: String,
      title: String,
      tipo: String,           // "Portaria", "Decreto-Lei", etc.
      numero: String,         // "123/2026/1"
      data_publicacao: NaiveDate,
      emissor: String,
      sumario: String,        // may contain HTML
      serie: String,
      db_id: String,
      file_id: String,
      tipo_conteudo: String,
      ano: u32,
  }
  ```
- [ ] Parse aggregation buckets for available filters:
  ```rust
  struct DrAggregations {
      tipo_ato: Vec<(String, u64)>,    // from TipoAtoAgg
      emissor: Vec<(String, u64)>,     // from EmissorAgg
      serie: Vec<(String, u64)>,       // from SerieAgg
  }
  ```

### Markdown Output (`src/dr/markdown.rs`)
- [ ] Implement `SearchResult` trait for `DrSearchResult`
- [ ] Markdown format:
  ```markdown
  ## Atos 1.ª Série — 20 results (Mar 14-21, 2026)

  ### 1. Portaria n.º 122/2026/1 (2026-03-20)
  **Emissor:** Economia e Coesão Territorial
  **Sumário:** Reconhece a Associação Empresarial de Águeda...
  ```
- [ ] Strip HTML from `sumario` field (`<p>`, `<a>` tags)
- [ ] Apply compact mode if enabled

### Wire Up CLI
- [ ] Connect `lawyerr dr search` command:
  - `--content` flag → `DrContentType` (default: `atos-1`)
  - `--type` flag → act type filter (repeatable)
  - `--since` / `--until` / `--recent` → date range
  - Query text → `texto` field
- [ ] Connect `lawyerr dr today` command:
  - Sets `since` and `until` to today
  - Optional `--type` filter
- [ ] Connect `lawyerr dr types` command:
  - Runs a search with no filters, extracts `TipoAtoAgg` buckets, displays available types with counts
- [ ] Connect `lawyerr dr fetch` command (if individual document fetching is needed)

### Verification
- [ ] `lawyerr dr search --content atos-1 --recent 1w` — returns individual acts from past week
- [ ] `lawyerr dr search --content atos-1 --type portaria --recent 1w` — only Portarias
- [ ] `lawyerr dr search "trabalho" --content atos-1 --recent 1m` — text search works
- [ ] `lawyerr dr search --content atos-2 --recent 1w` — 2nd series results
- [ ] `lawyerr dr search --content decisoes --recent 1m` — judicial decisions
- [ ] `lawyerr dr today` — today's publications
- [ ] `lawyerr dr types` — lists available act types
- [ ] `lawyerr dr search --format json` — valid JSON output
- [ ] Verify sumário HTML is stripped in markdown output
- [ ] Verify session refresh works (run search, wait, run again)

---

## Architecture Notes

**Template embedding:** Use `include_str!("../../docs/dr_request_template.json")` to embed the template at compile time. Parse it once at `DrSession::new()` and store as `serde_json::Value`. Clone and modify per search.

**Cookie management:** Use `reqwest::cookie::Jar` shared via `Arc`. Set cookies programmatically before each search. The cookie jar handles sending them automatically.

**JSON encoding gotcha:** The base64-encoded filtros in the cookie MUST use compact JSON (no spaces after `:` or `,`). Use `serde_json::to_string()` which produces compact JSON by default in Rust (unlike Python's `json.dumps` which adds spaces).

**Double-encoded strings:** `PesquisaAvancadaBools` and `SortFields` in the cookie wrapper are JSON-as-string (double-encoded). In Rust: `serde_json::to_string(&bools_map)` gives you the inner JSON string, then put that string as a value in the outer JSON.

**Response parsing:** `data.Resultado` is a JSON string, not a nested object. You need to deserialize the outer response first, then `serde_json::from_str(&resultado_string)` for the inner ES results.
