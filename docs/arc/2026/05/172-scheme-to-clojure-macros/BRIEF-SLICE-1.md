# Arc 172 slice 1 — BRIEF

**Lexer change; sonnet.** Swap quasiquote token-producing
characters: retire `,` and `,@` as Unquote/UnquoteSplicing
tokens (comma becomes pure whitespace per EDN spec); mint `~`
and `~@` as the new tokens.

This slice **intentionally breaks the workspace** — existing
macro bodies in `wat/*.wat` use `,name` / `,@list`, which after
this slice will be parsed as `name` (bare symbol, comma-as-
whitespace) and `@list` (parse error or different shape). Slice
2 sweeps the consumers; the two slices commit together as an
atomic pair when the workspace is green per recovery doc §
"Atomic commit across coordinated sweeps."

## Sequencing — wait for arc 171 slice 3

This slice touches `src/lexer.rs`. Arc 171 slice 3 is currently
in flight editing the same file (rejecting `,` inside keyword
bodies). **Spawn this slice ONLY AFTER arc 171 slice 3 ships
to disk** (its atomic commit lands first; then this slice's
delta against the post-slice-3 state).

## Mission

### 1. Retire comma as Unquote/UnquoteSplicing token

Current: `src/lexer.rs:234`
```rust
if c == ',' {
    let s = span_at(i);
    if i + 1 < bytes.len() && bytes[i + 1] as char == '@' {
        tokens.push(SpannedToken { token: Token::UnquoteSplicing, span: s });
        i += 2;
    } else {
        tokens.push(SpannedToken { token: Token::Unquote, span: s });
        i += 1;
    }
    continue;
}
```

After this slice: that whole block is gone. Comma at the main
lex-loop level becomes whitespace per EDN spec. The comma
handling INSIDE `lex_keyword` (paren-depth + angle-depth
tracker; arc 171's reject rule) stays unchanged — commas
inside `(...)` or `<...>` keyword bodies still mean what
they meant.

Add to the lexer's whitespace handling: treat `,` as
whitespace at the main loop. Same shape as `'\t'` / `' '` /
`'\n'`.

### 2. Mint `~` and `~@` as Unquote / UnquoteSplicing tokens

New block (likely placed where the old comma block was):
```rust
if c == '~' {
    let s = span_at(i);
    if i + 1 < bytes.len() && bytes[i + 1] as char == '@' {
        tokens.push(SpannedToken { token: Token::UnquoteSplicing, span: s });
        i += 2;
    } else {
        tokens.push(SpannedToken { token: Token::Unquote, span: s });
        i += 1;
    }
    continue;
}
```

The `Token::Unquote` and `Token::UnquoteSplicing` variants
KEEP their existing names — only their source-level
character changes from `,`/`,@` to `~`/`~@`. The parser dispatch
to `:wat::core::unquote` / `:wat::core::unquote-splicing`
reader macros is unchanged.

### 3. Tests

Mirror the existing quasiquote test shape in `src/parser.rs`
(grep for `Token::Quasiquote` or `unquote_wraps_following_form`).

| Test | Input | Expected |
|------|-------|----------|
| A — `~foo` unquote | `\`(a ~b c)` | parses as quasiquote-of-list-with-(unquote b) |
| B — `~@xs` splice | `\`(a ~@xs c)` | parses as quasiquote-of-list-with-(unquote-splicing xs) |
| C — comma is whitespace (top-level) | `(a , b)` | parses as `(a b)` (comma between elements is whitespace) |
| D — comma is whitespace inside list | `(a, b, c)` | parses as `(a b c)` |
| E — old comma-unquote SHOULD NOT WORK | `\`(a ,b c)` | parses as quasiquote-of-list-with-bare-symbols `a`, `b`, `c` (the comma is whitespace; `,b` is just `b`) |

Existing tests that assert `,foo` → unquote token MUST be
updated to use `~foo`. Surface these test changes as part of
the slice; don't leave any test asserting the retired comma
behavior.

## What to NOT do

- **No consumer sweep.** Don't migrate any `,name` to `~name`
  in wat sources. That's slice 2.
- **No new Clojure features (auto-gensym, `&form`/`&env`,
  `gensym` primitive).** Slice 3.
- **No INSCRIPTION authoring.** Orchestrator handles at atomic
  commit time.
- **No macro evaluator changes.** Tokens rename only; macro
  semantics (unquote / splicing / nesting per arc 029) stay
  identical.
- **No other Rust files modified beyond `src/lexer.rs` +
  `src/parser.rs` (if existing tests live there)**

## Substrate-grep citations

- `src/lexer.rs:234` — current comma-as-Unquote handling (to
  retire)
- `src/lexer.rs:229-232` — Quasiquote backtick handling (keep
  unchanged; the backtick stays as quasiquote)
- `src/parser.rs:239-241` — Token → reader-macro dispatch
  (unchanged; `Token::Unquote` and `Token::UnquoteSplicing`
  variant names keep working)
- `src/parser.rs::tests` (around lines 618-680) — existing
  quasiquote tests (update any that use `,` literal characters)
- `src/lexer.rs::tests` — find existing comma/unquote tests
  and either retire or migrate

## Test approach during slice

Phase the work:
1. Update `src/lexer.rs:234` block (retire comma; add tilde)
2. Run `cargo check --release` — verify lexer compiles
3. Run `cargo test --release --test wat_lexer` — many will
   fail (macro-using tests). EXPECTED.
4. Update the lexer's own tests (the 5 new + retire/update
   any old comma-unquote tests)
