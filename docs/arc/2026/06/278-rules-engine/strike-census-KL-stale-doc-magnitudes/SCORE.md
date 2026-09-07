# SCORE — two `census.rs` comments now say what the code does

Doc-only. Nothing was at risk. The 2026-08-01 measurement is kept and dated; today's shape is stated. The audit's L was half wrong and is amended. Floor GREEN. No mutation — these are comments.

## Scorecard

| # | result |
|---|---|
| 1 ★ K amended, measurement kept | **HOLD.** 2026-08-01 reading marked historical and named as why the five per-fact children were removed. Today: two marks, once per pass. Pair-count column and its argument survive. |
| 2 ★ L scoped correctly | **HOLD.** Zero-row claim held for STEP2 and CATCHUP; denied for MAINTAINER, with its `if already < right_elements.len()` guard as the reason. |
| 3 ★ the audit corrected | **HOLD.** The audit's *"neither can ever emit a 0 row"* was half wrong. CATCHUP's `n` is `n_all.saturating_sub(already)` — zero whenever `n_all == already`. Quoted below. |
| 4 ★ evidence quoted | **HOLD.** Two `alpha:*` marks, line numbers, each site's `n`. |
| 5 comments only | **HOLD.** `git diff` is `census.rs` `///` lines only. |
| 6 severity honest | **HOLD.** Doc-only. No number wrong. No mutation invented. |
| 7 floor | **HOLD.** `5465 passed, 21 skipped` — same count. No new tests. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 3 is the one worth the strike**: a work-list row that was checked and came back partly wrong gets amended, not quietly worked around.

## Row 4 — K evidence

`grep 'phase_end("[^"]*alpha' src/rete/kernel/fire/pass/alpha.rs`:

```
312:    phase_end("  ├ alpha:seed", __seed);
421:    phase_end("  └ alpha:delta", __delta);
```

Two marks. Both at the end of a pass function. Once per pass, not per fact.

## Row 4 — L evidence

| site | `n` | can emit 0? |
|---|---|---|
| `hash_join.rs:350` STEP2 | `dr.iter().count()` | **yes** — outside the loop, its own comment says so |
| `hash_join.rs:216` CATCHUP | `n_all.saturating_sub(already)` | **yes** — zero whenever `n_all == already` |
| `fire/mod.rs:928` MAINTAINER | `right_elements.len() - already` inside `if already < right_elements.len()` | **no** — the guard is "there is something to append" |

The audit said neither CATCHUP nor MAINTAINER can emit 0. CATCHUP can. That half is false.

## Mutation

None. A comment change cannot red an instrument. Saying so is the report.

## Final floor

`.floor/2026-09-07T00-47-44Z/`: `Summary [ 460.978s] 5465 tests run: 5465 passed (2 slow), 21 skipped`. Count unchanged. No new tests.
