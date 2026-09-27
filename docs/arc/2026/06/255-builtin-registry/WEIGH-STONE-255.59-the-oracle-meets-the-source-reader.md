# WEIGH — STONE 255.59: the clj oracle meets wat's source reader — ACCEPTED (measurement)

**Executor: grok via pulsare, commit `6366bcb45`** (the SCORE and an `#[ignore]` probe,
`crates/wat-reader/tests/clj_oracle_source_parity.rs`). Weighed by the orchestrator on 2026-09-27.

## Re-run

- `cargo test -p wat-reader --test clj_oracle_source_parity -- --ignored`: fails with `source reader vs clj oracle (62)`,
  as scored (44 non-exempt divergences + 18 spellings the probe's constructors cannot build).
- `crates/wat-reader/src/lexer.rs:433-452`: the only `#` shapes are `#holon` and `#{`.

## The reading (221 cases: 152 agree, 21 same spelling, 4 exempt desugars, 44 divergences)

| class | n | what |
|---|---|---|
| (H) `#` dispatch | 13 | tags (`#inst`, `#uuid`, `#u/T`, `#myapp/…`) read as **two forms** (inside a call: an extra argument); `#_` is a symbol, not a discard; `##Inf`/`##-Inf`/`##NaN` are symbols. `WatAST` has no tagged or discard node |
| (K) colon in a name | 10 | the source reader accepts `::foo`, `:wat::core::x`, `a::b` … which clj and `wat-edn` refuse: the `::` the 251 flip retires |
| (M) reader macros | 2 | `@` and `^` are claimed by the ward for the source reader, and are not implemented |
| (O) other | 19 | **real source-reader bugs:** integers past `i64` **silently become floats** (`9223372036854775808` → `9.223372036854776e18`); `:a😀`/`:λ` keywords are **byte-corrupted** (Latin-1 widening, `lexer.rs:820`); `+7` is a symbol; no `1.5M`; no `\uNNNN` in strings; duplicate map keys and set elements kept; `/x`, `x/` accepted. Two are deliberate standing rulings (non-ASCII token start refused; arc 109's `a<`) |

The ward compares verdicts only, which is why the golden reads OK/OK for rows that are wrong in the source tree. That
is a finding about the ward itself.

## For the builder

- **(O) needs no design.** Each is clj/EDN parity with an oracle row, apart from the two standing rulings. Two of them
  corrupt data silently.
- **(H) needs a ruling** on where a tag lives in source (the SCORE's STOP-1 choices).
- **(K) is the 251 flip's last act:** the lexer stops accepting `::` once the corpus is converted.
