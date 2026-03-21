# lawyerr — Initial Plan

## Overview

Rust CLI tool for searching and extracting Portuguese legal information from two sources:
1. **DGSI** (www.dgsi.pt) — Court jurisprudence (case law, decisions)
2. **Diário da República** (diariodarepublica.pt) — Official gazette (legislation, portarias, decretos-lei)

Primary goal: fast, parallel search with clean markdown output optimized for LLM consumption. Includes an HTTP server mode for remote access (Unraid, VPS, etc.).

---

# Module 1: DGSI (Jurisprudência)

## Target Platform

- IBM Lotus Domino/Notes (.nsf databases)
- No anti-scraping measures, simple table-based HTML
- Encoding: Latin-1 (ISO-8859-1) — must decode to UTF-8
- 10+ court databases, each independent

## Courts Supported

| Court | DB | View UNID |
|---|---|---|
| Supremo Tribunal de Justiça (STJ) | `jstj.nsf` | `954f0ce6ad9dd8b980256b5f003fa814` |
| Supremo Tribunal Administrativo (STA) | `jsta.nsf` | `35fbbbf22e1bb1e680256f8e003ea931` |
| Tribunal de Conflitos | `jcon.nsf` | `35fbbbf22e1bb1e680256f8e003ea931` |
| Tribunal da Relação do Porto | `jtrp.nsf` | `56a6e7121657f91e80257cda00381fdf` |
| Tribunal da Relação de Lisboa | `jtrl.nsf` | `33182fc732316039802565fa00497eec` |
| Tribunal da Relação de Coimbra | `jtrc.nsf` | `8fe0e606d8f56b22802576c0005637dc` |
| Tribunal da Relação de Guimarães | `jtrg.nsf` | `86c25a698e4e7cb7802579ec004d3832` |
| Tribunal da Relação de Évora | `jtre.nsf` | `134973db04f39bf2802579bf005f080b` |
| Tribunal Central Administrativo Sul | `jtca.nsf` | `170589492546a7fb802575c3004c6d7d` |
| Tribunal Central Administrativo Norte | `jtcn.nsf` | `89d1c0288c2dd49c802575c8003279c7` |

## Court Aliases

| Alias | Court |
|---|---|
| `stj` | Supremo Tribunal de Justiça |
| `sta` | Supremo Tribunal Administrativo |
| `conflitos` | Tribunal de Conflitos |
| `rel-porto` | Tribunal da Relação do Porto |
| `rel-lisboa` | Tribunal da Relação de Lisboa |
| `rel-coimbra` | Tribunal da Relação de Coimbra |
| `rel-guimaraes` | Tribunal da Relação de Guimarães |
| `rel-evora` | Tribunal da Relação de Évora |
| `tca-sul` | Tribunal Central Administrativo Sul |
| `tca-norte` | Tribunal Central Administrativo Norte |

## Search Strategy

**GET-based SearchView** (preferred over POST):

```
https://www.dgsi.pt/{db}.nsf/{viewUNID}?SearchView&Query={query}&SearchMax={max}&Count={perPage}&Start={offset}
```

### Query Syntax (Domino FT Search)

- Free text: `contrato trabalho` (implicit OR)
- Boolean: `contrato AND trabalho`, `contrato NOT trabalho`
- Proximity: `NEAR`, `SENTENCE`, `PARAGRAPH`
- Wildcard: `*` (truncation)
- Field search: `FIELD DESCRITORES contains trabalho`
- Date filter: `[DATAAC] > 01/01/2025`
- Max 1000 results per search; paginated via `Start`/`Count`

### POST Alternative (for reference)

POST to `/{db}.nsf/{formUNID}?CreateDocument` with form-encoded body:
```
termo1=usucapião&operador2=AND&termo2=&operador3=AND&termo4=
```
Note: HTTP→HTTPS redirect converts POST to GET, losing body. Must POST directly to HTTPS.

## DGSI Concrete Request Example

```http
GET https://www.dgsi.pt/jstj.nsf/954f0ce6ad9dd8b980256b5f003fa814?SearchView&Query=usucapi%E3o+AND+posse&SearchMax=0&SearchOrder=1&Count=25&Start=1
```

- `jstj.nsf` = STJ database
- `954f0ce6ad9dd8b980256b5f003fa814` = STJ view UNID
- `Query` = URL-encoded search (note: try both Latin-1 `%E3` and UTF-8 `%C3%A3` for `ã`)
- `SearchMax=0` = return all matches (up to 1000)
- `SearchOrder=1` = sort by date descending (omit for relevance)
- `Count=25` = 25 results per page
- `Start=1` = first result (1-based, use `Start=26` for page 2, etc.)

Response: HTML page with results table. Decode from Latin-1 to UTF-8 before parsing.

## DGSI HTML Structure

### Search Results Page

```html
<h4>{N} documents found</h4>
<table>
  <tr>
    <th></th>          <!-- relevance icon -->
    <th>SESSAO</th>    <!-- date -->
    <th>PROCESSO</th>  <!-- case number + link -->
    <th>RELATOR</th>   <!-- judge -->
    <th>DESCRITOR</th> <!-- keywords -->
  </tr>
  <tr valign="top">
    <td><img alt="83%"></td>                    <!-- relevance in alt text -->
    <td><font size="2">MM/DD/YYYY</font></td>
    <td><font size="2"><a href="/{db}.nsf/{viewUNID}/{docUNID}?OpenDocument">CASE#</a></font></td>
    <td><font size="2">JUDGE NAME</font></td>
    <td><font size="2">DESC1<br>DESC2</font></td>
  </tr>
</table>
```

