# EXPECTATIONS — the reader can see comments

| # | what | the command | expected |
|---|---|---|---|
| 1 | comments are captured, byte-exact | the witness | every comment's text verbatim, `;`s included |
| 2 | spans are right | the witness | each span starts at the `;`, ends before the `\n` |
| 3 | a `;` inside a string is NOT a comment | hazard 1 | zero comments captured from it |
| 4 | EOF without a trailing newline | hazard 3 | captured, span ends at EOF |
| 5 | a comment-only file | hazard 4 | all comments captured, zero tokens |
| 6 | `lex`'s output is UNCHANGED | same input, before vs after | identical `Vec<SpannedToken>` |
| 7 | no new `Token` variant | `git diff` on the `Token` enum | empty |
| 8 | the parser is untouched | `git diff --stat crates/wat-reader/src/parser.rs` | empty |
| 9 | the witness is NOT vacuous | delete the capture, run it | **RED**, naming a missing comment |
| 10 | the floor, doctests included | orchestrator, centrally | 5169/5169 or better |
| 11 | clippy | `--all-targets -D warnings` | 0 |

Rows 6–8 are the design's whole claim: a side channel costs nothing to anyone who does not ask. If
any of them moves, the stone did something other than what it was drawn as.

Row 3 is the correctness row, and it is inherited rather than built — the string branch consumes a
literal atomically, so an interior `;` never reaches the capture site. The row exists to PROVE that
inheritance, not to test new code.

## Independent prediction

**20–40 minutes.** The capture is a handful of lines at one site. The fixture and the
`lex`-unchanged proof are the real work.

## Trap doors — named before, not after

1. **`Comment.span`'s end.** A comment ends before the `\n`, and the last comment in a file may end
   at EOF with no newline at all. Off-by-one here is invisible until a formatter tries to place the
   comment and puts it one column wrong. Hazard 3 exists for this.
2. **CRLF.** If any fixture or corpus file uses `\r\n`, a naive "to `\n`" capture leaves a trailing
   `\r` inside the comment text. I have not measured whether the corpus has any; if the rider finds
   one, that is a finding.
3. **`lex` delegating may change its error path.** `lex_with_comments` returning `Result` and `lex`
   unwrapping into the same `LexError` must not alter which error fires first for malformed input.
   Row 6 catches output drift; an error-ORDER change it would not catch on valid input.
4. **A `;` inside a CHARACTER literal.** `\;` — I have not checked whether the char lexer runs
   before the comment skip the way the string branch does. If it does not, hazard 1's sibling is
   real and unmeasured.

## What I will do on return

Re-run rows 1–9 myself. Rows 10–11 are mine alone and are the only verdict on green.
