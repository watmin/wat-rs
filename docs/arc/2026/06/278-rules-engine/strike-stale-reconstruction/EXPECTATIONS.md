# EXPECTATIONS — C6

> ⛔ **INVARIANTS, NOT MILLISECONDS.** Absolute times in this arc are not reproducible to better than
> ~16% (C4's score) and the grid cannot resolve <20% (C8). Every row below states what must be TRUE.
> The two absolute numbers that DO appear are there because the claim *is* about their ratio, and
> both were driven by the orchestrator at HEAD.

## ⛔ NO PINNED TEST COUNT

Floor ≥ its current value, zero FAIL rows.

## The scorecard

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ no frozen filter constant | `FILTER_MS_MEASURED_IN_FIRE = 6.83` | gone; the value is read live in-test |
| 2 | ★ the reconstruction uses the native arm | uses `B` (`eval_test_core`) | uses `F` (`exec_where`) |
| 3 | ★ the declared check is asserted | `println!` only | a real assertion, message carries the whole table |
| 4 | the staleness is gone | frozen **6.83** vs live **0.38** — ~18x ⚠ (this row said 0.14/49x; wrong block of a three-size table) | the compared value is whatever the run measures |
| 5 | the headroom study survives | `A`,`B`,`D`,`E`,`B−A`,`D−A`,`B−E`,`B/F` present | all unchanged |
| 6 | ⚠ STOP-1 honoured | `F + C ≈ 2.7` vs live `~0.39` — ~7x ⚠ (this row said ~19x) | either the assertion passes on a band chosen from samples, **or** the rider stopped and reported. **A band widened to admit 19x is a FAILED strike.** |
| 7 | engine untouched | — | zero diff under `src/rete/kernel/fire/` |
| 8 | radius | — | `node_share_cost.rs` only |
| 9 | lints | 210/210 | green |
| 10 | clippy | rc=0 | silent |

## The mutation proofs

1. **Point the live read at `production` instead of `filter`** → assertion RED. Proves it reads the
   row it names.
2. **Scale `F` by 10 before the reconstruction** → assertion RED. Proves the band is narrow enough to
   catch a real change. *(Row 3 alone cannot distinguish an assertion from a vacuous one.)*
3. Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

40–60 minutes, unless STOP-1 fires — in which case the honest outcome is a report, and faster.

## What would make this strike a failure even if every test passes

**A band chosen so the assertion goes green.** The defect being removed is a check that was declared
and never enforced; a check enforced against a threshold picked to fit the current number is the same
defect wearing an assertion. The band must come from samples, stated in the comment beside it, and if
no honest band admits the measurement then STOP-1 is the correct outcome.

**And silently keeping `B` in the reconstruction** because it happens to land closer to the old
constant. `B/F = 3.75x` is printed in this very table: the two arms are known to differ, and the fire
runs `F`.