### Individual Decision Page

Two-column `<table>` with label/value rows:

```html
<table width="100%" border="0">
  <tr>
    <td bgcolor="#71B2CF"><font color="#FFFFFF">Label:</font></td>
    <td bgcolor="#E0F1FF"><font color="#000080">Value</font></td>
  </tr>
</table>
```

**Common fields:** Processo, Relator, Descritores, Data do Acordao, Votacao, Meio Processual, Decisao, Sumario, Decisao Texto Integral, Legislacao Nacional, Jurisprudencia Nacional, Doutrina

**Note:** Fields vary by court (STJ vs STA have different field sets).

## DGSI Date Filter

```
--since 2024-01-01  →  appends " AND [DATAAC] > 01/01/2024" to query
--until 2025-01-01  →  appends " AND [DATAAC] < 01/01/2025" to query
```

Date format for Domino: `MM/DD/YYYY`

## DGSI Encoding

- HTML served in **Latin-1 (ISO-8859-1)** — decode with `encoding_rs`
- **Query parameter encoding**: needs verification — may need Latin-1 or UTF-8. Will test both.
- Internal storage: always UTF-8 after decoding

---

# Module 2: Diário da República (DR)

## Target Platform

- **OutSystems React SPA** with ElasticSearch backend
- Search via POST to screen service endpoints (not simple HTML forms)
- Requires session initialization: GET page → extract CSRF token from `nr2Users` cookie → set search params cookie → POST screen services
- Date format: `YYYY-MM-DD`

## Content Types

