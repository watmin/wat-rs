# Arc 172 slice 1 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 30-60 minutes sonnet.**

Small lexer edit (one comma-block retired, one tilde-block
added) + lexer test migration (a few existing tests that use
`,` literal in inputs swap to `~`). Pattern fully specified
by BRIEF.

Comparable to:
- Arc 171 slice 1 (~8 min sonnet) — similar shape but smaller
  scope (apostrophe was already accepted; this slice adds a
  net-new lexer rule)
- Arc 171 slice 3 (predicted 30-45 min) — similar shape (single
  lexer rule + test migration)

**Hard cap: 90 minutes.** Wakeup scheduled.

**Workspace IS RED by design.** This slice intentionally
breaks macro-using tests — comma in macro bodies no longer
parses as unquote. Slice 2 (consumer sweep) closes the gap;
the two slices commit together as an atomic pair.

## Baseline (post-arc-171 slice 3 — TBD commit)

Arc 171 slice 3 lands BEFORE this slice. Post-arc-171 baseline:
- Workspace: ~1334 passed / ~854 failed (slice 3 retires one
  transition test, replaces with rejection test; net ±0)
- Lexer: rejects `,` inside keyword body; canonical apostrophe
  for dispatch suffix; comma still produces Unquote token at
  main lex loop (THIS slice retires that)

Predicted post-slice-172-1:
- Lexer tests: green (the 5 new tests + any migrated old tests)
- Workspace: RED — macro-using tests fail (855+ new failures
  from `,name` in macros no longer parsing). EXPECTED per BRIEF.
- Workspace fail count target: report the actual number; slice 2
  fixes them all.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — Comma block at lexer.rs:234 retired | grep no longer finds `if c == ','` in the MAIN lex loop (the `lex_keyword` body check from arc 171 slice 3 is a SEPARATE block; keep that) | ✓ |
| B — `~` produces `Token::Unquote` | new test passes | ✓ |
| C — `~@` produces `Token::UnquoteSplicing` | new test passes | ✓ |
| D — Comma is whitespace at top-level | new test passes (`(a , b)` parses as `(a b)`) | ✓ |
| E — Comma is whitespace inside list | new test passes (`(a, b, c)` parses as `(a b c)`) | ✓ |
| F — `Token::Unquote` / `Token::UnquoteSplicing` variants unchanged | parser dispatch still works without renames | ✓ |
| G — `cargo check --release` green | the lexer + parser compile cleanly | ✓ |
| H — Lexer-only tests green | `cargo test --release --test wat_lexer` (or unit tests if inline) pass | ✓ |
| I — Workspace IS RED (expected) | macro-using tests fail because `,name` in macros no longer parses as unquote — this is EXPECTED for slice 1; slice 2 fixes | ✓ (per BRIEF) |
| J — Workspace fail count reported | sonnet reports the new fail count so slice 2 has a target | ✓ |
| K — Only `src/lexer.rs` + maybe `src/parser.rs` tests modified | `git diff --stat` shows ≤ 2 files | ✓ |
| L — Zero new dependencies | Cargo.toml unchanged | ✓ |
| M — Honest deltas surfaced | per FM 5 | ✓ |

## Honest delta categories

- **Lexer-internal tests asserting old comma behavior** —
  surface which retire vs migrate
- **`,@` vs `~@` lookahead edge case** — same edge handling
  needed for `~@`
- **Backtick-comma grammar in `src/parser.rs:669` test** —
  the `quasiquote_with_unquote_splicing_inside` test uses
  `,@` literal text; update to `~@`
- **Doc comments** — any doc mentioning `,foo` unquote
  shorthand updates to `~foo`
- **Macro-using consumer tests** — surface the count of
  newly-failing tests (for slice 2's target)

## Calibration row

- Actual runtime: ___ min (Mode A clean / B partial / C failed)
- Workspace post-1f-0a: ___ passed / ___ failed (RED expected)
- Lexer test count delta: ___ (predicted: +5 new; some
  retired/migrated)
- Honest deltas surfaced: ___

## What's next (orchestrator-side, post-slice-1)

When slice 1 ships:

1. Verify lexer tests pass + workspace is red as expected
2. Author SCORE-SLICE-1.md
3. DO NOT atomic-commit yet — wait for slice 2 to ship
   (atomic-commit pair)
4. Author slice 2 BRIEF + EXPECTATIONS — wat-source consumer
   sweep (~30 files; `,name` → `~name`, `,@list` → `~@list`)
5. Spawn slice 2 sonnet (against the dirty tree from slice 1)
6. When slice 2 ships and workspace is green again, ATOMIC
   COMMIT slices 1 and 2 together

## Sonnet-delegation pre-flight (recovery doc § 7)

- [x] DESIGN.md current (passes 1-18 of arc 170 + arc 172
      DESIGN itself)
- [x] BRIEF + EXPECTATIONS authored + will-be-committed
- [x] Runtime band: 30-60 min predicted; 90 hard cap
- [x] Substrate-grep citations in BRIEF point at exact files
- [x] Verified each cited primitive exists (`src/lexer.rs:234`
      comma block; `src/parser.rs:239-241` reader-macro
      dispatch)
- [x] "Workspace will be RED" is EXPECTED per BRIEF; sonnet
      should not panic
- [x] Will spawn with `model: "sonnet"` explicitly
- [x] Will spawn with `run_in_background: true`
- [x] **Will spawn AFTER arc 171 slice 3 lands to avoid file
      collision**
- [x] Wakeup scheduled at 90 min hard cap

## SCORE artifact

Slice 1 of arc 172. SCORE-SLICE-1.md lands beside this when
slice ships AND atomic-commit pair with slice 2 lands.
