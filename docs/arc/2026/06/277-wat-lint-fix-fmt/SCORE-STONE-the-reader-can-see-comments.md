# SCORE — STONE: the reader can see comments

No commit. Whole stone is `crates/wat-reader/src/lexer.rs`. Parser, formatter, `src/` untouched.

## What shipped

```
pub struct Comment { pub text: String, pub span: Span }   // text VERBATIM, `;`s included

pub fn lex(src, file)               -> Result<Vec<SpannedToken>, LexError>            // UNCHANGED
pub fn lex_with_comments(src, file) -> Result<(Vec<SpannedToken>, Vec<Comment>), LexError>
```

`lex` delegates and drops comments. Capture is at the existing `;` skip — `;` through (not including) `\n`, or through EOF. Same `span_with_end(start, end)` as every other token. No `Token` variant.

`git diff --stat crates/wat-reader/src/parser.rs` — empty.
`Token` enum — no new variant. `src/lexer.rs` is already `pub use wat_reader::lexer::*;`; Comment rides that re-export without a `src/` edit.

## The four-hazard fixture

`reader_can_see_comments_four_hazards` in `lexer.rs`. Two sources (hazard 4 cannot share a file with forms), plus two named trap-door measurements.

```
"has;semi"     hazard 1: `;` inside a string is NOT a comment
() ;; trail    hazard 2: trailing `;;`; span starts at the first `;` (L2C4..L2C12)
\;             trap-door 4: Token::Char(';'), zero extra comments
;eof           hazard 3: EOF, no trailing newline; text `;eof`, span L4C1..L4C5
```

Tokens from that source: `Str("has;semi")`, `LParen`, `RParen`, `Char(';')`. Two comments: `;; trail`, `;eof`.

Hazard 4 — comment-only file `; only\n;; still\n`: two comments, **zero tokens**.

`lex` vs `lex_with_comments` compared **field-wise** (token + `span.line/col/end`). `Span::eq` is unconditionally true, so `Vec<SpannedToken>` PartialEq cannot prove the side-channel claim; the witness reads the fields.

## Sabotage red (verbatim, not re-run)

Deleted the `comments.push` (left the skip). One run. Arm: `assert_eq!(comments.len(), 2, "missing comment: …")`.

```
   Compiling wat-reader v0.1.0 (/home/john/work/holon/wat-rs/crates/wat-reader)
warning: variable does not need to be mutable
   --> crates/wat-reader/src/lexer.rs:351:9
    |
351 |     let mut comments = Vec::new();
    |         ----^^^^^^^^
    |         |
    |         help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

warning: `wat-reader` (lib) generated 1 warning (run `cargo fix --lib -p wat-reader` to apply 1 suggestion)
warning: `wat-reader` (lib test) generated 1 warning (1 duplicate)
    Finished `release` profile [optimized] target(s) in 0.68s
     Running unittests src/lib.rs (target/release/deps/wat_reader-0bcaacab0433a30b)

running 1 test

thread 'lexer::tests::reader_can_see_comments_four_hazards' (3463128) panicked at crates/wat-reader/src/lexer.rs:1796:9:
assertion `left == right` failed: missing comment: expected `;; trail` and `;eof`; got 0 comments
  left: 0
 right: 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test lexer::tests::reader_can_see_comments_four_hazards ... FAILED

failures:

failures:
    lexer::tests::reader_can_see_comments_four_hazards

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 103 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p wat-reader --lib`
```

Capture restored verbatim. Witness green after restore.

## Commands I ran

| command | result |
|---|---|
| `cargo build --release` | Finished `release` in 20.41s |
| `cargo test --release -p wat-reader` | **104** lib + **2** `reader_totality` passed, 0 failed |
| `cargo nextest run --release --test lint` | **118 tests run: 118 passed, 0 skipped** (92.041s) |

Floor and clippy `--all-targets -D warnings` are the orchestrator's.

## Trap doors

1. **Span end.** Hazard 3: `;eof` at EOF ends at col 5 (one past `f`). Hazard 2 ends before `\n`.
2. **CRLF.** Capture is `\n`-exclusive, so `; crlf\r\n()` yields text `"; crlf\r"`, span L1C1..L1C8. Measured in the witness. Walked every `.wat` / `.wat.bad` / `.rs` under `wat-rs/` (minus `target`/`.git`): **0 files contain `\r`**. Corpus does not exercise it; a future CRLF file would carry the `\r` in the comment until a later policy decides otherwise.
3. **`lex` error path.** Same loop, same `LexError`; `lex` is `lex_with_comments` + drop. No new error-order site.
4. **`\;` character literal.** The `\\` branch runs after the comment skip at the top of the loop, but `\;` starts with `\`, so it never hits the skip. Trap-door 4 is **not** a sibling of hazard 1. The fixture includes it; zero comments from it.

## What surprised me

`Span::eq` always returns true. A `assert_eq!(lex(src), tokens_from_lex_with_comments)` would have been vacuous on spans. The witness compares `line`/`col`/`end` by hand.

Census not touched (no `.wat` rewrite). 571 · 85 · 52 is the last known floor; I did not re-count.
