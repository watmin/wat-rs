# SCORE — census E's population was never observed across the suite

Zero lines. The tripwire file was never created. A `cond` that `alpha_pattern` rejects was never observed across the full suite — not "cannot happen". One comment sentence at the site. Tripwire gone. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ the number | **HOLD.** `wc -l /tmp/arc278-alpha-pattern-none.log` — file absent, **0 lines**. Either value is a PASS. |
| 2 ★ floor GREEN with the tripwire in | **HOLD.** `.floor/2026-09-06T21-44-01Z/`: `Summary [ 460.637s] 5465 tests run: 5465 passed (3 slow), 21 skipped`. |
| 3 ★ `matcher.rs` clean at the end | **HOLD**, modulo the one comment sentence. `git diff` is that sentence only. No `TRIPWIRE` remains. |
| 4 ★ the wording, if zero | **HOLD.** Site comment: *"never observed across the full suite"*. Dated 2026-09-06, floor `2026-09-06T21-44-01Z` (5465 passed). Does not say "cannot happen". |
| 5 if non-zero, verbatim | **N/A.** Zero. |
| 6 no corpus or test change | **HOLD.** Nothing added to make a case exist. |
| 7 floor after revert | **HOLD.** `.floor/2026-09-06T21-58-16Z/`: `Summary [ 460.614s] 5465 tests run: 5465 passed (3 slow), 21 skipped`. Count unchanged. Comment only. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 1 has no expected value on purpose.** Zero is the measurement, not a failure of the strike.

## The number

```
ls: cannot access '/tmp/arc278-alpha-pattern-none.log': No such file or directory
FILE ABSENT — 0 lines
```

The `None` branch of `alpha_match_inner_opts` never ran. `alpha_pattern` returns `None` for a non-List, an empty List, or a Symbol head that is not a `FactBind`. Keyword-headed forms (`:wat::rete::not`, `:wat::rete::exists`, …) return `Some` and fail the later head check instead — that is why the branch can be this empty and still be live as a guard.

One tripwire covered all three wrappers (`alpha_match_inner`, `_local`, `_seeded`).

## What this did not do

Did not add a non-alpha `cond`. Did not delete or `unreachable!()` the `None` branch. Did not interpret callers (there were none). Census E stays landed; its extra population remains empty everywhere this suite can see.

## Final floor (after revert)

`.floor/2026-09-06T21-58-16Z/`: `Summary [ 460.614s] 5465 tests run: 5465 passed (3 slow), 21 skipped`.
