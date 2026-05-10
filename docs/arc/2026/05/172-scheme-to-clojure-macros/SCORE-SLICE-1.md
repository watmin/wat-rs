# Arc 172 slice 1 — SCORE

**Result:** Mode A clean (substrate edit; workspace RED by design).
**Runtime:** ~10 min sonnet (under predicted 30-60 band).
**Files:** 2 modified (`src/lexer.rs` + `src/parser.rs`).

## Calibration

- **Predicted runtime:** 30-60 min sonnet
- **Actual:** ~10 min — well under band
- **Why fast:** the lexer edit was small (one comma block retired,
  one tilde block minted, comma added to whitespace handling +
  symbol-break set). Parser test migrations mechanical.

## What shipped (working tree only; atomic-commit pair with slice 2)

### src/lexer.rs
- Comma at lex-loop level retired as Unquote / UnquoteSplicing
  token; now treated as EDN whitespace
- `~` and `~@` mint as new Unquote / UnquoteSplicing tokens
- `is_symbol_break` includes `,` so symbols terminate at comma
- Test cases added covering tilde unquote, splice, comma-as-
  whitespace, comma-as-list-separator
- Arc 171's `lex_keyword` reject rule for keyword-body commas
  unchanged (still active)

### src/parser.rs
- Existing quasiquote tests using `,foo` literal text migrated
  to `~foo`
- `Token::Unquote` / `Token::UnquoteSplicing` variant names
  unchanged — only their source-level character changed

## Workspace impact (RED by design)

- Pre-slice baseline: 1334 passed / 854 failed
- Post-slice-1: 453 passed / 938 failed (RED — macro consumers
  using `,name` no longer parse)
- 797 tests didn't run (stdlib registration crashed cascading
  failures across test binaries)

This was the EXPECTED intermediate state. Slice 2 sweeps the
~22 consumer files; workspace returns to baseline.

## What's next

Slice 2 shipped (~45 min sonnet) — workspace returned to
1339/854 (+5 from previously-broken macros.rs unit tests now
fixed; 0 fails delta).

Atomic-commit pair: slices 1+2 commit together with INSCRIPTION
+ 058 row + memory update (this turn).

## Cross-references

- BRIEF: [`BRIEF-SLICE-1.md`](./BRIEF-SLICE-1.md)
- EXPECTATIONS: [`EXPECTATIONS-SLICE-1.md`](./EXPECTATIONS-SLICE-1.md)
- Sibling: [`SCORE-SLICE-2.md`](./SCORE-SLICE-2.md) — the
  consumer sweep that closes the RED state
