# Code Report

## Scope

This report reviews the current repository as Rust production code, not as a demo.
The standard here is simple:

- no fake features
- no decorative abstractions
- no “successful” behavior that hides failure
- no typed-looking API that is actually string soup underneath

I also ran:

- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

Both passed. That is useful, but it does not mean the code is clean. Right now the main problems are semantic, architectural, and honesty-of-surface problems rather than compiler-visible problems.

## Executive Summary

The codebase is not a disaster, but it has a very obvious AI-generated smell:

- it presents more capability than it actually has
- it duplicates orchestration instead of extracting it
- it uses typed Rust at the edges while keeping stringly, weakly validated logic in the middle
- it suppresses dead code instead of deleting it
- it treats some failures as success because that keeps the flow moving

The most embarrassing parts from a Rust developer perspective are not ownership mistakes or unsafe code. They are worse:

- fake-complete DR support
- `200 OK` responses when upstream work fully failed
- config surfaces that are mostly decorative
- duplicated parsing and output logic that is already drifting

## Findings Table

| Area | What is slop code | What could be simplified | What would make a Rust developer cringe | Severity | References |
|---|---|---|---|---|---|
| DR feature surface | The repo advertises DR support in README, CLI, and server, but implementation is a stub that returns an empty vec or log line. This is fake product surface. | Hide or remove DR commands until implemented, or return explicit `NotImplemented` errors everywhere. | Shipping commands that pretend to exist is worse than omitting them. It breaks trust in the binary. | High | `README.md:9`, `README.md:39`, `README.md:76`, `src/cli.rs:49`, `src/main.rs:282`, `src/dr/mod.rs:1` |
| API lies on total upstream failure | `/dgsi/search` logs per-court failures and still returns `200 OK` with empty results. Tests explicitly lock in this behavior. | Return an error if all courts fail. If some fail, return partial results with failure metadata. | “Warn and continue” is fine for partial failure, not for total failure. Returning success here is semantically dishonest. | High | `src/server/mod.rs:166`, `src/server/mod.rs:173`, `src/server/mod.rs:200`, `tests/server_test.rs:120`, `tests/server_test.rs:215`, `tests/server_test.rs:323` |
| Duplicate orchestration | DGSI search/fetch orchestration is implemented once in CLI and again in server. They already differ in defaults and behavior. | Extract one application service for “run DGSI search/fetch” and reuse it from CLI and HTTP handlers. | Duplication in Rust service code means drift is guaranteed. The compiler cannot save you from copy-pasted policy. | High | `src/main.rs:37`, `src/main.rs:224`, `src/server/mod.rs:129`, `src/server/mod.rs:208` |
| Decorative config | The config model looks rich, but major runtime paths ignore it. `AppState.config` exists and is barely used. CLI also ignores output defaults from config. | Wire config into runtime or delete unused fields until needed. | Carrying typed state that does nothing is textbook scaffolding slop. | High | `src/config.rs:47`, `src/config.rs:84`, `src/config.rs:109`, `src/main.rs:24`, `src/main.rs:74`, `src/server/mod.rs:20`, `src/server/mod.rs:163`, `src/server/mod.rs:167`, `src/server/mod.rs:281` |
| Zero-concurrency footgun | `max_concurrent` can become `0`, which builds a zero-permit semaphore and hangs work forever. | Validate `max_concurrent >= 1` in CLI/server/config boundary and use `NonZeroUsize` if possible. | Deadlock-by-configuration is exactly the kind of thing strong Rust code prevents at the type boundary. | High | `src/main.rs:74`, `src/main.rs:156`, `src/dgsi/mod.rs:88` |
| Stringly typed CLI and server params | `sort`, `format`, field names, and config format are strings with partial or inconsistent handling. | Replace with enums and validated newtypes. Parse once at the boundary. | Strongly typed language, weakly typed core behavior. Very AI-coded smell. | Medium | `src/cli.rs:97`, `src/server/mod.rs:238`, `src/format.rs:18`, `src/main.rs:73`, `src/main.rs:70`, `src/config.rs:85` |
| Explicit config errors are swallowed | Passing a broken config path does not fail fast. The code logs and silently falls back to defaults, and tests bless that. | Make explicit config load failures fatal. Fallback is fine only for implicit discovery. | Silent fallback on explicit user input is hostile debugging ergonomics. | Medium | `src/config.rs:181`, `src/config.rs:195`, `tests/config_test.rs:68`, `tests/config_test.rs:79` |
| Wrong HTTP status for bad input | Invalid query dates become `500 Internal Server Error` instead of `400 Bad Request`. | Distinguish user input errors from server errors in `AppError`. | Blaming the server for client mistakes is sloppy API design. | Medium | `src/server/mod.rs:37`, `src/server/mod.rs:141`, `src/server/mod.rs:150`, `tests/server_test.rs:237` |
| Retry diagnostics lose context | Retry paths replace URLs with placeholders like `"<retryable status>"` and `"<client build>"`. | Preserve the actual request URL in wrapped errors. | If you wrap errors in Rust, preserving context is the job. Throwing it away defeats the abstraction. | Medium | `src/http.rs:54`, `src/http.rs:103`, `src/http.rs:105` |
| Completion-order output | `search_all_courts` returns results in completion order, not input order. This creates nondeterministic output ordering between runs. | Preserve input order or sort results before returning. | Nondeterministic output from identical input is low-grade chaos and makes testing/reporting worse. | Medium | `src/dgsi/mod.rs:79`, `src/dgsi/mod.rs:90`, `src/dgsi/mod.rs:105` |
| Parser duplication | There are multiple hand-rolled HTML stripping and `<br>` parsing helpers across modules. | Centralize HTML text extraction in one small utility module. | Repeated near-identical parsing logic is how subtle divergence and bugs accumulate. | Medium | `src/compact.rs:28`, `src/dgsi/search.rs:187`, `src/dgsi/decision.rs:145`, `src/dgsi/decision.rs:177` |
| Table rendering contract is too loose | `Renderable::table_row()` returns owned header/value vectors per item, and fallback uses JSON object key order from the first item. | Use a typed row schema or per-renderer structs instead of trait-object-plus-fallback magic. | This is dynamic, allocation-heavy, weakly specified infrastructure for a very simple problem. | Medium | `src/format.rs:52`, `src/format.rs:221`, `src/format.rs:245` |
| Dead-code suppression | Multiple core modules had blanket `#![allow(dead_code)]`. That is a cleanup avoidance tactic, not a solution. | Remove allowances and delete or wire unused code. | “Strict clippy” next to blanket dead-code suppression is cosmetic rigor. | Medium | `src/format.rs:1`, `src/http.rs:1`, `src/config.rs:1`, `src/error.rs:1`, `src/dgsi/decision.rs:1`, `src/dgsi/markdown.rs:1` |
| Main is doing too much | `main` owns CLI parsing, config policy, search policy, progress UI, rendering, and command routing in one large function. | Split command handlers into functions or modules. Keep `main` as composition glue only. | A 300-line async `main` with policy embedded everywhere is not idiomatic maintainable Rust. | Medium | `src/main.rs:12` |
| Compact/stopword pipeline is policy soup | Output cleanup mixes compaction, HTML stripping, and stopword removal in a generic formatting layer. | Decide whether cleanup is parsing-time, render-time, or optional post-processing and keep it in one place. | Generic renderers should not quietly rewrite legal text unless the contract is explicit and consistent. | Medium | `src/format.rs:96`, `src/compact.rs:4`, `src/compact.rs:101` |
| README overclaims implementation | Documentation describes DR session init and ElasticSearch parsing that do not exist in the current code. | Rewrite README to match reality now, not intended architecture. | Overclaiming capability is how AI repos become impossible to trust. | Medium | `README.md:9`, `README.md:39`, `README.md:73` |
| Over-commented generated style | Many files use section-banner comments and “Helper” comments around obvious code while still missing stronger domain modeling. | Delete banner noise and invest that effort in better types and boundaries. | The repo reads like generated prose around mediocre structure. | Low | `src/dgsi/decision.rs:8`, `src/dgsi/search.rs:9`, `src/http.rs:9`, `src/server/mod.rs:16` |
| Hand-rolled string parsing where types should exist | Query building uses ad hoc string concatenation, including raw `FIELD {name} contains {value}` fragments. | Introduce a typed query builder and validated fields. | This is fragile and injection-prone domain code dressed as simple string formatting. | Low | `src/dgsi/search.rs:13` |
| Retry backoff math is unchecked | Backoff uses shifting that can become silly or overflow for bad config. | Clamp retries and use checked/saturating math. | Small issue, but robust Rust code does not trust open-ended integers in timing logic. | Low | `src/http.rs:88` |

