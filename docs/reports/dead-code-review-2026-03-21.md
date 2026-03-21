# Dead Code And Leftovers Review

Date: 2026-03-21

Scope: full repository review of `src/`, `tests/`, top-level project files, and `docs/`.

Method:
- Parallel code review across source, tests, and docs/project files.
- Local validation with `cargo check --lib --bins`, `cargo test`, `cargo fmt -- --check`, and targeted `rg`/line inspection.

## Findings

### High

1. `src/config.rs:67-80` exposes `output.strip_stopwords`, but production code never uses it.
- The CLI only reads `cli.strip_stopwords` in `src/main.rs:24-25`.
- Server rendering hard-codes `false` instead of consulting config in `src/server/mod.rs:226`, `src/server/mod.rs:245`, `src/server/mod.rs:357`, and `src/server/mod.rs:408`.
- Result: this is dead config surface. Users can set `strip_stopwords` in TOML, tests cover it, but it has no runtime effect.

2. `src/server/mod.rs:284-289` and `src/server/mod.rs:362-367` create fresh DR `HttpClient`s and bypass shared app state.
- `AppState.http_client` is used for DGSI, but DR handlers rebuild a client with fixed `None, 30, 3`.
- Result: configured proxy/retry behavior from app state is ignored for DR requests. This looks like leftover duplication from an earlier transport path.

### Medium

3. `src/dgsi/mod.rs:160-186` contains a leftover abstraction used only by tests.
- `SearchParams` is not used by production call paths.
- Its `fetch_full` field at `src/dgsi/mod.rs:166` is never read, even inside `execute_search`.
- Result: dead wrapper surface and one clearly dead field.

4. `src/server/mod.rs:433-439` still ships a permanent `/dr/fetch` stub.
- The route is wired in `src/server/mod.rs:484`.
- It always returns `501 Not Implemented` with a placeholder-style message.
- Result: explicit scaffolding is still exposed in the public server API.

5. `src/config.rs:10-14` keeps empty `DgsiConfig` and `DrConfig` structs in the public config model.
- They are loaded into `Config` at `src/config.rs:112-116`.
- They contain no fields and do not affect runtime behavior.
- Result: placeholder configuration sections remain in the API without functionality behind them.

6. `src/dr/search.rs:47-60` and `src/dr/search.rs:413-417` parse and retain aggregations that are not used anywhere meaningful.
- `DrSearchResponse.aggregations` is populated.
- Nothing in the server or CLI renders or returns aggregations.
- Result: dead payload processing and maintenance overhead for currently unused data.

7. `src/main.rs:24-35` only partially honors `OutputConfig`.
- `cfg.output.format` is used.
- `cfg.output.compact` is ignored by the CLI, which always derives compact mode from `--no-compact`.
- Result: not dead code in the strict sense, but a stale/incomplete config path.

### Low

8. `CLAUDE.md:70-74` still references deleted documentation assets.
- It points to `docs/plans/initial.md`, `docs/plans/phase1-core.md`, and `docs/dr_request_template.json`.
- The current `docs/` tree only contains `docs/real-world-test.md` before this report was added.
- Result: stale documentation references and dead guidance.

## Test Suite Leftovers

### Duplicate or Near-Duplicate Coverage

9. `tests/server_test.rs` contains several overlapping endpoint tests that look like leftover scaffolding.
- `dr_types_returns_json` at `tests/server_test.rs:55` overlaps `dr_types_json` at `tests/server_test.rs:593` and `dr_types_json_structure` at `tests/server_test.rs:753`.
- `dr_fetch_returns_501` at `tests/server_test.rs:107` overlaps `dr_fetch_still_501` at `tests/server_test.rs:650`.
- `dgsi_courts_returns_json_array` at `tests/server_test.rs:31` overlaps `dgsi_courts_response_structure` at `tests/server_test.rs:199`.
- `dr_today_with_type` at `tests/server_test.rs:574` overlaps `dr_today_with_type_filter` at `tests/server_test.rs:733`.

10. `tests/dgsi_mod_test.rs` duplicates behavior already covered in `tests/dgsi_test.rs`.
- `resolve_courts_empty_returns_all` at `tests/dgsi_mod_test.rs:213` overlaps `resolve_courts_empty` at `tests/dgsi_test.rs:198`.
- `resolve_courts_invalid_alias` at `tests/dgsi_mod_test.rs:231` overlaps `resolve_courts_unknown` at `tests/dgsi_test.rs:204`.
- `list_courts_returns_all` at `tests/dgsi_mod_test.rs:244` overlaps `court_list_all` at `tests/dgsi_test.rs:189`.

11. `tests/dr_test.rs` has multiple duplicate coverage clusters.
- `content_type_from_alias` at `tests/dr_test.rs:18` overlaps `test_from_alias_all_variants` at `tests/dr_test.rs:968`.
- `resolve_act_type_aliases` at `tests/dr_test.rs:57` overlaps `test_resolve_act_type_unknown_returns_none` at `tests/dr_test.rs:942` and `test_resolve_act_type_case_insensitive` at `tests/dr_test.rs:950`.
- `dr_search_result_html_stripped_in_markdown` at `tests/dr_test.rs:183` overlaps `markdown_strips_html_completely` at `tests/dr_test.rs:1140`.
- `dr_search_result_table_row` at `tests/dr_test.rs:166` overlaps `table_row_returns_correct_headers_and_values` at `tests/dr_test.rs:1165`.

12. Ignored live-network tests remain in the main suite despite fixture and wiremock coverage.
- DGSI: `tests/dgsi_test.rs:690-723`
- DR: `tests/dr_test.rs:210-219`, `tests/dr_test.rs:1423-1443`
- Result: not broken, but they are leftover manual/integration checks mixed into deterministic test files.

## Not Findings

- I did not find obviously unused fixture files under `tests/fixtures/`.
- The current tree passes `cargo test`.
- `cargo fmt -- --check` currently fails because of formatting drift in existing tests, but that is formatting debt, not dead code.

## Recommended Cleanup Order

1. Remove or wire up `output.strip_stopwords` so config matches runtime behavior.
2. Unify DR handlers onto shared `AppState.http_client` and config-driven transport settings.
3. Delete or internalize `dgsi::SearchParams` and its unused `fetch_full` field.
4. Decide whether `/dr/fetch` should be implemented or removed from the public router.
5. Remove empty config sections or add real fields behind them.
6. Trim duplicate tests and move ignored live-network checks into a separate manual/integration path.
7. Fix stale references in `CLAUDE.md`.

## Verification

- `cargo check --lib --bins`: passed
- `cargo test`: passed
- `cargo fmt -- --check`: failed due to pre-existing formatting drift in `tests/dr_test.rs` and `tests/server_test.rs`
