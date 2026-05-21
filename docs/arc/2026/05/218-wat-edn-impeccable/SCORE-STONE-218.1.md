# SCORE — Arc 218 Stone 218.1 — L1 fixes + cross-spell convergence

**Mode:** A
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-21

## Result: 9/9 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | cernere L1.A — USER-GUIDE.md:159 rewritten | PASS | Phantom `p.parse_next()?` loop replaced with a block showing the real `Parser` API surface: `Parser::new(input).parse_top()` and `Parser::new_wire(wire_input).parse_all()`. Comment explains `parse_top` (single form + expect EOF), `parse_all` (all forms until EOF), and `new_wire` (wire-mode). Teaching intent preserved: the block distinguishes direct `Parser` use from the convenience free functions above it. |
| 2 | cernere L1.B — IPC-BRIDGE.md:212 rewritten | PASS | Q1 framing prose updated: replaced `wat-edn's Parser::parse_next` (phantom) with `wat-edn's Parser::parse_all`. Added one paragraph naming the full real API surface (`new` / `new_wire` / `parse_top` / `parse_all`), explicitly stating no `parse_next` method exists, and explaining that incremental behavior comes from upstream buffering. |
| 3 | temperare L1 — lexer.rs:346-347 single-iterator | PASS | Collapsed double `chars()` walk to one iterator. `let mut it = body_str.chars(); if let Some(c) = it.next() { if it.next().is_none() { return Ok(Token::Char(c)); } }`. Single iterator construction; single traversal; semantically identical to original. |
| 4 | escapes.rs gains `write_keyword_body_to` | PASS | `crates/wat-edn/src/escapes.rs` — new `pub(crate) fn write_keyword_body_to<W: std::fmt::Write>(seg: &str, w: &mut W) -> std::fmt::Result`. Docstring cites arc 170 REALIZATIONS-SLICE-1.md pass 14 (swap rationale) + arc 218 stone 218.1 (extraction). Visibility: `pub(crate)` — both call sites are crate-internal; no external consumer exists. |
| 5 | value.rs collapses to shared helper | PASS | `crates/wat-edn/src/value.rs` — local `write_keyword_segment` fn deleted (was lines 451–469). Callers at the two sites in `impl fmt::Display for Keyword` now route to `crate::escapes::write_keyword_body_to(ns, f)?` and `crate::escapes::write_keyword_body_to(self.name(), f)`. |
| 6 | writer.rs collapses to shared helper | PASS | `crates/wat-edn/src/writer.rs` — local `write_keyword_body` fn deleted (was lines 169–196). Import updated: `use crate::escapes::{char_to_name, encode_string_escape, write_keyword_body_to}`. Both call sites in `write_keyword` now use `write_keyword_body_to(seg, out).expect("String fmt::Write is infallible")`. `String` infallibility honored with an honest `.expect()`. |
| 7 | `display_equivalence.rs` test still PASSES | PASS | `tests/display_equivalence.rs` — all 4 tests pass: `bare_slash_symbol_matches_display`, `keyword_writer_matches_display`, `symbol_writer_matches_display`, `tag_writer_matches_display`. Byte-identical lock between `Display` + writer paths intact. |
| 8 | wat-edn test suite: zero regressions | PASS | `cargo test --release -p wat-edn`: **336/336 PASS** (42 unit + 16 accessor + 176 comprehensive + 4 display_equivalence + 8 pretty + 7 round_trip + 23 spec_conformance + 36 spec_strict + 0 uuid_v4_mint + 23 wire_encoding + 1 doc-test). `cargo clippy --release -p wat-edn -- -D warnings`: **0 warnings, 0 errors**. |
| 9 | SCORE doc inscribed | PASS | This file. |

## Deltas from EXPECTATIONS

**Delta 1 — USER-GUIDE teaching block shape.**
The original phantom example was a `loop { match p.parse_next()? { None => break, Some(v) => ... } }`. Replacing it with a single `Parser::new(input).parse_all()?` call would exactly duplicate the free-function `parse_all("1 2 3")?` example already present 3 lines above. Instead the replacement shows two distinct real API forms — `parse_top` and `new_wire` — that are NOT covered by the convenience functions. Teaching intent is preserved (showing how to drive `Parser` directly) and the block now adds coverage rather than duplicating it.

**Delta 2 — `write_keyword_body` in writer.rs: one `.expect()` per call site.**
The BRIEF suggested a single `.expect()` rune. Two call sites exist (namespace segment + name segment). Both get `.expect("String fmt::Write is infallible")`. This is verbose-but-honest: each call site's infallibility claim is explicit, not inherited from a wrapper. Consistent with the `feedback_verbose_is_honest` principle.

**No other deltas.** All three edits landed exactly as specified. `fmt::Write` unification worked cleanly (STOP-3 did not trigger). No test regressions (STOP-1, STOP-2 did not trigger).

## Verification summary

```
cargo build --release -p wat-edn          — OK (0 warnings, 0 errors)
cargo test --release -p wat-edn           — 336/336 PASS (zero regressions)
cargo clippy --release -p wat-edn -- -D warnings  — 0 warnings, 0 errors
```

`display_equivalence.rs` result: **4/4 PASS** — `keyword_writer_matches_display` confirms byte-identical lock between `Keyword::fmt` (now via `escapes::write_keyword_body_to`) and `write_keyword` in writer.rs (now via same helper). Structural proof of semantic preservation.

## Files changed

- `crates/wat-edn/docs/USER-GUIDE.md` — phantom `parse_next` loop replaced with real API block
- `crates/wat-edn/docs/IPC-BRIDGE.md` — Q1 framing prose corrected; real API surface named
- `crates/wat-edn/src/lexer.rs` — char-literal single-char arm: single `chars()` iterator
- `crates/wat-edn/src/escapes.rs` — new `pub(crate) fn write_keyword_body_to<W: fmt::Write>`
- `crates/wat-edn/src/value.rs` — `write_keyword_segment` deleted; callers route to `escapes::write_keyword_body_to`
- `crates/wat-edn/src/writer.rs` — `write_keyword_body` deleted; import + callers updated

## STOP triggers

- **STOP-1 (`display_equivalence.rs` regresses):** DID NOT TRIGGER. 4/4 pass.
- **STOP-2 (other wat-edn test regresses):** DID NOT TRIGGER. 336/336 pass.
- **STOP-3 (`fmt::Write` unification broken):** DID NOT TRIGGER. `fmt::Formatter<'_>` and `String` both implement `fmt::Write`; generic bound unified cleanly.
- **STOP-4 (doc example intent unclear):** DID NOT TRIGGER. Context read showed `parse_next` was the only phantom; real API surface confirmed in `parser.rs:34,45,53,62`.
- **STOP-5 (60 min elapsed):** DID NOT TRIGGER.

## Elapsed time

Target: 25-45 min. Actual: ~20 min. Below prediction band (lower than lower end).

## Calibration check

- Target runtime: 25-45 min
- Actual runtime: ~20 min
- Within prediction band? Below lower end — faster than predicted
- Rationale: All three pieces were fully mechanical. Orchestrator pre-greps confirmed exact line locations; no ambiguity. `fmt::Write` unification resolved immediately. The only non-trivial judgment call was the USER-GUIDE replacement shape (Delta 1) — resolved by noting the duplicate teaching intent and pivoting to `parse_top` + `new_wire` coverage instead. Total: clean build on first attempt; tests 336/336 green on first run.
