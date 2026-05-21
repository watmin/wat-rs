# EXPECTATIONS — Arc 218 Stone 218.6b — Emoji revert + interop-tests warning cleanup

Mode A target: 10/10 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `write_char` panics on supplementary-plane | `crates/wat-edn/src/writer.rs:307-330` — `write_char` includes a `cp > 0xFFFF` branch that panics with diagnostic naming the codepoint + the BMP-only constraint + cross-language reason. BMP control/DEL + non-printable + printable behaviors preserved. |
| 2 | `lex_char` rejects supplementary-plane | `crates/wat-edn/src/lexer.rs:345-352` — single-character path includes `(c as u32) > 0xFFFF` gate; returns `Error::at(start, ErrorKind::InvalidChar(...))` with diagnostic surfacing "supplementary-plane" or "BMP" terminology. |
| 3 | round_trip.rs test replacement | `crates/wat-edn/tests/round_trip.rs` — `supplementary_plane_char_round_trips` DELETED. Two new negative tests: `writer_panics_on_supplementary_plane_char` (uses `#[should_panic(expected = "supplementary-plane")]` or equivalent) + `parser_rejects_supplementary_plane_char_literal` (asserts `parse("\\😀").expect_err(...)` returns InvalidChar with diagnostic surfacing the constraint). |
| 4 | USER-GUIDE BMP-only note | `crates/wat-edn/docs/USER-GUIDE.md` — char-literal section gains a one-paragraph note explaining wat-edn rejects supplementary-plane chars at both writer and parser, citing cross-language interop with Clojure's reader. Exact placement + wording sonnet's call. |
| 5 | `shape_matrix.rs:37` PI fix | `Value::Float(3.14)` → `Value::Float(2.5)`. |
| 6 | `shape_matrix_reader.rs:70` PI fix | Assertion comparison value flipped from `3.14` → `2.5`; tolerance unchanged. |
| 7 | Clojure interop sides updated | `crates/wat-edn/interop-tests/clj/consume_shapes.clj` — `:primitive-f64` assertion updated to `2.5`. `crates/wat-edn/interop-tests/clj/produce_shapes.clj` — `:primitive-f64` produce value updated to `2.5`. Both sides of the bidirectional matrix consistent. |
| 8 | Unused imports dropped | `crates/wat-edn/interop-tests/src/main.rs:10` — `Symbol` removed from `use wat_edn::{...}`. `crates/wat-edn/interop-tests/src/bin/typed_reader.rs:5` — `Value` removed (or whole import collapsed to a simpler form). |
| 9 | All test suites green | `cargo build --release -p wat-edn` 0 warnings. `cargo test --release -p wat-edn` PASS at 344 (343 baseline - 1 deleted + 2 added). `cargo test --release --lib -p wat` PASS 824/0. From `crates/wat-edn/interop-tests/`: `cargo build --release` 0 warnings + `cargo test --release` PASS (no test count change expected). |
| 10 | Clippy + interop handshakes clean | `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0 warnings. From `crates/wat-edn/interop-tests/`: `cargo clippy --release --all-targets -- -D warnings` 0 warnings. All 4 interop handshakes (consume.clj / reader / consume_shapes.clj / shape_matrix_reader) PASS — shape matrix now exchanges `:primitive-f64 = 2.5`. |

## Independent prediction (calibration record)

**Target runtime:** 15-25 min Mode A
**Upper bound:** 40 min
**Confidence:** high

**Rationale:**
- Pure mechanical edits — no architectural decisions
- Part A surface: 1 writer fn + 1 lexer branch + 2 test replacements + 1 USER-GUIDE note = 4 sites
- Part B surface: 2 Rust sites + 2 Clojure sites + 2 unused-import drops = 6 sites (mostly one-line)
- Substrate-pre-grep dense: every line number confirmed; both clippy diagnostics pinpoint
- Sonnet has 218.6 SCORE for calibration shape; same author/pattern
- Calibration six-for-six below band: 218.1 ~20, 218.2 ~15, 218.3 ~25, 218.4 ~20, 219.1 below, 218.6 ~8. This stone smaller than 218.6 (no API moves, no interop probe surgery)

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites Stone 218.6 SCORE (~8 min ship; 12/12 PASS with STOP-6 modified). 218.6b has ~half the surface and no decision-points — confidence high; band 15-25.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- L2 sweep (struere, solvere L2, cernere L2, temperare L2, intueri, purgare L2) — deferred until vigilia checkpoint cast informs final scope
- INSCRIPTION + arc 218 closure — user direction 2026-05-22: "218 has work we haven't expressed yet"
- Any other interop-tests rework (the matrix shape, additional probes) — Part B is warning-cleanup only
- New char-literal forms (no `#wat-edn.char/supplementary` tag minting) — scope creep
- Performance optimization — surfaced items only
- Touching tagged-literal naming or wat-edn syntax — encoding doctrine locked

## Honesty deltas accepted

- `write_char` panic message wording — sonnet preserves "supplementary-plane" + "BMP" terminology; exact prose may shift
- `lex_char` error diagnostic wording — same constraint
- USER-GUIDE placement of the BMP-only note — sonnet picks the cleanest location near the char-literal teaching
- Non-PI float choice (BRIEF suggests `2.5`; sonnet may pick another clean value like `1.5` or `4.5` as long as all 4 sites stay consistent + Clojure pr-str / wat-edn write both emit identical lexical form)
- `typed_reader.rs:5` import collapse — `use wat_edn::parse;` vs `use wat_edn::{parse};` — rustfmt may prefer one over the other
- Test count: report actual. May be 344 (-1 + 2 = +1 net) or 345 if sonnet adds a third negative test (e.g. proving named-char `\newline` still works to lock the no-regression contract)

## Honesty deltas NOT accepted

- Skipping the writer panic in favor of a Result return — would change API; out of scope
- Skipping the parser rejection (writer-only) — symmetric strictness is the contract; rejection at both boundaries
- Keeping the deleted `supplementary_plane_char_round_trips` test in any form — it asserts the wrong behavior
- Using `#[allow(clippy::approx_constant)]` to silence the PI lint — the value carries no PI semantics; fix at root by changing the value
- Bypassing tests/clippy/handshakes — never
- Touching scope beyond the 10 substantive items — STOP at the boundary
- Renaming `Value::Char` or adding a `BmpChar` variant — type system API stays; only the validation gates change
