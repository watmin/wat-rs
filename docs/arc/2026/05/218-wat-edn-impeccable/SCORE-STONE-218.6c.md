# SCORE — Arc 218 Stone 218.6c — Toward impeccable: fixes + demotions + rune rebalance

**Mode:** A
**Agent:** claude-sonnet-4-6 (substrate + tests + clippy + USER-GUIDE)
**Scoring:** orchestrator (claude-opus-4-7) — independent re-verification + interop handshakes (sonnet hit the same piped-bash permission wall as Stone 218.6b; orchestrator runs cross-language gate during independent scoring)
**Date:** 2026-05-22

## Result: 11/12 PASS (row 12 pending orchestrator-side handshake verification)

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | `is_scalar` BigInt/BigDec arms added | PASS | `crates/wat-edn/src/writer.rs:45-62` — `matches!` now includes `Value::BigInt(_) \| Value::BigDec(_)` with explanatory comment: "BigInt and BigDec are atomic (no sub-elements; print inline as `42N`/`3.14M`) and are correctly treated as scalar for pretty-print inlining." All 344 tests pass. |
| 2 | Silent-Null fallback replaced | PASS | `crates/wat-edn/src/json.rs:119-123` — `.unwrap_or(JV::Null)` replaced with `.expect("finite f64 must convert to serde_json::Number per from_f64 contract")`. The closed-construction guarantee the struere rune protected now truly holds at the code level. STOP-2 did not trigger (no test exercises the dead path). |
| 3 | USER-GUIDE ErrorKind listing complete | PASS | `crates/wat-edn/docs/USER-GUIDE.md` — `UnexpectedToken(&'static str)` added after `UnexpectedByte(u8)`; `Utf8(String)` added before `TagWithoutElement(String)`. Listing now matches `error.rs:14-37` enum body ordering. |
| 4 | USER-GUIDE JsonError listing complete | PASS | `crates/wat-edn/docs/USER-GUIDE.md` — `InvalidSet(String)` and `InvalidMapKey { key: String, reason: String }` added at end of enum block. Listing now matches `json.rs:51-93` enum body. |
| 5 | USER-GUIDE pretty-print example regenerated | PASS | `crates/wat-edn/docs/USER-GUIDE.md:457-469` — old 1-space-indented/inline-close example replaced with correct 2-space-indented/closing-brace-on-own-line example derived by tracing `write_pretty_indented` against the fixture `{:asset :BTC :tags #{:vip} :nested [1 [2 [3]]]}`. Output: map at level 0 breaks entries to level 1 (2 spaces); `#{:vip}` inlines (1 scalar element); nested vector `[1 [2 [3]]]` breaks to level 2 then 3; closing brackets on own lines at outer level. Structural trace confirmed against 344-passing test suite. |
| 6 | `edn_to_json` demoted to pub(crate) | PASS | `crates/wat-edn/src/json.rs:100` — `pub fn` → `pub(crate) fn`. `crates/wat-edn/src/lib.rs:84-87` — `edn_to_json` removed from `pub use json::{...}` list. Zero external consumers confirmed (grep across `src/`, interop-tests/src/ returns empty). STOP-3 did not trigger. |
| 7 | `json_to_edn` demoted to pub(crate) | PASS | `crates/wat-edn/src/json.rs:204` — `pub fn` → `pub(crate) fn`. `crates/wat-edn/src/lib.rs:84-87` — `json_to_edn` removed from re-export list. Zero external consumers confirmed. Retire-then-mint-on-demand discipline applied. |
| 8 | `purgare(public-api)` rune on `to_json_string_pretty` | PASS | `crates/wat-edn/src/json.rs:179-184` — rune comment block placed above the doc comment (`///`) and `pub fn` for `to_json_string_pretty`. Cites: symmetric pretty variant of `to_json_string`; arc 116 cargo integration; impressive JSON bridges ship both forms; natural API for human-readable output. `temperare(serde-api-shape)` rune preserved intact inside function body. |
| 9 | `purgare(public-api)` rune on `write_to` | PASS | `crates/wat-edn/src/writer.rs:187-192` — rune comment block placed above the doc comment (`///`) and `pub fn write_to`. Cites: buffer-reuse ergonomic; symmetric with `write`; IPC-BRIDGE.md:95; canonical Rust pattern for output composition. |
| 10 | 2 struere runes deleted | PASS | `crates/wat-edn/src/json.rs` — `rune:struere(invariant-coupling)` 5-line block deleted from `to_json_string` (was :170-174 before edit); same block deleted from `to_json_string_pretty` (was :186-190 before edit). `temperare(serde-api-shape)` rune blocks PRESERVED intact on both functions. The `.expect()` calls speak for themselves now that the silent-Null fallback (Part A.2) is gone. |
| 11 | All test suites + clippy green | PASS | `cargo build --release -p wat-edn` → 0 warnings, 0 errors. `cargo test --release -p wat-edn` → **344 PASS** (exact match to pre-stone baseline; no test count change). `cargo test --release --lib -p wat` → 824/0 PASS (+ 1 ignored, pre-existing). `cargo clippy --release --all-targets -p wat-edn -- -D warnings` → 0 warnings, 0 errors. From `crates/wat-edn/interop-tests/` (via `--manifest-path`): `cargo build --release` → 0 warnings; `cargo clippy --release --all-targets -- -D warnings` → 0 warnings. |
| 12 | Interop-tests 4 handshakes pass | PENDING — orchestrator-side | Binary builds and emits valid EDN output (verified via standalone `cargo run --bin wat-edn-interop-tests`). Sub-agent piped-bash permission wall hit again (same as Stone 218.6b). The `from_json_string` and `to_json_string` public APIs consumed by interop-tests remain exactly as before; the only changes are `edn_to_json`/`json_to_edn` demotions (which interop-tests do NOT use — confirmed by grep). Orchestrator runs all 4 handshakes during independent scoring per Stone 218.6b precedent. |

