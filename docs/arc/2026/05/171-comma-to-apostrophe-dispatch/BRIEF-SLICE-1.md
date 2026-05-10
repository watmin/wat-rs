# Arc 171 slice 1 — BRIEF

**Lexer change; sonnet.** Make `lex_keyword` accept `'`
(apostrophe) as a keyword-body character. Purely additive —
existing `,N` keyword bodies continue to parse during the
transition. Slice 2 sweeps consumers; slice 3 retires comma
acceptance and ships closure.

## Mission

Edit `src/lexer.rs::lex_keyword` to accept `'` inside keyword
bodies with the same position-aware treatment commas currently
have. After this slice:

- `:wat::core::op'2` parses as a single keyword (analogous to
  `:wat::core::op,2` today)
- `:wat::core::op'i64'i64` parses as a single keyword
  (multi-apostrophe; analogous to `:wat::core::op,i64-i64`
  today — the user's locked table uses apostrophe for ALL
  internal separators, including between type names where
  commas previously held a dash)
- Existing `,N` and `,xxx-yyy` keyword bodies KEEP working
  (transition mode)

## Substrate edit

File: `src/lexer.rs`

Find `lex_keyword` (around line 389, per pre-grep). The
function walks keyword bytes and pushes accepted characters
onto an output buffer. Find the section that handles "every
other character pushed as-is" — currently around the comment
at line 387:
```
/// Every other character (including `<`, `>`, `/`, `-`, `,`, `!`, `?`)
/// is pushed as-is. Whitespace inside an unclosed `(` is an error.
```

Add `'` to the accepted set explicitly OR confirm that the
fall-through arm already pushes it (sonnet investigate: is
apostrophe currently in the "pushed as-is" set, or does it
break the loop?). If apostrophe is currently a TERMINATOR,
that's the bug to fix; if it's already in the as-is set,
update the doc comment to name it.

## Tests to add

Add to `src/lexer.rs`'s test module (find existing
`lex_keyword` tests; mirror the shape):

| Test | Input | Expected |
|------|-------|----------|
| A — arity suffix | `:wat::core::op'2` | one Token::Keyword with body `:wat::core::op'2` |
| B — multi-discriminator | `:wat::core::op'i64'i64` | one Token::Keyword with body `:wat::core::op'i64'i64` |
| C — full op table | `:wat::core::op'f64'i64` | parses as single keyword |
| D — apostrophe inside parametric brackets | `:HashMap<i64,String>'snapshot` | parses as single keyword (apostrophe outside `<>` works the same as commas-outside) |
| E — comma still works (transition) | `:wat::core::op,2` | parses as single keyword (existing behavior preserved) |
| F — apostrophe-only no body | `:'foo` | (whatever current behavior; surface as honest delta if it's now ambiguous with quote-shorthand) |

## What to NOT do

- **No consumer sweep.** Don't migrate any `,N` to `'N` in
  this slice. That's slice 2.
- **No comma rejection.** Don't add the diagnostic for `,`
  inside keyword body. That's slice 3.
- **No parser changes.** Apostrophe is purely lexical; the
  parser sees the keyword as a single token.

## Substrate-grep citations

- `src/lexer.rs:387-389` — doc comment + start of `lex_keyword`
- `src/lexer.rs:234` — current comma handling OUTSIDE keyword
  bodies (Unquote / UnquoteSplicing tokens; this slice does
  NOT touch this)
- existing tests in `src/lexer.rs::tests` (find by grep
  `mod tests` or `#[test]`)

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A — `:wat::core::op'2` parses as single keyword | new test passes | ✓ |
| B — `:wat::core::op'i64'i64` parses as single keyword | new test passes | ✓ |
| C — `:wat::core::op'f64'i64` parses as single keyword | new test passes | ✓ |
| D — `:HashMap<i64,String>'snapshot` parses as single keyword | new test passes (apostrophe after `>` works) | ✓ |
| E — `:wat::core::op,2` still parses (transition) | existing comma-suffix tests still green | ✓ |
| F — `cargo check --release` green | no compile errors | ✓ |
| G — Workspace cargo test stays at baseline (1328/854) | no regressions | ✓ |
| H — Only `src/lexer.rs` modified | `git diff --stat` shows one file | ✓ |
| I — Honest deltas surfaced | per FM 5 | ✓ |

## Honest delta categories

- **Apostrophe ambiguity with quote shorthand** — if `'foo`
  outside a keyword position is the Clojure quote shorthand,
  position-awareness needs to distinguish. Inside `lex_keyword`
  (we entered after `:`), the `'` is just keyword-body content.
  Outside, the lexer's main loop sees `'` separately. Surface
  any friction.
- **Existing tests fail** — if any test asserts that
  apostrophe DOESN'T parse inside a keyword body, surface; that
  test gets retired or updated.
- **`'` already accepted** — if pre-grep shows the lex_keyword
  fall-through already pushes apostrophe as-is, no functional
  change needed; the slice just adds the test cases + updates
  the doc-comment.

## Predicted runtime

30-45 min sonnet. Single function edit + 5-6 test cases.

**Hard cap:** 90 min. Wakeup scheduled.

## Reference

- DESIGN.md (this arc; the broader scope)
- `feedback_apostrophe_dispatch_separator.md` (the user's
  locked convention)
- `src/lexer.rs:387-389` + `:234` (the edit + non-edit sites)