5. Re-run `cargo test --release --test wat_lexer` — lexer
   tests should pass
6. `cargo test --release --workspace` — workspace will be
   RED (855+ failures expected; slice 2's job to fix)

The slice ships with workspace red; slice 2 sweeps and commits
together.

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A — Comma block at lexer.rs:234 retired | grep no longer finds `if c == ','` in main lex loop (still in `lex_keyword` for the keyword-body-rejection per arc 171) | ✓ |
| B — `~` and `~@` produce Unquote / UnquoteSplicing | new test cases pass | ✓ |
| C — Comma is whitespace (top-level) | new test passes | ✓ |
| D — `Token::Unquote` / `Token::UnquoteSplicing` variant names unchanged | parser dispatch in `parser.rs:239-241` works without rename | ✓ |
| E — Lexer-only tests pass | `cargo test --release --test wat_lexer` (or the equivalent unit-test bin if lexer tests are inline) green | ✓ |
| F — Workspace IS RED (expected) | macro-using tests fail because `,name` no longer parses as unquote. This is EXPECTED for slice 1; slice 2 fixes | ✓ (slice 2 closes) |
| G — `cargo check --release` green | the lexer + parser compile cleanly | ✓ |
| H — Only `src/lexer.rs` + maybe `src/parser.rs` tests modified | `git diff --stat` shows ≤ 2 files | ✓ |
| I — Zero new dependencies | Cargo.toml unchanged | ✓ |
| J — Honest deltas surfaced | per FM 5 | ✓ |

## Honest delta categories

- **Lexer-internal tests that assert old comma behavior** —
  surface which ones get retired vs migrated to `~`. Some may
  be testing lexer-error cases that still apply (e.g., `,foo`
  inside a particular context).
- **`,@` lookahead edge case** — `,@` at end-of-input or just
  before whitespace. New `~@` shape needs same edge handling.
- **Backtick-comma vs backtick-tilde grammar** — the
  quasiquote parser test
  `quasiquote_with_unquote_splicing_inside` at
  `src/parser.rs:669` probably uses `,@` literal text in the
  test input. Update to `~@`.
- **Doc comments in `src/lexer.rs` / `src/parser.rs`** — any
  doc comment mentioning `,foo` as unquote shorthand updates
  to `~foo`.

## Predicted runtime

30-60 min sonnet. The lexer edit is small (one block retired,
one block added); the test migration is mechanical (`,` →
`~` in test inputs).

**Hard cap:** 90 min.

## Reference

- DESIGN.md (this arc; the broader scope including slice 2
  sweep + slice 3 Clojure features + slice 4 closure)
- Arc 170 REALIZATIONS pass 14 — the decision that opened
  this arc
- Arc 010 (variadic-quote) — quasiquote primitive precedent
- Arc 029 (nested-quasiquote) — depth-tracking semantics this
  slice preserves
- `src/lexer.rs:228-245` — current quasiquote / unquote /
  splice lexer block
- `src/parser.rs:239-241` — reader-macro dispatch (unchanged)
