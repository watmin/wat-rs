# Arc 171 slice 1 — SCORE

**Result:** Mode A clean.
**Runtime:** ~8 min sonnet (well under predicted 30-45; far under 90 hard cap).
**Files:** 1 modified (`src/lexer.rs` — 79 ins / 2 del; doc comment + 6 new tests).

## Calibration

- **Predicted runtime band:** 30-45 min sonnet
- **Actual:** ~8 min — far under band
- **Why:** the lexer's fall-through arm already accepted `'`
  (any non-matched character pushes as-is). No functional
  change required; the slice's value was test coverage + doc
  comment naming the convention. Sonnet correctly diagnosed
  this and shipped lock-in tests rather than redundant code.

## Scorecard (9 rows)

| Row | What | Result |
|-----|------|--------|
| A — `:wat::core::op'2` parses as single keyword | ✓ test `keyword_apostrophe_arity_suffix` |
| B — `:wat::core::op'i64'i64` parses as single keyword | ✓ test `keyword_apostrophe_multi_discriminator` |
| C — full op table (all 4 type-discriminator variants) | ✓ test `keyword_apostrophe_full_op_table` |
| D — `:HashMap<i64,String>'snapshot` parses as single keyword | ✓ test `keyword_apostrophe_after_parametric_close` |
| E — `:wat::core::op,2` still parses (transition) | ✓ test `keyword_comma_suffix_transition` |
| F — `cargo check --release` green | ✓ clean (1 pre-existing warning unrelated) |
| G — Workspace at baseline + new tests | ✓ 1334/854 (+6 from 1328/854; zero regression) |
| H — Only `src/lexer.rs` modified | ✓ git diff --stat confirms |
| I — Honest deltas surfaced | ✓ 2 (no-functional-change + no-quote-collision) |

**9/9 rows pass.** Mode A clean.

## Honest deltas

### 1. Apostrophe was already accepted — no functional change needed

`lex_keyword`'s fall-through arm (`_ => out.push(c)` at line
506) already pushes any character not matched by an explicit
case. Apostrophe wasn't in any explicit match arm, wasn't in
`is_symbol_break`, wasn't handled before `lex_keyword` in the
main lex loop. The slice's value was:
- Test cases that pin the apostrophe-acceptance behavior
- Doc comment naming `'` explicitly alongside `,` (the
  transition-mode rule)
- Lock-in via tests so future lexer refactors can't silently
  break apostrophe-in-keyword-body

This is the cleanest possible "slice ship" — the substrate
already had the right shape; the slice surfaced + documented +
test-locked the contract.

### 2. No quote-shorthand collision

The main lex loop has no special `'` dispatch. Inside
`lex_keyword` (entered after `:` is consumed), `'` is purely
keyword-body content. Outside keyword position, `'` falls
through to `lex_symbol` — would become part of a bare symbol,
not a Clojure-style quote reader-macro.

Test F (`:'foo` → `Token::Keyword(":'foo")`) confirms and
documents this. **Future arc 172** introduces Clojure-style
quote shorthand `'foo`; the apostrophe-after-`:` case is
unambiguously keyword-body (no collision with the future
quote token).

## Calibration row

- **Actual runtime:** ~8 min (Mode A clean — far under
  predicted band)
- **Workspace post-slice:** 1334 passed / 854 failed
- **Pass-count delta from baseline:** +6 (the new tests)
- **Fail-count delta:** 0 (zero regressions)
- **Honest deltas surfaced:** 2 (both: "no functional change
  needed; lock-in via tests")
- **Sonnet model validated:** mechanical lexer-investigation
  + test-authoring → ~8 min execution. Far under any
  predicted band. Sonnet's wheelhouse confirmed.

## What's next

1. **Commit slice 1 atomically** (this turn) — `src/lexer.rs`
   change + this SCORE doc.
2. **Author slice 2 BRIEF + EXPECTATIONS** — the consumer
   sweep across all 167 sites. Mechanical pattern:
   - `,N` (arity) → `'N`
   - `,xxx-yyy` (type-discriminator) → `'xxx'yyy` per user's
     locked table (each `-` between type names also becomes `'`)
   - All ~80 wat source files + ~10 Rust diagnostic-string
     files
3. **Spawn slice 2 sonnet** — mechanical migration; should be
   fast.

## Cross-references

- BRIEF: [`BRIEF-SLICE-1.md`](./BRIEF-SLICE-1.md)
- DESIGN: [`DESIGN.md`](./DESIGN.md)
- Memory: `feedback_apostrophe_dispatch_separator.md` (user's
  locked convention)
- Sibling: arc 170 slice 1f-W (`4278c4d`) — EDN-compliance
  predecessor work
- Successor: arc 172 — Scheme → Clojure macro flavor swap;
  ships AFTER arc 171 closes
