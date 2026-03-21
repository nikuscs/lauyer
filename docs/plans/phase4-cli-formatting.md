# Phase 4: CLI Polish & Output Formatting

**Goal:** Polish the CLI experience — progress bars, all output formats, `--recent` shorthand, `--output` file writing, `--quiet` mode. After this phase, the CLI should feel production-ready for local use.

**Depends on:** Phase 2 (DGSI) and Phase 3 (DR) — at least one module working

---

## Checklist

### Progress Bars (`indicatif`)
- [ ] Add progress bars for multi-court DGSI search:
  - Multi-progress bar (`MultiProgress`)
  - One spinner per court: `[stj] Searching... ✓ 42 results`
  - Failed courts show: `[rel-evora] ✗ timeout`
- [ ] Add progress bar for `--fetch-full` mode:
  - Per-court progress bar showing `[3/42]` fetched decisions
- [ ] Add spinner for DR search (single request but may be slow)
- [ ] Respect `--quiet` flag: suppress all progress output when set
- [ ] Progress output goes to stderr (so stdout can be piped)

### Output Formats
- [ ] **Markdown** (default) — already implemented in phase 2/3, verify it works end-to-end
- [ ] **JSON** — structured output:
  ```json
  {
    "source": "dgsi",
    "query": "usucapião",
    "total": 42,
    "results": [{ ... }]
  }
  ```
  - For DGSI: include all `DgsiSearchResult` / `DgsiDecision` fields
  - For DR: include all `DrSearchResult` fields
  - Use `serde_json::to_string_pretty()` for human-readable, `to_string()` for piping
- [ ] **Table** — aligned columns for terminal:
  - DGSI: `| Date | Processo | Relator | Descritores |`
  - DR: `| Date | Tipo | Número | Emissor | Sumário (truncated) |`
  - Use simple string formatting with padding (no heavy table crate needed)
  - Truncate long fields to terminal width

### Output Destination
- [ ] `--output <path>` — write to file instead of stdout
- [ ] Auto-detect format from extension if `--format` not specified:
  - `.json` → JSON
  - `.md` → Markdown
  - `.csv` → Table/CSV (bonus)
- [ ] Default (no `--output`): write to stdout

### Recent Shorthand
- [ ] Parse `--recent` values: `1w`, `2w`, `1m`, `3m`, `6m`, `1y`, `2y`
- [ ] Convert to `--since` date: subtract from today
- [ ] `--recent` and `--since` are mutually exclusive (error if both specified)

### Compact & Strip Stopwords Integration
- [ ] Apply `compact_text()` to all text output when `--no-compact` is NOT set
- [ ] Apply `strip_stopwords()` when `--strip-stopwords` IS set
- [ ] For DR: strip HTML tags from `sumario` before applying compact/stopwords
- [ ] For DGSI: strip HTML from decision text fields before applying

### Combined Search (bonus)
- [ ] Consider: `lawyerr search "trabalho"` (no subcommand) → searches BOTH DGSI and DR in parallel
- [ ] Merge results by date, label source
- [ ] This is optional/future — don't block on it

### Verification
- [ ] `lawyerr dgsi search "contrato" --court stj --format json | jq .` — valid JSON, parseable by jq
- [ ] `lawyerr dgsi search "contrato" --court stj --format table` — aligned table output
- [ ] `lawyerr dr search --type portaria --recent 1w --output portarias.md` — writes file
- [ ] `lawyerr dgsi search "contrato" --quiet 2>/dev/null` — only results, no progress
- [ ] `lawyerr dgsi search "contrato" --court stj --recent 1y` — works as shorthand
- [ ] `lawyerr dgsi search "contrato" --court stj --strip-stopwords` — shorter output
- [ ] Pipe test: `lawyerr dgsi search "contrato" --court stj --quiet | wc -l` — works cleanly

**Quality gate:** `cargo fmt --check && cargo clippy -- -D warnings && cargo test` must pass before this phase is complete.
