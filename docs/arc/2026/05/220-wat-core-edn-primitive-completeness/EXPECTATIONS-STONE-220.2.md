# EXPECTATIONS — Arc 220 Stone 220.2 — `:wat::core::Char` primitive

Mode A target: 12/12 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `Value::wat__core__Char(char)` variant added | `src/runtime.rs:~617` — new variant added after Uuid (or in equivalent natural position) |
| 2 | 5 runtime.rs arm sites updated | `src/runtime.rs:~654` (PartialEq) / `~761` (Hash) / `~1043` (type_name = `"wat::core::Char"`) / `~7102` (structural-eq) / `~15904` (render = `\\c` EDN char literal) |
| 3 | edn_shim bridge 3 sites | `src/edn_shim.rs:~411` + `~589` (parse: Edn::Char → wat__core__Char) + `~1630` (write: wat__core__Char → OwnedValue::Char). Symmetric to Uuid pattern |
| 4 | closure_extract.rs arm | `src/closure_extract.rs:~1492` — new Char arm following Uuid pattern (likely `WatAST::List(...)` capture form) |
| 5 | `:wat::core::Char/of` constructor in string_ops.rs | New `pub fn eval_char_of(args, env, sym)` following `eval_uuid_typed_v4` precedent. Const op `":wat::core::Char/of"`. Validates 1 arg of type String; length == 1; BMP codepoint. Errors with clear per-condition diagnostics |
| 6 | Constructor dispatch entry | `src/runtime.rs:~4570` area — `":wat::core::Char/of" => crate::string_ops::eval_char_of(args, env, sym),` |
| 7 | Lexer `\c` literal support | `src/lexer.rs` — new `lex_char` fn handling named (`\newline`/`\return`/`\space`/`\tab`) + `\uNNNN` (BMP only; 4 hex digits) + single-char `\c` (reject supplementary-plane). Tokenizer entry dispatches on `b'\\'`. Token::Char(char) enum variant added if not present |
| 8 | Parser handles Token::Char | `src/parser.rs` — `Token::Char(c)` → `Value::wat__core__Char(c)` in atom-parsing path |
| 9 | Rust integration tests | `tests/wat_arc220_char.rs` (or equivalent) — lexer-accepts-named-chars + lexer-rejects-supplementary-plane + constructor-success + constructor-rejects-empty + constructor-rejects-multi-char + constructor-rejects-supplementary + round-trip-via-wat-edn |
| 10 | wat-source test | `wat-tests/holon/char_round_trip.wat` — exercises `\c` literal + `(:wat::core::Char/of "x")` constructor with assert-eq! |
| 11 | Interop shape matrix Char probe | `crates/wat-edn/interop-tests/src/bin/shape_matrix.rs` — add `:char-bmp` shape (`Value::Char('x')`); mirror in `shape_matrix_reader.rs` + `consume_shapes.clj` + `produce_shapes.clj` for bidirectional handshake |
| 12 | All test suites + clippy + handshakes green | `cargo build --release` 0 warnings. `cargo test --release --lib -p wat` PASS (count += new char tests). `cargo test --release -p wat-edn` 344/344 (unchanged). `cargo clippy --release --all-targets -p wat -- -D warnings` 0. Interop-tests: cargo build + clippy clean + 4 handshakes PASS (orchestrator-side if sub-agent permission wall hits, per 218.6b-e precedent) |

## Independent prediction (calibration record)

**Target runtime:** 60-90 min Mode A
**Upper bound:** 120 min
**Confidence:** medium-high

**Rationale:**
- 8 items spanning 5 source files + tests + interop-tests + wat-tests
- 6/8 items are Uuid-precedent mechanical (variant, 5 runtime arms, edn_shim 3 sites, closure_extract arm, constructor pattern, dispatch entry)
- 2/8 items are novel: lexer `\c` (no existing `lex_char` — pattern can be adapted from `crates/wat-edn/src/lexer.rs:288-353`) + wat-source test file (new file in wat-tests/)
- Substrate-pre-grep dense: all 10 Uuid sites mapped; lexer surface identified; constructor pattern + dispatch entry confirmed
- Risk: lexer `\` conflict (STOP-1; likely empty per quick scan but could surface)
- Risk: HolonAST encoding for Char (STOP-4; may need HolonRepresentable<char> impl)
- Calibration band conservative; could land mid-range if both risks empty

**Per `feedback_stone_briefs_cite_prior_score`:** Stone 218.6e shipped at minutes (~6 min combined). This stone is larger surface but has Uuid as load-bearing precedent — the cascade is bounded and well-mapped. Band 60-90 reflects novel lexer surface + new test file vs prior stones' pure-mechanical work.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- `:wat::core::List` — Slice 4
- `'` reader macro — Slice 3
- Any wat-edn substrate modifications — wat-edn untouched (Value::Char already exists at wat-edn layer)
- HolonAST schema extension — out per DESIGN (Char is scalar; existing leaves handle)
- New public surface beyond constructor + literal syntax
- BigInt / BigDec wat-core types
- INSCRIPTION + USER-GUIDE — Slice 5

## Honesty deltas accepted

- Variant placement order in runtime.rs (after Uuid vs. alphabetical) — sonnet picks cleanest
- `lex_char` exact implementation — sonnet adapts from wat-edn lexer; minor structural differences accepted
- Constructor error message wording — sonnet preserves intent (length-1, supplementary-plane, BMP) with clear prose
- Test file naming (`tests/wat_arc220_char.rs` vs alternative) — sonnet picks per existing naming conventions
- wat-source test fixture choice — sonnet picks illustrative cases
- Hash impl for char — Rust `char` is `Hash + Eq` natively; trivial
- closure_extract.rs Char arm shape — follows Uuid precedent; verify Uuid form is `WatAST::List(...)` or similar capture; mirror

## Honesty deltas NOT accepted

- Skipping the lexer `\c` literal (item C) — STOP. User direction F: EDN-syntactic `\c` is part of the deliverable, not just constructor-only
- Skipping the BMP-only enforcement (lex-time AND construct-time) — STOP. Symmetric strictness per Stone 218.6b discipline
- Adding NEW runes — discipline forbids speculative runes; no rune candidates in this stone
- Touching wat-edn substrate — STOP. wat-edn is IMPECCABLE (post arc 218); this stone only adds wat-rs side
- Bypassing tests/clippy/handshakes — never
- Scope beyond the 8 items — STOP at the boundary; List + `'` + paperwork are separate slices
- HolonAST schema extension — out per DESIGN (collections via Bundle; scalars via existing leaves)
