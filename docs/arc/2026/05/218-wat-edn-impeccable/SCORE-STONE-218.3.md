# SCORE — Arc 218 Stone 218.3 — Contract precision

**Mode:** A
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-21

## Result: 11/11 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | Pretty-print map symmetric with vector | PASS | `crates/wat-edn/src/writer.rs:106-125` — map arm now opens `{` + `\n`, calls `push_indent(out, inner)` before EVERY entry, closes with `\n` + `push_indent(out, level)` + `}`. Structurally identical to vector arm at lines 66-104. |
| 2 | Pretty-print test expectations updated | PASS | All pretty-print tests in `tests/pretty.rs` use `contains('\n')` and round-trip checks (not exact snapshot strings), so no test expectation strings required updating. 0 snapshot strings changed; 8/8 pretty tests pass. The format change is behaviorally correct per round-trip identity; no regression. |
| 3 | `to_json_string` rune annotation | PASS | `crates/wat-edn/src/json.rs:162-170` — `// rune:struere(invariant-coupling)` annotation added above `serde_json::to_string(...)`. Explains `.expect()` is structurally unreachable per `edn_to_json`'s closed construction. |
| 4 | `to_json_string_pretty` rune annotation | PASS | `crates/wat-edn/src/json.rs:175-182` — matching `// rune:struere(invariant-coupling)` annotation added above `serde_json::to_string_pretty(...)`. Same invariant; matching annotation. |
| 5 | `parse_map_key` strict mode | PASS | `crates/wat-edn/src/json.rs:298-312` — EDN-looking keys that fail to parse now return `Err(JsonError::InvalidMapKey { key, reason })` instead of silently falling through to `Value::String`. `JsonError::InvalidMapKey { key: String, reason: String }` variant added to `JsonError` enum. |
| 6 | Probe for parse_map_key strict mode | PASS | `crates/wat-edn/src/json.rs` internal `#[cfg(test)]` — `parse_map_key_strict_edn_looking_invalid_returns_error` test added. Probe: `{":": 1}` — bare colon is EDN-looking (colon prefix) but invalid EDN; asserts `Err(JsonError::InvalidMapKey { .. })`. Test passes. |
| 7 | Closer-token diagnostic split | PASS | `crates/wat-edn/src/parser.rs:158-161` — arm split: `Token::Eof` → `ErrorKind::UnexpectedEof`; `Token::RParen` → `ErrorKind::UnexpectedToken(")")`; `Token::RBracket` → `ErrorKind::UnexpectedToken("]")`; `Token::RBrace` → `ErrorKind::UnexpectedToken("}")`. New variant `UnexpectedToken(&'static str)` added to `ErrorKind` enum in `error.rs` with Display arm `"unexpected token '{t}'"`. |
| 8 | `lexer.rs:213` allocation tightened | PASS | `crates/wat-edn/src/lexer.rs:213` — `String::with_capacity(self.input.len() - body_start)` → `String::with_capacity(self.pos - body_start)`. Already-consumed body length is the tighter upper bound. |
| 9 | Identifier suffix scan fold | PASS | `crates/wat-edn/src/parser.rs:382-409` — two-scan (`body.find('/')` + `name.contains('/')`) folded to single `splitn(3, '/')` pass. Three-arm match: `(None, _)` bare name; `(Some(name), None)` exactly one slash namespaced; `(Some(_), Some(_))` two-or-more slashes illegal. All four error messages preserved exactly: "empty prefix in", "empty name in", "more than one / in", per-side `validate_first_char` wrapping. |
| 10 | wat-edn test suite: zero regressions | PASS | `cargo build --release -p wat-edn` — OK (0 warnings, 0 errors). `cargo test --release -p wat-edn` — **337/337 PASS** (43 unit + 16 accessor + 176 comprehensive + 4 display_equivalence + 8 pretty + 7 round_trip + 23 spec_conformance + 36 spec_strict + 0 uuid_v4_mint + 23 wire_encoding + 1 doc-test). `cargo clippy --release -p wat-edn -- -D warnings` — **0 warnings, 0 errors**. |
| 11 | SCORE doc inscribed | PASS | This file. |

## Deltas from EXPECTATIONS

**Delta 1 — Vigilia file citation: `writer.rs:162-170` is actually `json.rs:162-170`.**
Already documented in BRIEF as an honest delta. The `to_json_string` and `to_json_string_pretty` functions live in `json.rs`, not `writer.rs`. Fix target same; file path corrected per BRIEF.

**Delta 2 — Vigilia line citation: `parser.rs:158` is actually `:159`.**
Already documented in BRIEF as an off-by-one honest delta. Actual token arm is at line 159 (post-218.2 substrate). Fix applied at the correct line.

**Delta 3 — Pretty-print snapshot update count: 0.**
All pretty-print tests use behavioral assertions (`contains('\n')`, round-trip identity) rather than exact snapshot strings. No snapshot strings required updating. The symmetrization changed the output format, but no test was asserting the old asymmetric byte-string directly. 8/8 pretty tests pass without any string changes.

