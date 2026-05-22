# SCORE — Arc 218 Stone 218.6e — IMPECCABLE polish (6 items)

**Mode:** A
**Agent:** claude-sonnet-4-6 (substrate + docs + clippy)
**Scoring:** orchestrator (claude-opus-4-7) — independent re-verification + interop handshakes (sonnet hit the same piped-bash permission wall as Stones 218.6b + 218.6c + 218.6d; orchestrator runs cross-language gate during independent scoring)
**Date:** 2026-05-21

## Result: 7/8 PASS (row 8 pending orchestrator-side handshake verification)

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | `lexer.rs` local `open_pos` → `quote_start` (intueri L2) | PASS | `crates/wat-edn/src/lexer.rs` — 3 occurrences in `lex_string` renamed: `let open_pos` → `let quote_start` (line 191), `Error::at(open_pos, ...)` → `Error::at(quote_start, ...)` (line 198), `self.lex_string_escaped(open_pos, ...)` → `self.lex_string_escaped(quote_start, ...)` (line 206). Caller local + callee parameter now share `quote_start` across the call boundary. |
| 2 | `writer.rs` `all_scalar` → `all_inline` (intueri L2) | PASS | `crates/wat-edn/src/writer.rs` — `fn all_scalar` renamed to `fn all_inline` (definition at line 71); call site at line 87 updated to `all_inline(items)`. Doc comment retained unchanged (correctly describes the inlining behavior). Name now matches the delegate `is_inline_value` (renamed in 218.6d). |
| 3 | USER-GUIDE §7 example fixed (cernere L2) | PASS | `crates/wat-edn/docs/USER-GUIDE.md` — import at line 405 updated from `use wat_edn::{to_json_string, to_json_string_pretty, from_json_string, edn_to_json, json_to_edn}` to `use wat_edn::{to_json_string, to_json_string_pretty, from_json_string}` (both demoted functions removed). Lines 419-421 "Or work with serde_json::Value directly" block (2 demoted-function calls) removed entirely. Example now shows the 3 public JSON functions + their use; compiles against actual public surface. |
| 4 | Test counts updated (cernere L2) | PASS | Current count verified via cargo: **344** (exact baseline match, unchanged by this stone's edits). 3 sites updated: `crates/wat-edn/README.md:45` "313" → "344"; `crates/wat-edn/README.md:97` "342/342" → "344/344"; `crates/wat-edn/docs/USER-GUIDE.md:792` "313" → "344". All 3 sites now consistent with each other and with reality. |
| 5 | `to_json_string_pretty` rune rewritten (purgare L1) | PASS | `crates/wat-edn/src/json.rs:179-184` — false "consumed by src/edn_shim.rs" claim removed. New wording: "symmetric pretty variant paired with to_json_string (which IS actively consumed by src/edn_shim.rs:105,166 for WAT_TEST_OUTPUT cargo integration per arc 116). to_json_string_pretty itself has no current direct caller; justification is symmetric-completeness with the live compact variant + future Clojure-IPC bridge surface (crates/wat-edn/docs/IPC-BRIDGE.md)." Honest: names the lie, names the real consumer, cites vision without fabricating a current consumer. |
| 6 | `write_to` rune category → `future-fixture` (purgare L2) | PASS | `crates/wat-edn/src/writer.rs:195-200` — category changed from `purgare(public-api)` to `purgare(future-fixture)`. Justification cites IPC-BRIDGE.md:95 as the named future-downstream. Retirement criterion explicit: "This rune retires when the IPC bridge ships and write_to gains a real caller (per purgare SKILL: 'rune retires when the downstream lands')." |
| 7 | All tests + clippy green | PASS | `cargo build --release -p wat-edn` → 0 warnings. `cargo test --release -p wat-edn` → **344/344 PASS** (exact baseline match). `cargo test --release --lib -p wat` → 824/0 PASS (+ 1 ignored, pre-existing). `cargo clippy --release --all-targets -p wat-edn -- -D warnings` → 0 warnings, 0 errors. `cargo build --release --manifest-path .../interop-tests/Cargo.toml` → 0 warnings. `cargo clippy --release --all-targets --manifest-path .../interop-tests/Cargo.toml -- -D warnings` → 0 warnings. |
| 8 | Interop-tests 4 handshakes pass | PENDING — orchestrator-side (handshakes) | Piped handshakes (`| clojure -M ...`) blocked by same permission wall as Stones 218.6b + 218.6c + 218.6d. All changed code is internal to renames and doc comments — zero public API surface touched; all 4 handshake paths use `to_json_string` / `from_json_string` / `write` / parser — all unchanged. Orchestrator runs all 4 during independent scoring. |

## Deltas from EXPECTATIONS

**Delta 1 — Handshake verification moved to orchestrator-side (piped-bash permission wall).**
Same as Stones 218.6b + 218.6c + 218.6d. The `cd crates/wat-edn/interop-tests` compound piped command is denied. Binary confirmed to build cleanly (row 7). All changed public APIs are orthogonal to what the interop handshakes exercise. Per `feedback_sonnet_bash_firewall`.

**No other deltas.** All 6 substantive items shipped as specified.

## Verification summary

```
cargo build --release -p wat-edn                                  — OK (0 warnings)
cargo test --release -p wat-edn                                   — 344/344 PASS (exact baseline match)
cargo test --release --lib -p wat                                 — 824/0 PASS (+ 1 ignored, pre-existing)
cargo clippy --release --all-targets -p wat-edn -- -D warnings    — 0 warnings, 0 errors

cargo build --release --manifest-path .../interop-tests/Cargo.toml     — OK (0 warnings)
cargo clippy --release --all-targets --manifest-path .../interop-tests  — 0 warnings

Interop handshake 1 (wat-edn → consume.clj)                      — pending orchestrator
Interop handshake 2 (produce.clj → reader)                       — pending orchestrator
Interop handshake 3 (shape_matrix → consume_shapes.clj)          — pending orchestrator
Interop handshake 4 (produce_shapes.clj → shape_matrix_reader)   — pending orchestrator
```

## Files changed

- `crates/wat-edn/src/lexer.rs` — local `open_pos` → `quote_start` in `lex_string` (3 occurrences)
- `crates/wat-edn/src/writer.rs` — `all_scalar` → `all_inline` (definition + 1 call site); `write_to` rune category `public-api` → `future-fixture` with honest retirement criterion
- `crates/wat-edn/src/json.rs` — `to_json_string_pretty` rune rewritten: false edn_shim consumer claim dropped; honest symmetry + IPC-BRIDGE vision justification
- `crates/wat-edn/README.md` — test counts updated: "313" → "344" (line 45); "342/342" → "344/344" (line 97)
- `crates/wat-edn/docs/USER-GUIDE.md` — §7 example import and demoted-function block removed (lines 405, 419-421); test count "313" → "344" (line 792)

## STOP triggers

- **STOP-1 (A.1 additional `open_pos` site):** DID NOT TRIGGER. Exactly 3 occurrences in `lex_string` as documented; callee's parameter already `quote_start` from 218.6d.
- **STOP-2 (A.2 additional `all_scalar` caller):** DID NOT TRIGGER. Exactly 1 call site at writer.rs:87 as documented.
- **STOP-3 (B.3 additional demoted-function usage):** DID NOT TRIGGER. `edn_to_json` and `json_to_edn` appeared only at lines 405, 420, 421 in USER-GUIDE as documented.
- **STOP-4 (B.4 test count not 344):** DID NOT TRIGGER. Cargo confirmed 344 exactly (sum of all `test result:` lines).
- **STOP-5 (rune wording rejected by clippy):** DID NOT TRIGGER. 344/344 pass; 0 clippy warnings.
- **STOP-6 (20 min elapsed):** DID NOT TRIGGER.

## Elapsed time

**Sonnet substrate + docs + clippy:** ~8 min (reading 4 brief docs + 5 source files; 8 edits; 7 verification commands).
**Orchestrator-side handshake verification + SCORE drafting:** ~2 min (estimated per 218.6b/c/d pattern).
**Total wall-clock (within stone scope):** ~10 min estimated.

## Calibration check

- Target runtime: 5-10 min
- Actual runtime: ~8 min (sonnet) + ~2 min (orchestrator)
- Within prediction band? YES — within [5-10] min band
- Rationale: Smallest stone yet delivered within band. All 6 edits were surgical once the files were read; all line numbers matched the orchestrator's pre-grep exactly. No surprises. The intueri renames were a single find-and-replace each; the doc fixes were verbatim removals; the purgare rewrites followed the BRIEF verbatim. Pattern holds: locked decisions + mechanical edits = at or below lower bound.