## Detailed Notes

### 1. The repo is pretending to be more complete than it is

This is the dominant smell.

- The README sells dual-source search.
- The CLI exposes DR subcommands.
- The server exposes DR endpoints.
- The implementation is an empty async stub plus info logs.

That is not “work in progress”; that is a dishonest API surface.

If you want this repo to stop looking AI-generated, the first rule is:

> never expose unfinished capability as if it were productized.

Either:

- remove DR from public commands and docs until it exists, or
- return a hard `NotImplemented` error everywhere, consistently, with no fake successful path

The current state is the worst middle ground.

### 2. The server’s search semantics are wrong

`/dgsi/search` currently treats upstream failure as if it were a valid empty result set.

That decision poisons everything around it:

- API clients cannot distinguish “no results” from “could not search”
- logs become required for correctness
- tests bless the wrong contract
- downstream automation will silently do the wrong thing

A clean contract should be:

- all courts fail: return error
- some courts fail: return partial success with explicit failure list
- zero results with successful upstream calls: return `200 OK` and empty results

Right now the code collapses all three into one bucket too easily.

### 3. The config system looks serious, but much of it is theater

The typed config structs look fine in isolation. The problem is that the runtime mostly ignores them.

Examples:

- `output.format` exists, but CLI format resolution ignores config defaults
- `output.compact` exists, but CLI uses `!cli.no_compact` as the entire policy
- `http.max_concurrent` exists, but server search hard-codes `3`
- `AppState.config` exists, but handlers barely use it