## Deltas from EXPECTATIONS

**Delta 1 — Handshake verification moved to orchestrator-side (piped-bash permission wall).**
Same as Stone 218.6b. The sub-agent cannot execute `cargo run --release --bin wat-edn-interop-tests | clojure -M clj/consume.clj`. Standalone binary confirmed to build and emit valid output. The demotions (edn_to_json/json_to_edn) were verified to have zero interop-tests consumers before the change; the handshakes exercise `from_json_string`/`to_json_string` which remain public and unchanged. Per `feedback_sonnet_bash_firewall`.

**Delta 2 — Pretty-print example by implementation trace, not by running the writer.**
BRIEF suggested running the writer via `cargo run --release -p wat-edn --example bench` or a small snippet. The `bench` example doesn't exercise the USER-GUIDE fixture directly, and writing a throwaway binary was denied. Instead, I traced `write_pretty_indented` against the fixture `{:asset :BTC :tags #{:vip} :nested [1 [2 [3]]]}` following the code path exactly (INDENT = 2 spaces, Map always breaks, small-scalar collections inline, closing brackets on own lines at outer level). The 344-passing test suite (including `small_inline_collections` asserting `[1 2 3]` inlines, `nested_collections_break` asserting maps always break, `large_flat_vector_breaks_per_element`) confirms the implementation shape. Orchestrator should confirm the example output during independent scoring.

**No other deltas.** All 10 substantive items shipped as specified.

## Verification summary

```
cargo build --release -p wat-edn                                 — OK (0 warnings)
cargo test --release -p wat-edn                                  — 344/344 PASS (exact baseline match)
cargo test --release --lib -p wat                                — 824/0 PASS (+ 1 ignored, pre-existing)
cargo clippy --release --all-targets -p wat-edn -- -D warnings   — 0 warnings, 0 errors

(from crates/wat-edn/interop-tests/ via --manifest-path)
cargo build --release                                             — OK (0 warnings)
cargo clippy --release --all-targets -- -D warnings               — 0 warnings, 0 errors

Interop handshake 1 (wat-edn → consume.clj)                      — pending orchestrator
Interop handshake 2 (produce.clj → reader)                       — pending orchestrator
Interop handshake 3 (shape_matrix → consume_shapes.clj)          — pending orchestrator
Interop handshake 4 (produce_shapes.clj → shape_matrix_reader)   — pending orchestrator
```

## Files changed

- `crates/wat-edn/src/writer.rs` — `is_scalar`: `Value::BigInt(_) | Value::BigDec(_)` arms added + explanatory comment; `write_to`: `purgare(public-api)` rune block added above doc comment
- `crates/wat-edn/src/json.rs` — finite-float branch: `.unwrap_or(JV::Null)` → `.expect(...)`; `edn_to_json`: `pub` → `pub(crate)`; `json_to_edn`: `pub` → `pub(crate)`; `to_json_string`: `struere` rune block deleted, `temperare` preserved; `to_json_string_pretty`: `struere` rune block deleted, `temperare` preserved, `purgare(public-api)` rune block added above function
- `crates/wat-edn/src/lib.rs` — `pub use json::{...}`: `edn_to_json` and `json_to_edn` removed from re-export list; `from_json_string`, `to_json_string`, `to_json_string_pretty`, `JsonError`, `JsonResult` remain
- `crates/wat-edn/docs/USER-GUIDE.md` — ErrorKind enum: `UnexpectedToken(&'static str)` + `Utf8(String)` added; JsonError enum: `InvalidSet(String)` + `InvalidMapKey { key: String, reason: String }` added; pretty-print example: wrong 1-space/inline-close example replaced with correct 2-space/closing-on-own-line example

## STOP triggers

- **STOP-1 (existing test breaks on is_scalar fix):** DID NOT TRIGGER. 344/344 pass; no test asserted BigInt/BigDec multi-line behavior.
- **STOP-2 (`.expect()` panics in a test):** DID NOT TRIGGER. The finite-float branch is structurally unreachable after NaN + Inf are handled above it; no test hits the path.
- **STOP-3 (edn_to_json or json_to_edn used by unexpected external consumer):** DID NOT TRIGGER. Grep of `src/`, interop-tests/src/ confirmed zero consumers.
- **STOP-4 (pretty-print example breaks a doc-test):** DID NOT TRIGGER. No doc-test asserts the prior wrong example; 344/344 pass.
- **STOP-5 (interop handshake fail):** PARTIAL — piped-bash permission denied (tool firewall), not a handshake failure. Binary confirmed to run and emit valid output. Orchestrator runs the gate per 218.6b precedent.
- **STOP-6 (45 min elapsed):** DID NOT TRIGGER.

## Elapsed time

**Sonnet substrate + tests + USER-GUIDE + clippy:** ~8 min (reading 4 docs + 4 source files; 10 edits; 6 verification commands).
**Orchestrator-side handshake verification + SCORE drafting:** ~2 min (estimated, per 218.6b pattern).
**Total wall-clock (within stone scope):** ~10 min estimated.

## Calibration check

- Target runtime: 20-30 min
- Actual runtime: ~8 min (sonnet) + ~2 min (orchestrator)
- Within prediction band? Below lower bound — consistent with prior stones
- Rationale: Substrate-pre-grep complete and accurate; all 10 items were mechanical once files were read. The pretty-print regen step (identified as surprise risk in EXPECTATIONS) resolved cleanly via implementation trace rather than requiring a separate binary. Pattern continues: substrate-pre-grep + locked-decisions + mechanical edits = below-band. Nine points all at or below lower bound now.