Based on practical legal relevance (per Mariana's guidance):

### Keep (relevant)

| Boolean Field | Label | Content Type ID |
|---|---|---|
| `DiarioRepublica` | Diário da República | `f24881a6-9ce7-447b-8d56-898cce7e0e37` |
| `Atos1` | Atos da 1.ª Série | `4032105f-6c93-462a-a43b-4a7e5a75dc67` |
| `Atos2` | Atos da 2.ª Série | `d2cbb9fb-c583-44c7-b8ae-2f8c2cac486e` |
| `Jurisprudencia` | Decisões Judiciais | `cfe0a1d9-fc0a-48fa-aa81-e490d0bac769` |

### Skip (not useful)

| Boolean Field | Label | Reason |
|---|---|---|
| `AcordaosSTA` | Acórdãos do Supremo Tribunal Administrativo | Already covered by DGSI module |
| `AtosSocietarios` | Atos Societários | Corporate acts, discontinued 2006 |
| `Legacor` | Jornal Oficial dos Açores | Regional, niche |
| `DGODOUT` | Direção-Geral do Orçamento | Budget circulars |
| `DGAP` | DGAEP | Public admin employment |
| `REGTRAB` | Regulamentação do Trabalho | Collective labor instruments |

## DR Search Flow — Concrete Examples

### Step 1: Session Initialization (VALIDATED — this is the actual working flow)

```
1. GET https://diariodarepublica.pt/dr/moduleservices/moduleversioninfo
   → {"versionToken": "AZD1bB4VsZW75XkS1nz5tg"}  (changes on deploy)

2. GET https://diariodarepublica.pt/dr/moduleservices/roles
   Headers: X-CSRFToken: T6C+9iB49TLra4jEsMeSckDMNhQ=
   → {"rolesInfo": ","}
   → Sets cookies: nr1Users (HttpOnly, session), nr2Users (CSRF)
```

The CSRF token `T6C+9iB49TLra4jEsMeSckDMNhQ=` is hardcoded in OutSystems.js — use it for the initial roles request.

### Step 2: Build Cookie Filtros (for base64 encoding)

This is the **COOKIE** format — plain arrays, compact JSON, only non-empty fields:

```json
{"tipoConteudo":["AtosSerie1"],"serie":["I"],"dataPublicacaoDe":"2026-03-14","dataPublicacaoAte":"2026-03-21","tipo":["Portaria"],"emissor":[],"entidadeProponente":[],"entidadePrincipal":[],"entidadeEmitente":[],"DescritorList":[]}
```

**IMPORTANT:** Use compact JSON (`separators=(',',':')`) before base64-encoding. Spaces in JSON = different base64 = different search results.

### Step 3: Build Body FiltrosDePesquisa (for POST body)

This is the **BODY** format — OutSystems typed lists, ALL fields present:

```json
{
  "tipoConteudo": {"List": ["AtosSerie1"]},
  "serie": {"List": ["I"]},
  "numero": "",
  "ano": "0", "suplemento": "0",
  "dataPublicacao": "",
  "dataPublicacaoDe": "2026-03-14",
  "dataPublicacaoAte": "2026-03-21",
  "parte": "", "apendice": "", "fasciculo": "",
  "tipo": {"List": ["Portaria"], "EmptyListItem": ""},
  "emissor": {"List": [], "EmptyListItem": ""},
  "texto": "",
  "sumario": "",
  "entidadeProponente": {"List": [], "EmptyListItem": ""},
  "numeroDR": "",
  "paginaInicial": "0", "paginaFinal": "0",
  "dataAssinatura": "", "dataDistribuicao": "",
  "entidadePrincipal": {"List": [], "EmptyListItem": ""},
  "entidadeEmitente": {"List": [], "EmptyListItem": ""},
  "docType": "", "proferido": "", "processo": "", "assunto": "",
  "recorrente": "", "recorrido": "", "relator": "",
  "empresa": "", "concelho": "", "nif": "", "anuncio": "", "numeroDoc": "",
  "DataAssinaturaDe": "1900-01-01", "DataAssinaturaAte": "1900-01-01",
  "DataDistribuicaoDe": "1900-01-01", "DataDistribuicaoAte": "1900-01-01",
  "semestre": "",
  "IsLegConsolidadaSelected": false, "IsFromData": false,
  "DescritorList": {"List": [], "EmptyListItem": ""}
}
```

**Key differences from cookie format:** Lists become `{"List": [...], "EmptyListItem": ""}`. Numbers become strings `"0"`. ALL fields must be present.

### Step 3: Build TipoConteudosBools

```json
{
  "DiarioRepublica": false,
  "Atos1": true,
  "Atos2": false,
  "AcordaosSTA": false,
  "AtosSocietarios": false,
  "Legacor": false,
  "DGODOUT": false,
  "DGAP": false,
  "REGTRAB": false,
  "Jurisprudencia": false
}
```

Set `true` for whichever content types are selected. Groups are exclusive: selecting `DiarioRepublica` alone is Group A; selecting any of `Atos1`/`Atos2`/`AcordaosSTA`/`AtosSocietarios` is Group B; etc.

### Step 4: Build SortFields

```json
[
  {"field": "dataPublicacao", "order": "desc"},
  {"field": "numeroDR.keyword", "order": "desc"},
  {"field": "serieNR", "order": "asc"},
  {"field": "suplemento", "order": "asc"},
  {"field": "apendice.keyword", "order": "asc"}
]
```

### Step 5: Assemble PesquisaAvancada Cookie

```
1. base64_encode(json(PesquisaAvancadaRec)) → "eyJ0aXBvQ29udGV1ZG8iOlsi..."

2. Build the wrapper:
   {
     "PesquisaAvancadaFiltros": "eyJ0aXBvQ29udGV1ZG8iOlsi...",
     "PesquisaAvancadaBools": "{\"DiarioRepublica\":false,\"Atos1\":true,...}",
     "SortFields": "[{\"field\":\"dataPublicacao\",\"order\":\"desc\"},...]"
   }
   Note: PesquisaAvancadaBools and SortFields are JSON-as-string (double-encoded).

3. URL-encode the whole thing and set as cookie:
   Cookie: PesquisaAvancada=%7B%22PesquisaAvancadaFiltros%22%3A%22eyJ0aXBv...

4. Set sort cookie:
   Cookie: sort=data_Desc
```

### Step 6: Execute Search (VALIDATED)

```http
POST https://diariodarepublica.pt/dr/screenservices/dr/Pesquisas/PesquisaResultado/DataActionGetPesquisas
Headers:
  Content-Type: application/json; charset=UTF-8
  X-CSRFToken: T6C+9iB49TLra4jEsMeSckDMNhQ=
  Accept: application/json
  outsystems-locale: pt-PT
Cookie: (set via cookie jar from Step 1 + PesquisaAvancada + sort=8 + ComesFrom=PA)
Body: ~30KB JSON (see docs/dr_request_template.json)
```

The body MUST include:
- `versionInfo.moduleVersion` (from Step 1) + `versionInfo.apiVersion` (`6Bnghy+TVcnOZSN2FpzXbQ`)
- `viewName: "Pesquisas.PesquisaResultado"` (exact string, NOT `"*"`)
- `screenData.variables` with ALL ~80+ variables including empty ES templates
- `clientVariables` with app URLs and session UUID

**An empty body `{}` does NOT work.** The full screen state template is required. See `docs/dr_request_template.json`.

### Step 7: Parse Response (VALIDATED)

Response wraps ElasticSearch JSON as a **string** inside `data.Resultado`:
```json
{
  "versionInfo": {"hasModuleVersionChanged": false, "hasApiVersionChanged": false},
  "data": {
    "Resultado": "{\"took\":20,\"hits\":{\"total\":{\"value\":20},\"hits\":[...]},\"aggregations\":{...}}",
    "ResultsCount": "20",
    "HasErrorPesquisa": false,
    "DestaquesExcertos2": false,
    "Data_DeValid": true,
    "Data_AteValid": true
  }
}
```

**Parse `data.Resultado` as JSON string** → then access `hits.hits[]._source` for results.

If `versionInfo.hasApiVersionChanged: true` and `data` is empty, the apiVersion hash is stale. Fetch the latest from `dr.Pesquisas.PesquisaResultado.mvc.js` in the module manifest.

### DR Session Refresh

If the CSRF token expires (screen service returns 403 or role validation error):
1. Re-do Step 1 (GET pesquisa-avancada) to get fresh cookies
2. Extract new CSRF token
3. Retry the search

## DR Act Types (Tipo de Ato)

Common types relevant for legal work:
- **Portaria** — Ministerial order
- **Decreto-Lei** — Decree-law
- **Lei** — Law
- **Resolução do Conselho de Ministros** — Council of Ministers resolution
- **Despacho** — Administrative decision/order
- **Decreto** — Decree
- **Aviso** — Notice
- **Declaração de Retificação** — Rectification declaration
- **Decreto Regulamentar** — Regulatory decree
- **Lei Orgânica** — Organic law

## DR Sort Options

| Key | Description |
|---|---|
| `data_Desc` | Date descending (default) |
| `data_Asc` | Date ascending |
| `frequencia` | Relevance (default when text search used) |
| `ato_Desc` / `ato_Asc` | By act type |
| `emissor_Desc` / `emissor_Asc` | By issuer |

## DR Key Endpoints

```
# Dropdown data (act types, issuers, etc.)
POST /dr/screenservices/dr/Pesquisas/PesquisaAvancada/DataActionGetListsForDropdown

# Main search
POST /dr/screenservices/dr/Pesquisas/PesquisaResultado/DataActionGetPesquisas

# Pagination/refresh
POST /dr/screenservices/dr/Pesquisas/PesquisaResultado/DataActionRefreshPesquisas

# PDF export
POST /dr/screenservices/dr/Pesquisas/PesquisaResultado/ActionExportarPesquisaResultadoPDF
```

## DR Implementation Approach

**Pure HTTP confirmed working** — no headless browser needed. OutSystems screen services work via POST with proper session init, typed body, and cookies.

### Validated Session Flow (tested, working)

```
Step 1: GET /dr/moduleservices/moduleversioninfo → {"versionToken": "..."}
Step 2: GET /dr/moduleservices/roles (with X-CSRFToken header) → sets nr1Users/nr2Users cookies
Step 3: Set PesquisaAvancada cookie with base64-encoded search params
Step 4: POST /dr/screenservices/dr/Pesquisas/PesquisaResultado/DataActionGetPesquisas
        → Returns ElasticSearch JSON with results
```

### Key Constants

- **AnonymousCSRFToken** (hardcoded): `T6C+9iB49TLra4jEsMeSckDMNhQ=`
- **API version for DataActionGetPesquisas**: `6Bnghy+TVcnOZSN2FpzXbQ` (from MVC JS, may change on deploy)
- **API version for ActionSetPAEstatisticaInfo**: `SNCHCQfuIfS5q4uny9AzEg`

### Critical Implementation Details

1. **viewName MUST be `"Pesquisas.PesquisaResultado"`** — `"*"` causes "No role validation found"
2. **Body must include full screen state** — OutSystems requires typed `screenData.variables` with ALL ~80+ variables, including empty ElasticSearch result templates. The body is ~30KB.
3. **Lists use OutSystems format**: `{"List": [...], "EmptyListItem": ""}` not plain arrays
4. **Numbers are strings**: `"0"` not `0` (for ES-related fields)
5. **JSON for base64 must be compact**: use `separators=(',',':')` — no spaces
6. **Required headers**: `outsystems-locale: pt-PT`, `X-CSRFToken`, `Content-Type: application/json; charset=UTF-8`
7. **PesquisaAvancada cookie** contains base64-encoded search filters — the server reads from BOTH the cookie AND the body
8. **Search text goes in `texto` field** for full-text search on individual acts. The `numero` field is for DR issue number search when using DiarioRepublica content type.
9. **Content type values are PascalCase**: `"AtosSerie1"`, `"AtosSerie2"`, `"DiarioRepublica"`, `"Jurisprudencia"`. NOT camelCase (`"atosSerie1"`) or short form (`"Atos1"`) — those return wrong results.

### Response Format

The response `data.Resultado` is a **JSON string** (not an object) containing raw ElasticSearch results:

```json
{
  "data": {
    "Resultado": "{\"took\":20,\"hits\":{\"total\":{\"value\":33},\"hits\":[{\"_source\":{...}}]}}",
    "ResultsCount": "33",
    "HasErrorPesquisa": false
  }
}
```

Each hit's `_source` fields (confirmed via testing):
- `title` — Full title (e.g., "Diário da República n.º 56/2026, Suplemento, Série I")
- `dataPublicacao` — Publication date (YYYY-MM-DD)
- `numero` — DR number or act number
- `serie` — Series ("I" or "II")
- `serieNR` — Series number (1 or 2)
- `suplemento` — Supplement number
- `tipo` / `type` — Act type (e.g., "Portaria", "Decreto-Lei")
- `emissor` — Issuer
- `sumario` — Summary (for individual acts)
- `dbId` — Database ID (for fetching full document)
- `fileId` — File ID
- `tipoConteudo` — Content type ("DiarioRepublica", "atosSerie1", etc.)
- `ano` — Year
- `className` — Entity class (DiarioRepublica vs Ato)
- `docType` — Document type ("LEGISLACAO")
- `numPaginas`, `paginaInicial`, `paginaFinal` — Page info
- `tamanhoFicheiro` — File size

### Two Result Levels

1. **DR Issues** — returned when `tipoConteudo: ["DiarioRepublica"]` and `PesquisaAvancadaBools: {"DiarioRepublica": true}`. Returns whole Diário numbers (className: DiarioRepublica).
2. **Individual Acts** — returned when searching with `Atos1: true` or specific content type. Returns Portarias, Decretos-Lei, etc. with `tipo`, `emissor`, `sumario` fields populated.

**CRITICAL — tipoConteudo values (case-sensitive, tested and confirmed):**

| tipoConteudo value | Returns | type field |
|---|---|---|
| `"DiarioRepublica"` | Whole DR issues (PDFs) | `DiarioRepublica` |
| `"AtosSerie1"` (PascalCase!) | **Individual acts (Portarias, Decretos-Lei, etc.)** | `DiplomaLegis` |
| `"AtosSerie2"` (PascalCase!) | **Individual acts from 2nd series** (despachos, avisos, anúncios) | `DiplomaLegis` |
| `"Jurisprudencia"` | **Judicial decisions** (Acórdãos TC, STA published in DR) | `DiplomaLegis` |
| `"atosSerie1"` (camelCase) | Returns DR issues (WRONG!) | `DiarioRepublica` |
| `"Atos1"` | Returns DR issues (WRONG!) | `DiarioRepublica` |

**Aggregation buckets** in the response provide available filters for act types (`TipoAtoAgg`) and issuers (`EmissorAgg`). Use these to populate dropdown options in the CLI.

**Search fields:**
- `texto` — Full text search across document content (confirmed working)
- `numero` — DR number search (for DiarioRepublica content type)
- `tipo` — Act type filter, but does **partial matching** ("Lei" matches "Decreto-Lei" too). Use exact values from TipoAtoAgg buckets.
- `emissor` — Issuer filter (exact match from EmissorAgg)
- `serie` — `["I"]` for 1st series, `["II"]` for 2nd, `[]` for all

**Use `"AtosSerie1"` to get individual acts.** The camelCase `"atosSerie1"` from the static entity IDs does NOT work — it returns DR issues instead.

For Portaria filtering, set `tipo: ["Portaria"]` in both the cookie filtros AND the body FiltrosDePesquisa.

`ActionSetPAEstatisticaInfo` is analytics only — not needed for search results.

### Empty Screen State Template

The implementing agent must build a ~30KB body with ALL screen variables. The template is a fixed structure — only `FiltrosDePesquisa`, `PesquisaAvancadaFiltros`, `PesquisaAvancadaBools`, `DataDe`/`DataAte`, and cookie-related fields change per search. Everything else is static empty defaults.

A complete reference body is saved at `docs/dr_request_template.json` for the implementing agent to use as a starting point.

---

# Shared Architecture

## Project Structure

```
src/
├── main.rs
├── cli.rs                  # clap derive commands (subcommands per module)
├── config.rs               # lawyerr.toml + env + CLI layering
├── compact.rs              # Structural boilerplate removal (on by default)
├── format.rs               # Output: Markdown (default), JSON, Table
├── dgsi/
│   ├── mod.rs
│   ├── client.rs           # reqwest client, Latin-1 decoding
│   ├── courts.rs           # Court enum, aliases, db/view mappings
│   ├── search.rs           # Query builder + search results HTML parser
│   ├── decision.rs         # Individual decision page parser
│   └── markdown.rs         # Decision → clean markdown
├── dr/
│   ├── mod.rs
│   ├── client.rs           # Session init, CSRF, cookie management
│   ├── content_types.rs    # Content type enum, UUIDs, booleans
│   ├── search.rs           # Search params builder, ES results parser
│   ├── document.rs         # Individual document parser
│   └── markdown.rs         # Document → clean markdown
└── server/
    ├── mod.rs              # Axum router, graceful shutdown
    ├── routes.rs           # HTTP handlers (all endpoints)
    └── state.rs            # AppState (shared clients, config)
```

## CLI Commands

```bash
# ============ DGSI (Jurisprudência) ============

# Search all courts in parallel (default when no --court specified)
lawyerr dgsi search "usucapião"

# Search specific court(s)
lawyerr dgsi search "contrato trabalho" --court stj --court rel-porto

# Date filtering — recent decisions
lawyerr dgsi search "usucapião" --court stj --since 2024-01-01
lawyerr dgsi search "usucapião" --court stj --recent 6m

# Limit and sort
lawyerr dgsi search "usucapião" --limit 20 --sort date

# Field-specific search
lawyerr dgsi search --field DESCRITORES --value "contrato" --court stj

# Fetch a specific decision → markdown
lawyerr dgsi fetch <url-or-doc-id>

# List available courts
lawyerr dgsi courts

# ============ Diário da República ============

# Search DR for portarias
lawyerr dr search "portaria" --type portaria --since 2024-01-01

# Search for recent decreto-leis
lawyerr dr search "arrendamento" --type decreto-lei --recent 1y

# Search by act type only (no text)
lawyerr dr search --type portaria --since 2025-01-01

# Search in specific content types
lawyerr dr search "trabalho" --content atos-1

# Fetch today's publications
lawyerr dr today
lawyerr dr today --type portaria

# Fetch a specific DR document
lawyerr dr fetch <url-or-doc-id>

# List available act types
lawyerr dr types

# ============ Shared flags ============

# Proxy support (applies to both modules)
lawyerr dgsi search "usucapião" --proxy socks5://host:port

# Output format
lawyerr dgsi search "usucapião" --format json
lawyerr dr search "portaria" --format table

# Write output to file
lawyerr dgsi search "usucapião" --output results.md

# Compact mode (on by default, disable with flag)
lawyerr dr search "portaria" --no-compact

# Stop word removal (opt-in)
lawyerr dgsi search "usucapião" --strip-stopwords

# Fetch full text for all search results
lawyerr dgsi search "usucapião" --fetch-full

# Control parallelism
lawyerr dgsi search "usucapião" --max-concurrent 5

# Start HTTP server
lawyerr serve --port 3000 --host 0.0.0.0
```

### Defaults

- No `--court` specified → searches **all courts** in parallel
- `--limit` default: `50` per court/query (safety cap, max `1000`)
- `--sort` default: `relevance`
- `--compact` default: `on`
- `--format` default: `markdown`
- `--max-concurrent` default: `10`
- `--delay-ms` default: `100`
- DR content types default: Atos 1.ª Série + Atos 2.ª Série + Decisões Judiciais

### DR Content Type Aliases

| Alias | Content |
|---|---|
| `dr` | Diário da República (all) |
| `atos-1` | Atos da 1.ª Série |
| `atos-2` | Atos da 2.ª Série |
| `decisoes` | Decisões Judiciais |

### DR Act Type Aliases

| Alias | Act Type |
|---|---|
| `portaria` | Portaria |
| `decreto-lei` | Decreto-Lei |
| `lei` | Lei |
| `resolucao` | Resolução do Conselho de Ministros |
| `despacho` | Despacho |
| `decreto` | Decreto |
| `aviso` | Aviso |
| `retificacao` | Declaração de Retificação |

## Compact Mode (default: on)

Removes structural bloat without altering legal meaning:
- Strips formulaic preambles ("Acordam no Tribunal da Relação de...")
- Removes repeated headers and procedural boilerplate
- Collapses excessive whitespace/empty lines
- Cleans HTML formatting artifacts
- Estimated ~20-30% token reduction

## Stop Word Removal (opt-in: `--strip-stopwords`)

Conservative Portuguese stop word list that preserves legal meaning:

**Removed:** articles (`o`, `a`, `os`, `as`, `um`, `uma`, `uns`, `umas`), prepositions (`de`, `em`, `por`, `para`, `com`), conjunctions (`e`, `ou`, `que`, `mas`)

**Never removed (legal-critical):** `não`, `sem`, `nem`, `nunca`, `nenhum`, `nenhuma`, `jamais`, `salvo`, `excepto`, `apenas`, `somente`

## Config File (`lawyerr.toml`)

```toml
# DGSI defaults
[dgsi]
courts = ["stj", "sta", "rel-porto", "rel-lisboa"]

# DR defaults
[dr]
content_types = ["atos-1", "atos-2", "decisoes"]
# act_types = []  # empty = all types

# HTTP settings (shared)
[http]
proxy = "socks5://host:port"   # optional
delay_ms = 100
max_concurrent = 10
timeout_secs = 30
retries = 3

# Output defaults
[output]
format = "markdown"            # markdown | json | table
compact = true
strip_stopwords = false

# Server mode
[server]
host = "0.0.0.0"
port = 3000
```

Config location search order: `./lawyerr.toml` → `~/.config/lawyerr/lawyerr.toml` → CLI flags override all.

## Dependencies

```toml
[dependencies]
# CLI
clap = { version = "4", features = ["derive", "env"] }

# Async
tokio = { version = "1", features = ["full"] }

# HTTP
reqwest = { version = "0.12", features = ["gzip", "brotli", "socks", "json", "cookies"] }

# HTML parsing
scraper = "0.25"

# HTML → Markdown
htmd = "0.1"

# Encoding
encoding_rs = "0.8"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Base64 (for DR search params)
base64 = "0.22"

# HTTP server
axum = "0.8"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Error handling
anyhow = "1"
thiserror = "2"

# Date handling
chrono = { version = "0.4", features = ["serde"] }

# Progress bars
indicatif = "0.17"

# UUID generation (for DR session)
uuid = { version = "1", features = ["v4"] }
```

## Concurrency Model

- DGSI: each court search runs as an independent `tokio::spawn` task
- DR: single session, sequential searches (OutSystems may not handle parallel well)
- `--max-concurrent` controls semaphore permits (default: 10)
- `--fetch-full`: decision/document fetches run in parallel within results
- Optional delay between requests (`--delay-ms`, default: 100ms)
- Auto-pagination up to `--limit`

## Retry & Resilience

- Exponential backoff on network errors: `2^(attempt-1) * 500ms`, max 3 retries
- Distinguish fatal (404, invalid query) vs retryable (timeout, 503, connection reset)
- Cookie jar via `reqwest::cookie::Jar` — both DGSI and DR need cookies
- Graceful degradation: if one court/source fails, still return results from others
- DR session refresh: if CSRF token expires, re-initialize session automatically

## Progress Feedback

- `indicatif` progress bars during multi-court searches and `--fetch-full` operations
- Show: `[court/source] Searching... ✓ 42 results`
- For `--fetch-full`: progress bar showing fetched/total
- Quiet mode: `--quiet` suppresses progress output (useful for piping)

## HTTP Server Mode (`lawyerr serve`)

Axum-based HTTP server wrapping both modules as REST endpoints.

### Endpoints

```
# DGSI
GET /dgsi/search?q={query}&court={court}&since={date}&until={date}&limit={n}&sort={sort}&format={md|json}&compact={bool}&fetch_full={bool}
GET /dgsi/fetch?url={dgsi_url}&format={md|json}&compact={bool}
GET /dgsi/courts

# Diário da República
GET /dr/search?q={query}&type={act_type}&content={content_type}&since={date}&until={date}&limit={n}&format={md|json}&compact={bool}
GET /dr/fetch?url={dr_url}&format={md|json}&compact={bool}
GET /dr/today?type={act_type}&format={md|json}
GET /dr/types

# Shared
GET /health
```

### Docker / Unraid

```dockerfile
FROM rust:1-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/lawyerr /usr/local/bin/
EXPOSE 3000
CMD ["lawyerr", "serve", "--port", "3000", "--host", "0.0.0.0"]
```

## Output Examples

### DGSI Markdown (compact)

```markdown
## STJ — 3 results for "usucapião"

### 1. Processo 08A3210 (2008-10-14) — Rel. Azevedo Ramos
**Relevance:** 83%
**Descritores:** Usucapião, Posse, Boa Fé

#### Sumário
A usucapião constitui uma forma de aquisição originária...

#### Decisão
Negada revista.

---
```

### DR Markdown (compact)

```markdown
## Diário da República — 5 results

### 1. Portaria n.º 123/2025 (2025-03-15) — Série I
**Emitente:** Ministério da Justiça
**Sumário:** Regulamenta os procedimentos relativos à...

#### Texto
[Full text content here...]

---
```

### JSON

```json
{
  "source": "dgsi",
  "court": "stj",
  "query": "usucapião",
  "total_found": 3,
  "results": [
    {
      "processo": "08A3210",
      "date": "2008-10-14",
      "relator": "Azevedo Ramos",
      "relevance": 83,
      "descriptors": ["Usucapião", "Posse", "Boa Fé"],
      "url": "https://www.dgsi.pt/jstj.nsf/...",
      "sumario": "...",
      "decisao": "..."
    }
  ]
}
```

## Implementation Order

### Phase 1: DGSI Core
1. `dgsi/courts.rs` — Court enum, aliases, db/view mappings
2. `dgsi/client.rs` — HTTP client with Latin-1 decoding, proxy support
3. `dgsi/search.rs` — Query builder + search results HTML parser
4. `dgsi/decision.rs` — Decision page HTML parser
5. `dgsi/markdown.rs` — Decision → markdown converter

### Phase 2: Shared Infrastructure
6. `compact.rs` — Boilerplate removal
7. `format.rs` — JSON/Table output
8. `cli.rs` + `main.rs` — Wire up DGSI commands
9. `config.rs` — TOML config file support

### Phase 3: DR Module
10. `dr/client.rs` — Session init, CSRF, cookie management
11. `dr/content_types.rs` — Content type enum, UUIDs
12. `dr/search.rs` — Search params builder, ES results parser
13. `dr/document.rs` — Document parser
14. `dr/markdown.rs` — Document → markdown converter
15. Wire up DR CLI commands

### Phase 4: Server & Deployment
16. `server/` — Axum HTTP server mode with all endpoints
17. `Dockerfile` — Container support

---

# Validation & Testing Reference

Everything below was tested live on 2026-03-21. Use these as smoke tests during implementation.

## DGSI Smoke Test

```bash
# Simple GET — should return 200 with HTML containing "5 documents returned; 1000 found"
curl -s "https://www.dgsi.pt/jstj.nsf/954f0ce6ad9dd8b980256b5f003fa814?SearchView&Query=usucapiao&SearchMax=0&Count=5&Start=1" | iconv -f ISO-8859-1 -t UTF-8 | head -20
```

**Expected:** HTML table with 5 rows, each containing SESSÃO (date MM/DD/YYYY), PROCESSO (case number link), RELATOR (judge name), DESCRITOR (keywords).

**Encoding confirmed:** Response is Latin-1. Characters like `ã`, `ç`, `õ` appear garbled without `iconv`. Use `encoding_rs` in Rust.

**Pagination confirmed:** `Start=1` returns first page, `Start=6` returns second page of 5.

## DR Smoke Test (Python — can be translated to curl or Rust)

```python
import urllib.request, urllib.parse, json, base64, http.cookiejar, uuid
from http.cookiejar import Cookie

# 1. Init session
cjar = http.cookiejar.CookieJar()
op = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(cjar))
v = json.loads(op.open('https://diariodarepublica.pt/dr/moduleservices/moduleversioninfo').read()).get('versionToken','')
op.open(urllib.request.Request('https://diariodarepublica.pt/dr/moduleservices/roles',
    headers={'X-CSRFToken':'T6C+9iB49TLra4jEsMeSckDMNhQ='}))

# 2. Build cookie (compact JSON, base64)
fc = json.dumps({'tipoConteudo':['AtosSerie1'],'serie':['I'],
    'dataPublicacaoDe':'2026-03-14','dataPublicacaoAte':'2026-03-21',
    'tipo':[],'emissor':[],'entidadeProponente':[],
    'entidadePrincipal':[],'entidadeEmitente':[],'DescritorList':[]},
    separators=(',',':'))
fb64 = base64.b64encode(fc.encode()).decode()
pb = '{"Atos1":true}'
pc = urllib.parse.quote(json.dumps({'PesquisaAvancadaFiltros':fb64,
    'PesquisaAvancadaBools':pb,'SortFields':'[{"Field":"dataPublicacao","Order":"desc"}]'},
    separators=(',',':')))

# 3. Set cookies
for n,val in [('PesquisaAvancada',pc),('sort','8'),('ComesFrom','PA')]:
    cjar.set_cookie(Cookie(0,n,val,None,False,'diariodarepublica.pt',
        False,False,'/',False,True,9999999999,False,None,None,{}))

# 4. Load template, set dynamic fields, POST
# (load docs/dr_request_template.json, set FiltrosDePesquisa etc.)
# 5. Parse: result['data']['Resultado'] is a JSON STRING → json.loads() it
```

**Expected:** `data.Resultado` contains ES JSON with `hits.total.value > 0` and `hits.hits[]._source` containing `tipo`, `numero`, `emissor`, `sumario`, `dataPublicacao`.

## What Was Tested & Confirmed

### DGSI
| Test | Result |
|------|--------|
| GET search for "usucapiao" on STJ | 200 OK, 1000 results found, 5 returned per page |
| HTML structure matches documented format | Confirmed: `<h4>`, `<table>`, `<img alt="94%">`, `<font size="2">` |
| Latin-1 encoding | Confirmed: raw bytes show `Ã` for `ã` without decoding |
| Pagination via `Start` param | Confirmed working |

### DR Session Flow
| Test | Result |
|------|--------|
| `GET moduleversioninfo` | 200, returns `{"versionToken":"..."}` |
| `GET roles` with CSRF header | 200, sets `nr1Users` + `nr2Users` cookies |
| POST with `viewName: "*"` | 200 but `"No role validation found"` — WRONG viewName |
| POST with `viewName: "Pesquisas.PesquisaResultado"` + empty body | 400 `"Failed to parse JSON"` — needs full body |
| POST with `viewName: "Pesquisas.PesquisaResultado"` + full template body | 200 OK with results! |

### DR Content Types (tipoConteudo values)
| Value | Result |
|-------|--------|
| `"AtosSerie1"` (PascalCase) | Individual acts: Portarias, Decretos-Lei, etc. (type=DiplomaLegis) |
| `"AtosSerie2"` (PascalCase) | Individual acts 2nd series: Despachos, Avisos, etc. |
| `"DiarioRepublica"` | Whole DR issues (PDFs, type=DiarioRepublica) |
| `"Jurisprudencia"` | Judicial decisions (Acórdãos TC, STA) |
| `"atosSerie1"` (camelCase) | WRONG — returns DR issues, not individual acts |
| `"Atos1"` (short) | WRONG — returns DR issues |
| `"4032105f-..."` (UUID) | WRONG — returns DR issues |

### DR Search Fields
| Field | In cookie filtros | In body FiltrosDePesquisa | Confirmed |
|-------|------------------|-------------------------|-----------|
| `texto` | plain string | plain string | Full text search works |
| `tipo` | plain array `["Portaria"]` | `{"List":["Portaria"],"EmptyListItem":""}` | Filters by act type (partial match: "Lei" also matches "Decreto-Lei") |
| `dataPublicacaoDe/Ate` | `"YYYY-MM-DD"` | `"YYYY-MM-DD"` | Date filtering works |
| `serie` | `["I"]` or `["II"]` | `{"List":["I"]}` | Series filter works |
| `emissor` | plain array | OutSystems list | Not tested individually but present in aggregations |
| `numero` | plain string | plain string | DR number search (for DiarioRepublica type) |

### DR Aggregation Buckets (from response)
The response includes aggregation data useful for building filter UIs:
- `TipoAtoAgg.buckets` — available act types with counts (e.g., `[{"key":"Portaria","doc_count":20},{"key":"Decreto-Lei","doc_count":3}]`)
- `EmissorAgg.buckets` — available issuers with counts
- `SerieAgg.buckets` — available series
- `EntidadeProponenteAgg.buckets` — proposing entities
- `DescritorAgg.buckets` — descriptors/keywords
- `ParteAgg.buckets` — parts
- `CalendarioAgg.buckets` — calendar dates

### DR Sample Results (Portarias, week of Mar 14-21, 2026)
Confirmed 20 Portarias returned, including:
- Portaria n.º 122/2026/1 — Reconhece a AEA/ACOAG como câmara de comércio
- Portaria n.º 123/2026/1 — Programa Nacional de Promoção da Saúde Oral
- Portaria n.º 123-A/2026/1 — Taxas de imposto sobre produtos petrolíferos
- Portaria n.º 117/2026/1 — Comunicações eletrónicas GNR/PSP/MP/tribunais
- Portaria n.º 116/2026/1 — Mapa de pessoal do MENAC

## How Variables Flow (DR Module)

The same search parameters appear in THREE places — they must be consistent:

```
┌─────────────────────────────────────────────────────────┐
│ 1. PesquisaAvancada COOKIE                              │
│    base64(compact_json({                                │
│      tipoConteudo: ["AtosSerie1"],   ← plain arrays     │
│      serie: ["I"],                                      │
│      tipo: ["Portaria"],                                │
│      dataPublicacaoDe: "2026-03-14",                    │
│      ...                                                │
│    }))                                                  │
├─────────────────────────────────────────────────────────┤
│ 2. POST body → screenData.variables.FiltrosDePesquisa   │
│    {                                                    │
│      tipoConteudo: {"List":["AtosSerie1"]}, ← OS lists  │
│      serie: {"List":["I"]},                             │
│      tipo: {"List":["Portaria"],"EmptyListItem":""},    │
│      dataPublicacaoDe: "2026-03-14",                    │
│      ...                                                │
│    }                                                    │
├─────────────────────────────────────────────────────────┤
│ 3. POST body → screenData.variables.PesquisaAvancada*   │
│    PesquisaAvancadaFiltros: base64 string (same as #1)  │
│    PesquisaAvancadaBools: '{"Atos1":true}'              │
│    GetCookiePesquisas.Pesquisas.Avancada: URL-encoded   │
│      cookie value (same as what's in the cookie)        │
│    GetDecodeURLPesquisaAvancada: JSON wrapper of #1     │
│    DataDe/DataAte: same dates as filtros                │
└─────────────────────────────────────────────────────────┘
```

**The server reads from the cookie AND the body.** Both must contain consistent search parameters. The cookie drives the ElasticSearch query; the body provides screen state for OutSystems framework validation.

## How to Add a New Search Parameter

1. Add the field to the cookie filtros JSON (plain value)
2. Add the same field to body `FiltrosDePesquisa` (OutSystems typed)
3. Re-encode the cookie: `base64(compact_json(filtros))` → update `PesquisaAvancadaFiltros` in body too
4. Update `GetCookiePesquisas.Pesquisas.Avancada` with the new URL-encoded cookie
5. Update `GetDecodeURLPesquisaAvancada.PesquisaAvancada_URL_Decoded` with the new wrapper JSON

## Files Reference

| File | Purpose |
|------|---------|
| `docs/plans/initial.md` | This plan — full architecture, API details, test results |
| `docs/dr_request_template.json` | Complete ~30KB POST body template with ALL screen variables. Fields marked `DYNAMIC` change per search; everything else is static empty defaults. |
