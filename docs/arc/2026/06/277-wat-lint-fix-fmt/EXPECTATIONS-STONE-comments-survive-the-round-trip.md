# EXPECTATIONS — STONE: comments survive the round trip

Written BEFORE the strike, so the result cannot move the goalposts.

| # | what | command | expected |
|---|---|---|---|
| 1 | it builds | `cargo build --release` | clean |
| 2 | a comment above a form SURVIVES | round-trip test, input `;; c\n(a b)` | output contains `;; c` on its own line, before `(a b)` |
| 3 | a TRAILING comment survives | round-trip test, input `(a b) ;; t` | output contains `;; t` after `(a b)`, on the same line |
| 4 | a comment INSIDE a form survives | input `(a\n  ;; why\n  b)` | `;; why` present, between `a` and `b` |
| 5 | **the fixpoint** | parse→print→parse over each input above | same forms, same comment TEXTS, same ORDER |
| 6 | **the non-vacuity control** | the same test asserts a COUNT of comments seen | count > 0 and printed — a green with zero comments examined proves nothing |
| 7 | `lex` is untouched | `crates/wat-reader/src/lexer.rs:327-333` | still `let (tokens, _comments) = …`; still delegating |
| 8 | `parse_all_with_file` untouched | `git diff` on parser.rs | only an ADDED fn; no line of the existing two changed |
| 9 | no new `WatAST` variant | `git diff crates/wat-reader/src/ast.rs` | EMPTY |
| 10 | reader crate green | `cargo test --release -p wat-reader` | ≥107 pass (105 lib + 2 totality was the last count), 0 fail |
| 11 | the char-literal hazard holds | the first stone's `\;` test | still green — `\;` is a char literal, NOT a comment |
| 12 | the floor (ORCHESTRATOR) | `scripts/floor.sh` | 5171+ run, **0 FAILED** |
| 13 | clippy (ORCHESTRATOR) | `cargo clippy --release --all-targets -- -D warnings` | 0 |

**Runtime prediction:** 25-45 min. The parser change is ~8 lines and mechanical; the printer's
placement logic is the real work.

## Trap-doors named in advance

- **The `\;` hazard.** `\;` is a CHAR LITERAL, not a comment. The first stone measured this. A
  placement pass that scans text rather than using the lexer's spans will re-break it — row 11.
- **A comment at end-of-file** with no following form. Placement has no "next form" to attach above.
  It must still be emitted.
- **Two comments on consecutive lines** above one form. Both go above, in order — not merged, not
  reordered.
- **The vacuous green.** Row 6 exists because an "all comments preserved" pass over zero comments is
  indistinguishable from success. This was published once this session; the control is mandatory.
- **A comment inside a form that is being JOINED onto one line.** It cannot be — a line comment pins
  a newline. If a layout decision and a comment conflict, the comment wins. (This stone does not
  make layout decisions, but the printer must not produce a joined line carrying a comment.)
