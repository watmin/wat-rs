# BRIEF — STONE O: a lex error names the user's file, line and column (the-little-wat F-091)

**Drawn 2026-09-25.** First of the per-mechanism fixes for "errors that point into our source". A
census over the-little-wat's 364 probes found 154 of 166 errors already locating in the user's file;
the lexer accounts for 2 of the 12 misses, and F-091 records its real cost — finding which of 532
files raised `lex error at byte 258` meant grepping all 532 outside wat.

## The defect, driven

```
probes/name-lt.wat               → lex error at byte 258 … :file "crates/wat-reader/src/parser.rs" :line 201
probes/typer/unicode-in-source   → lex error at byte 153: unexpected character 'λ' … same parser.rs:201
```

## The root — one line, and the information already exists

`crates/wat-reader/src/parser.rs:~200`:
```rust
impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        ParseError { span: crate::rust_caller_span!(), kind: ParseErrorKind::Lex(e) }
    }
}
```
`LexError` (`crates/wat-reader/src/lexer.rs:190`) carries only a byte `position`. But
`lex_with_comments(src, file)` (`lexer.rs:345`) already holds the **file** (`Arc<String>`), a
**line-start table** (`compute_line_starts`, `:578`) and a **`line_col`** helper (`:590`) — it uses them
to build every token's span.

## The work

1. Give `LexError` a real `Span` (file, line, col; `end` = the same point or the offending character).
   Attach it ONCE at the lexer's return boundary, where the file and line table are in hand — NOT by
   editing all 43 `LexError { … }` construction sites.
2. `From<LexError> for ParseError` uses that span instead of `rust_caller_span!()`.
3. Keep the byte offset in the message if you like, but the `:location` must be the user's file.
4. Every caller of `lex`/`lex_with_comments` passes a real file today? Check; report any that pass a
   placeholder.

## Blast radius — measured

- ⛔ `tests/diagnostics/probe_ex003_diagnostic_locates_the_user.rs` — its `f091_*` test PINS THE DEFECT
  (asserts the error still names a `.rs` file). It WILL go red: that is the signal. Flip it to the
  control's shape (asserts no `.rs` `:file`), as stone B did for F-006.
- `tests/value/wat_arc220_char__char_literal_supplementary_plane_rejected.edn` pins the old location —
  recapture with `UPDATE_EDN=1`, and check the new golden names the fixture file.
- `tests/lint/no_new_broken_doc_link.rs` names `parser.rs` — check whether it is a location assertion
  or just a doc link.

## Prove it

- Both probes above: `:location` names the user's `.wat`, with the right line and column.
- The flipped `f091` test green; the control test still green.
- **Mutation:** restore `rust_caller_span!()` in the `From` → the flipped test reds.
- Floor 0 failed; clippy clean.

## STOP triggers

1. A caller of the lexer has no real file to give — report it rather than inventing one.
2. Any gate reddens that you did not add (other than the named `f091` flip and golden) — capture
   whole, name the arm. ⛔ Do not re-run first.

## Mechanics — ⛔ read

- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks (each under the tool's 600s
  cap). Do not end your turn while it runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines, never
  piped exit codes.
- No `cargo fmt` / `rustfmt`. Stage explicit paths, never `git add -A`.
- Test hygiene lints that bit every recent stone: an EDN string literal in a test trips
  `no_inlined_edn` (use an `.edn` golden + `assert_edn_matches_file!`); a `contains`/`ends_with` in an
  assert trips `no_loose_string_assert`.