**Delta 4 — `JsonError::InvalidMapKey` field naming: `reason` not `source`.**
BRIEF specified `InvalidMapKey { key: String, source: String }`. `thiserror` treats a field literally named `source` as a `std::error::Error` source, which `String` does not implement. Field renamed to `reason` to avoid the trait conflict. Error message template updated to `"invalid map key '{key}': {reason}"`. Semantics preserved; diagnostic still surfaces both the key and the parse error text.

**Delta 5 — Test file placement for probe (Item 6): internal `json.rs` tests.**
BRIEF offered `comprehensive.rs` / `spec_conformance.rs` / new file. All existing JSON-contract tests (`from_json_string`, `to_json_string`, `JsonError` variants) live in `json.rs`'s internal `#[cfg(test)]` block. Placing the new probe there is the obvious home — same family, same import scope, no new file needed.

**Delta 6 — Test count: 337, not 336.**
The new probe (Delta 5) adds 1 to the unit test count (42 → 43). Total: 337 PASS (was 336). This is additive, not a regression.

**Delta 7 — `ErrorKind::UnexpectedToken` shape: `(&'static str)`.**
BRIEF offered `(&'static str)` vs `(String)` vs reuse `UnexpectedByte(u8)`. All three closer token names (`")"`, `"]"`, `"}"`) are string literals — `&'static str` is the natural fit, avoids allocation, and matches the diagnostic pattern where the token name is known at compile time. `UnexpectedByte(u8)` was rejected: a byte doesn't cleanly display as a token name for multi-char tokens (though these are single-byte), and the diagnostic message `"unexpected token '}'"` is more human-readable than `"unexpected byte 0x7d"`.

**No other deltas.** STOP triggers 1-7 did not fire.

## Verification summary

```
cargo build --release -p wat-edn          — OK (0 warnings, 0 errors)
cargo test --release -p wat-edn           — 337/337 PASS (zero regressions; +1 new probe)
cargo clippy --release -p wat-edn -- -D warnings  — 0 warnings, 0 errors
```

## Files changed

- `crates/wat-edn/src/writer.rs` — `write_pretty_indented` Map arm symmetrized to match Vector pattern
- `crates/wat-edn/src/json.rs` — rune annotations on `to_json_string` + `to_json_string_pretty`; `JsonError::InvalidMapKey` variant added; `parse_map_key` strict mode; new probe test
- `crates/wat-edn/src/error.rs` — `ErrorKind::UnexpectedToken(&'static str)` variant added with Display arm
- `crates/wat-edn/src/parser.rs` — closer-token arm split (4 arms); identifier suffix scan folded via `splitn(3, '/')`
- `crates/wat-edn/src/lexer.rs` — `with_capacity` tightened to `self.pos - body_start`

## STOP triggers

- **STOP-1 (pretty-print breaks behavior tests):** DID NOT TRIGGER. All pretty tests use `contains('\n')` + round-trip identity; format change is transparent to them.
- **STOP-2 (InvalidMapKey ripples to many call sites):** DID NOT TRIGGER. Variant added to enum; `parse_map_key` is the only construction site; zero additional call site changes required.
- **STOP-3 (UnexpectedToken ripples to error display sites):** DID NOT TRIGGER. Added one Display arm in `error.rs`; no other sites affected.
- **STOP-4 (identifier suffix fold breaks parser tests):** DID NOT TRIGGER. All four error messages preserved exactly; 337/337 PASS.
- **STOP-5 (wat-edn test regresses):** DID NOT TRIGGER. 337 PASS (336 original + 1 new probe).
- **STOP-6 (clippy new warnings):** DID NOT TRIGGER. 0 warnings, 0 errors.
- **STOP-7 (90 min elapsed):** DID NOT TRIGGER.

## Elapsed time

Target: 40-65 min. Actual: ~25 min. Below lower bound.

## Calibration check

- Target runtime: 40-65 min
- Actual runtime: ~25 min
- Within prediction band? Below lower end — faster than predicted
- Rationale: Orchestrator pre-greps were accurate and complete (off-by-one and file-delta already documented; no surprises at any target site). The `thiserror` `source`-field conflict was the one compile error, resolved in one edit. The pretty-print test sweep revealed 0 snapshot strings to update (all behavioral/round-trip assertions) — the "sweep" cost was a single grep. Pattern mirrors 218.1 and 218.2 calibration: substrate-pre-grep + mechanical edits ship faster than predicted. The two enum variant additions (InvalidMapKey, UnexpectedToken) were single-function-body changes with no call-site ripple. Calibration trend now three points: 218.1 ~20 (band 25-45), 218.2 ~15 (band 30-50), 218.3 ~25 (band 40-65). Substrate-pre-grep density consistently beats the lower bound.
