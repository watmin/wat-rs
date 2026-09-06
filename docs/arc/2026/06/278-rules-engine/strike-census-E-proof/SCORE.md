# SCORE — census E stays unproven; the counters are gated anyway

STOP-1 fired. Reverting E's move stayed GREEN. The delete-bump control REDs, so the test is watching the counter and the corpus simply has no non-alpha `cond`. Three assertions kept. `matcher.rs` byte-identical to HEAD. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ the mutation REDs | **STOP-1.** Bump put back below `alpha_pattern`'s `?`: `match:calls = 10000` / `interp_calls = 10000`, **PASS**. Quoted below. No facts added. |
| 2 ★ restored and green | **HOLD for the control's restore.** `git diff --quiet src/rete/matcher.rs`. Passing numbers quoted below. |
| 3 ★ all three assertions present | **HOLD.** `match:calls == interp_calls`, `compiled:exec == calls`, `calls == interp_calls`. |
| 4 ★ the numbers are real | **HOLD.** `calls = 10000`, `interp_calls = 10000`, `match:calls = 10000`, `compiled:exec = 10000`. |
| 5 doc row added | **HOLD**, and then **rewritten** after STOP-1 so it does not claim the move-revert must RED. It now says liveness is gated and this corpus cannot prove E's widening. |
| 6 nothing else touched | **HOLD.** `alpha_discrimination.rs` only at the end. `matcher.rs` == HEAD. |
| 7 floor | **HOLD.** `5465 passed, 21 skipped` — same count. No new test; row 3 is inside the existing one. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 1 is the entire strike as drawn, and it did not land.** The assertions close a different hole: these counters were ungated. Census E's widening remains unproven.

## Row 1 — E's move reverted: GREEN, not RED

`compiled_cond_failure_path_allocates_no_binding_keys_at_50_100`, `--release --no-capture`, bump **below** the `?` (the pre-E defect):

```
  ROW 3 — call-counter differential, same corpus
  match:calls    = 10000  (hand interp_calls = 10000)
  compiled:exec  = 10000  (hand calls = 10000)

test …compiled_cond_failure_path_allocates_no_binding_keys_at_50_100 ... ok
```

The DESIGN's trap: every `cond` in this corpus is an alpha pattern, so the `?` never returns early. `match:calls == interp_calls` is identical before and after census E's move. The assertion cannot prove E.

## The control — delete the bump: RED

This is what distinguishes "mutation real but inert" from "mutation never reached the binary."

Bump deleted. Same test, same binary path:

```
  ROW 3 — call-counter differential, same corpus
  match:calls    = 0  (hand interp_calls = 10000)
  compiled:exec  = 10000  (hand calls = 10000)

assertion `left == right` failed: match:calls is 0 but the loop made 10000 calls — the counter misses invocations (census E moved it above `alpha_pattern`'s `?` so it would not)
  left: 0
 right: 10000
```

The test observes `match:calls`. STOP-1 is a corpus finding, not a dead gate. Restored: `git checkout HEAD -- src/rete/matcher.rs`.

## Restored passing numbers

```
  ROW 3 — call-counter differential, same corpus
  match:calls    = 10000  (hand interp_calls = 10000)
  compiled:exec  = 10000  (hand calls = 10000)
```

`calls == interp_calls == 10000`. Same corpus. Both counters live.

## What this did not do

Did not add a non-alpha `cond` to force a red. That is the next strike's corpus decision. Census E (`c91121e5e`) stays landed and stays unproven.

## Final floor

`.floor/2026-09-06T21-20-28Z/`: `Summary [ 461.327s] 5465 tests run: 5465 passed (2 slow), 21 skipped`.
