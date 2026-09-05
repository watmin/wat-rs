# BRIEF — the reader can see comments

Design: `[[DESIGN-STONE-the-reader-can-see-comments]]` (same dir). Read it first — it carries the
side-channel decision and, more importantly, what is deliberately NOT in this stone.
Anchor: `/home/john/work/holon/wat-rs`; verify with `pwd`.

The whole stone lives in `crates/wat-reader/`. It does not touch the parser, the formatter, or
anything in `src/`.

## THE WORK

**Rooms, in order:** `crates/wat-reader/src/lexer.rs:73` (`SpannedToken` — tokens already carry
spans; `Comment` mirrors that shape) · `:311` (`pub fn lex`) · `:350-356` (the comment skip — the
ONE capture site) · `:459` (the string branch — read it, then leave it alone; see below) · `:1069`
(`line_comment`, the test that proves comments are discarded today).

```rust
pub struct Comment { pub text: String, pub span: Span }   // text VERBATIM, `;`s included

pub fn lex(src, file)               -> Result<Vec<SpannedToken>, LexError>            // UNCHANGED
pub fn lex_with_comments(src, file) -> Result<(Vec<SpannedToken>, Vec<Comment>), LexError>
```

`lex` delegates to `lex_with_comments` and drops the comments, so **every existing caller is
byte-identical.** No new `Token` variant — `Token::` has 111 sites in this file and 32 in the
parser, and a variant would make every one of them a place that must remember to skip comments.

**Capture at the existing skip site.** The bytes from the `;` to (not including) the newline are
the text; the span is the same `span_with_end(start, end)` shape every other token uses.

⛔ **Do NOT re-implement string awareness.** A `;` inside a string literal already never reaches
that site — the string branch at `:459` consumes a literal atomically through its closing quote.
The capture inherits that correctness. **If it feels like you need to check for strings, the
capture is in the wrong place** — STOP and say so.

## THE WITNESS — four hazards, and the fixture must carry all four

A round-trip test asserting every comment is captured with byte-exact text and correct spans, over
a fixture containing:

```
1  a `;` INSIDE a string literal        -> must NOT be captured as a comment
2  a `;;` trailing on a line after code -> captured, span starts at the `;`
3  a comment at EOF with NO trailing newline
4  a file that is ONLY comments (no forms at all)
```

Hazard 1 is the one that matters — it is the reason the capture site is where it is.

**And prove `lex` is unchanged**: the same input through `lex` yields the identical
`Vec<SpannedToken>` it did before this stone. That is the whole claim of the side-channel design;
assert it, do not assume it.

**Show it firing.** Delete the capture (leave the skip), and the witness must go RED naming a
missing comment. Report that red's text verbatim, then restore.

## STOP TRIGGERS — rejections. Ship nothing, report, let me re-plan.

**STOP-1 — no `Token` variant, no parser change.** If capturing seems to require either, STOP.
That is the design's central decision and its reversal is mine, not the stone's.

**STOP-2 — no attachment.** Do not decide which AST node a comment belongs to, do not add a field
to any AST node, do not sort comments into leading/trailing/section-break. That is policy and it
belongs beside the style rules. This stone makes comments VISIBLE; that is all.

**STOP-3 — string awareness at the capture site.** See above. If it looks necessary, the site is
wrong and I want to hear it rather than have it worked around.

**STOP-4 — a red is a red.** Do NOT re-run. Copy the failing test's whole stdout+stderr block
verbatim, name the exact assertion, report.

## What you run, and what you do not

FOREGROUND: `cargo build --release`, `cargo test --release -p wat-reader`, and
`cargo nextest run --release --test lint` before you yield (`-p` runs do not build `tests/lint/`,
which is how 20 loose assertions once reached the central floor). **Do not run the full floor** —
I run it centrally on a quiescent tree, and it includes a doctest stage; do not disable it.
No commit, push, stash, or revert.

## Report

The four-hazard fixture and what each asserted · proof `lex`'s output is unchanged · the sabotage
red verbatim · the `--test lint` result · anything that surprised you.
