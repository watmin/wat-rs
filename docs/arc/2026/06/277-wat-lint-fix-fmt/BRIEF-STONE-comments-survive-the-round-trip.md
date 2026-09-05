# BRIEF — STONE: comments survive the round trip

Make a comment survive `source → parse → print`. The lexer already captures comments and the caller
one line up throws them away; carry them through the parser as a **side channel** and teach the
printer to place them. Read `[[DESIGN-STONE-comments-survive-the-round-trip]]` first — it pins the
one contract decision and the placement rule.

## READ IN ORDER — the rooms, and why you are being sent to each

1. **`crates/wat-reader/src/lexer.rs:327-333`** — `lex`. It calls `lex_with_comments` and binds the
   comments to `_comments`. This is the exact line where they die today. **`lex` stays as it is** —
   its callers are the whole tree and none of them want comments.
2. **`crates/wat-reader/src/lexer.rs:75-91`** — `SpannedToken { token, span }` and
   `Comment { text, span }`. Both carry a `Span`, so a merged source-order stream is a sort.
3. **`crates/wat-reader/src/parser.rs:250-262`** — `parse_all_with_file`. Four lines: lex, make a
   `Cursor`, loop `parse_form`, return. Your new function is this with the comments kept.
4. **`crates/wat-reader/src/parser.rs:237-249`** — `parse_one_with_file`, the single-form sibling.
   Look, do not change.
5. **`src/edn/render.rs:775-830`** — `write_wat_source`, the recursive printer. Note the `FloatLit`
   arm's comment: someone already reasoned about round-trip fidelity here. Your printer is a
   comment-aware peer of this function.
6. **`src/edn/render.rs:728-755`** — `eval_ast_to_source`, the intrinsic handler that calls it.
   Look only; **no new intrinsic in this stone.**
7. **`crates/wat-reader/src/lexer.rs:1745-1835`** — the first stone's own tests, including the
   four measured hazards (`\;` as a char literal, CRLF). Copy their shape for yours.

## SKETCH — fill this in; do not invent a different shape

```rust
// parser.rs — beside parse_all_with_file, which stays UNCHANGED and keeps calling lex()
pub fn parse_all_with_comments(
    src: &str,
    file: &str,
) -> Result<(Vec<WatAST>, Vec<Comment>), ParseError> {
    let file_arc = Arc::new(file.to_string());
    let (tokens, comments) = lex_with_comments(src, file_arc)?;
    let mut cursor = Cursor::new(&tokens);
    let mut out = Vec::new();
    while let Some(node) = cursor.parse_form()? { out.push(node); }
    Ok((out, comments))
}
```

```rust
// render.rs — a comment-aware peer of write_wat_source.
// Placement is arithmetic on spans (honest as of today), never stored ownership:
//   comment BEFORE the next form, on its own line  -> emit above that form, at its indent
//   comment on the SAME LINE as and after a form   -> emit trailing that form
//   comment CONTAINED in a form's extent           -> emit with that form's contents
// ★ A line comment PINS A NEWLINE after it. Nothing may follow it on its line.
pub(crate) fn write_wat_source_with_comments(
    forms: &[WatAST], comments: &[Comment], out: &mut String,
) { … }
```

## BLAST RADIUS

```
crates/wat-reader/src/parser.rs   ADD one pub fn. Change nothing existing.
src/edn/render.rs                 ADD one printer beside write_wat_source. Change nothing existing.
tests                             ADD the round-trip test described in EXPECTATIONS.
```

**No `WatAST` variant. No change to `lex`, `parse_all_with_file`, or `parse_one_with_file`. No new
intrinsic. No registry row. No `.wat` corpus edit.**

## STOP TRIGGERS

- **STOP-1 — if placing comments seems to require a `WatAST` variant, STOP.** The DESIGN pins
  "beside the tree, never in it" and gives three reasons. Surface what forced it; do not add the
  variant.
- **STOP-2 — if any existing test goes red, STOP.** This stone is purely additive. A red means an
  existing path changed, which the blast radius forbids. Capture the failing test's whole block
  verbatim and surface it; do not re-run first.
- **STOP-3 — if a real corpus file produces a comment whose placement is ambiguous under
  "above stays / within shifts", STOP and surface that exact case.** Do not invent a tie-break.
  The policy is the builder's and an ambiguity is a question for him, not a coin flip.
- **STOP-4 — if you find yourself changing how a NON-comment token prints, STOP.** Literal-spelling
  normalisation (`3.00`→`3.0`) is measured, known, and explicitly out of scope pending a ruling.

## PRIOR COMPARABLE — copy its shape

`[[BRIEF-STONE-the-reader-can-see-comments]]` and its `[[SCORE-STONE-the-reader-can-see-comments]]`.
Same arc, one layer down, same "new function beside an unchanged one" pattern, four hazards measured
and reported. That SCORE is the shape yours should take.

## THE FLOOR IS THE ORCHESTRATOR'S

Run what proves your change. `scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are
mine — but note that clippy caught a real red in the last stone (`items_after_test_module`, from a
test module placed mid-file), so **put new `mod tests` at the END of a file.**
