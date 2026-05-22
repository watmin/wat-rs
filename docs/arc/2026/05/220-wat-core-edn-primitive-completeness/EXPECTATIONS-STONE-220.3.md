# EXPECTATIONS — Arc 220 Stone 220.3 — `'` reader macro

Mode A target: 7/7 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `Token::Quote` variant added | `src/lexer.rs:~115` — new variant after `Quasiquote` with doc comment citing arc 220 Slice 3 + arc 171 distinction |
| 2 | Lexer top-level `b'\''` emit | `src/lexer.rs:~281-292` area — new branch alongside backtick / unquote / unquote-splicing emitting `Token::Quote` |
| 3 | Parser dispatch | `src/parser.rs:~286` — new branch: `Token::Quote => self.parse_reader_macro(":wat::core::quote", span)` |
| 4 | 3 new parser tests | `src/parser.rs:~810` area — `quote_wraps_following_form`, `quote_over_list`, `quote_does_not_disturb_keyword_body_apostrophe` (arc 171 invariant guard) |
| 5 | Arc 171 keyword-body regression check | `cargo test --release --lib -p wat keyword_apostrophe` PASSes unchanged. All existing `:foo'2` keyword-body tests continue to parse as single Keyword tokens |
| 6 | wat-edn untouched | `cargo test --release -p wat-edn` 344/344 (unchanged; this stone is parser-only) |
| 7 | All test suites green | `cargo build --release` 0 warnings. `cargo test --release --lib -p wat` PASS (count += 3). `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0. Wat-clippy NOT gated (arc 170 backlog visibility per user direction) |

## Independent prediction (calibration record)

**Target runtime:** 15-25 min Mode A
**Upper bound:** 35 min
**Confidence:** very high

**Rationale:**
- Smallest stone since 218.6e (~6 min combined)
- 4 mechanical edits + 3 tests, all backtick-precedent
- Verbatim backtick code shown in BRIEF — sonnet adapts the same shape for `'`
- Risk: arc 171 keyword-body regression (STOP-1; the position discipline must hold) — but lex_keyword absorbs `'` inside body BEFORE top-level dispatch sees it, so structurally sound
- Calibration: 12 stones at-or-below band; weaponized BRIEF + tight precedent = low surprise

**Per `feedback_stone_briefs_cite_prior_score`:** Stone 220.2 shipped ~30 min (substantial, with novel lex_char). Stone 220.3 is ~10% of that scope — pure backtick-precedent copy. Band 15-25 conservative.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- `:wat::core::List` — Slice 4
- INSCRIPTION / USER-GUIDE — Slice 5
- wat-edn modifications
- New runes
- Interop-tests handshakes (no interop-tests files touched)
- New public surface beyond the `'` reader macro

## Honesty deltas accepted

- Exact placement of `Token::Quote` (alphabetical / next-to-Quasiquote) — sonnet picks
- Test fixture exact phrasing — sonnet may add additional regression tests if surfaces an interesting edge case
- Doc comment wording — sonnet preserves intent (arc 220 Slice 3 + arc 171 distinction)

## Honesty deltas NOT accepted

- Skipping the arc 171 regression-guard test — STOP. The position discipline IS the load-bearing invariant; must explicitly verify
- Adding `'` handling INSIDE `lex_keyword` (the arc 171 path) — STOP. That path is unchanged
- Bypassing the keyword_apostrophe test regression check — STOP
- Touching `:wat::core::quote` special form OR `eval_quote` — they exist + work; this stone only adds the syntax sugar
- Adding new runes (no candidates this stone) — STOP
- Scope beyond the 4 edits + 3 tests — STOP at the boundary