This is a common AI move: build a full-looking config model because it feels architectural, then never thread it through the actual behavior.

Rust developers hate this because it creates a false sense of correctness. The types suggest a coherent system that the runtime does not honor.

### 4. The code uses Rust’s type system less than it should

The project already depends on `clap`, derives enums, and uses typed structs. So the weak spots stand out even more.

Bad examples:

- `sort: String`
- server-side `format: Option<String>`
- query field filters as free-form strings
- `output.format: String` in config instead of `OutputFormat`

This is where Rust should be doing real work:

- `SortOrder`
- `OutputFormat`
- maybe `SearchField`
- maybe `NonZeroUsize` for concurrency

Instead, you get string comparisons and silent fallback behavior.

That is not catastrophic, but it is exactly the kind of code that feels un-Rusty even when it compiles cleanly.

### 5. There is too much duplicate logic across CLI/server/parser code

The CLI and server both:

- parse dates
- determine format
- run DGSI searches
- optionally fetch full decisions
- collect renderables
- render output

This should be one application service layer with two front ends:

- CLI front end
- HTTP front end

Until that exists, drift is inevitable. You can already see it:

- different defaults
- different error semantics
- different config usage

The same duplication exists in HTML helpers:

- generic tag stripping in `compact`
- tag stripping in DGSI search
- tag stripping and `<br>` handling in DGSI decision parsing

That is not fatal yet, but it is classic entropy territory.

### 6. The “strictness” is partly cosmetic

The repo advertises strong clippy posture. That is good.

But the code also had module-wide `dead_code` allowances in core areas. That combination reads like this:

- “please admire the strict lints”
- “also don’t look too closely at unused or drifting code”

This is exactly the kind of mismatch that makes AI-generated code feel polished at a distance and hollow up close.

Rigor is not:

- lots of lints
- lots of banner comments
- lots of config structs

Rigor is:

- smaller honest surface
- sharper type boundaries
- fewer duplicate paths
- no fake success behavior

### 7. Some of the formatting pipeline is too magical

`format::render` does more than render. It can:

- compact text
- strip stopwords
- truncate cells
- mutate JSON string values recursively only at top-level object fields

That is too much policy inside a generic output function.

For legal-text tooling, especially, arbitrary content rewriting must be very explicit. Otherwise you are building a pipeline that can quietly change semantics.

The current code is not necessarily wrong in every case, but it is too implicit.

## What I Would Clean First

### Phase 1: honesty and correctness

1. Remove or hard-disable DR public surface.
2. Fix `/dgsi/search` to fail when all upstream searches fail.
3. Return `400` for invalid user input.
4. Validate `max_concurrent >= 1`.
5. Rewrite README so it only claims what exists today.

### Phase 2: stop the drift

1. Extract DGSI search/fetch orchestration into a shared application layer.
2. Reuse the same output-format parsing everywhere.
3. Wire runtime behavior to config or delete unused config fields.
4. Preserve deterministic court ordering.

### Phase 3: make it actually feel like Rust

1. Replace stringly flags with enums/newtypes.
2. Introduce typed query/filter objects.
3. Remove blanket dead-code suppressions.
4. Consolidate HTML extraction utilities.
5. Shrink `main` into composition only.

## What Should Probably Be Deleted

- DR commands and docs, if implementation is not imminent
- unused config fields that are not wired
- dead-code allowances
- duplicated helper functions for HTML stripping once a shared utility exists
- banner comments that only narrate obvious code structure

Deleting surface area here would improve the repo faster than adding more code.

## Final Verdict

This repo is not shameful because it is messy. It is shameful because it is too eager to look complete.

The code that most clearly reads as AI slop is the code that:

- exposes unimplemented features
- accepts failure and reports success
- creates config and abstractions that are not actually connected
- duplicates flows instead of extracting domain services

The good news is that the fix is not a rewrite. The core is small enough that you can make it respectable quickly by being ruthless:

- cut fake surface
- tighten types
- unify orchestration
- make failure semantics honest

If you do only those four things, the repo will stop smelling like generated demo code and start reading like a real Rust tool.
