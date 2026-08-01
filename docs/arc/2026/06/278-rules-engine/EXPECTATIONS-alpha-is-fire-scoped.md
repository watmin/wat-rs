# EXPECTATIONS — alpha is fire-scoped (written BEFORE the strike)

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the new gate passes all four | `cargo nextest run --release -E 'test(/alpha_is_fire_scoped/)'` | green; native alpha keys 0, oracle alpha keys > 0, derived counts equal and > 0 |
| 2 | 2b re-pointed, still asserting | `cargo nextest run --release -E 'test(/2b_insert_alpha/)'` | green, via `fire-rules-spec` |
| 3 | the RESULT did not move | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all pass — **this is the load-bearing row** |
| 4 | the floor | `cargo nextest run --release` | 4208 + 2 new = **4210/4210** |
| 5 | no new lint debt | `cargo clippy --all-targets --release` | silent |
| 6 | the win, measured | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | `out:alpha` collapses from ~52.7 ms to ~0; `OUT: to_persistent` from ~31.5% to <1% |
| 7 | the grid | `bash wat-scripts/perf/grid/run-axis.sh accum "[200 200]"` | `accum` ratio improves; **`:accuracy :match` unchanged** |

## Independent prediction

- **Runtime:** 15–25 min. Two one-line edits plus a new probe pair; the only real work is authoring the
  gate's four entries in wat.
- **Diff size:** ~+80 / −10 lines, almost all of it the new probe pair.
- **The win:** fire at `G=200 W=200` should fall from ~168 ms toward **~116 ms (−31%)**. Accum's marginal
  rate should move from 4.29 µs/fact toward **~3.0**, against Clara's 1.78. **This narrows the one axis we
  lose; it does not flip it** — say so plainly in the score rather than reporting a flip.

## Trap-doors named in advance

- **The gate could be vacuously green.** If the chosen workload derives nothing, assertions 1 and 3 both
  pass while proving nothing. That is exactly why assertions 2 and 4 exist. If the rider reports the gate
  green but `oracle-alpha-key-count` is 0, the gate is a lie — re-check the workload, not the code.
- **A count differential going red is the real alarm.** Rows 1/2 are cheap; row 3 is the one that says
  the RESULT is unchanged. Weigh it by my own re-run, never the rider's report.
- **`out:alpha` not falling to ~0** would mean the clear landed somewhere that is not on the measured
  path — a wrong-site edit that still compiles.
- **The phase census is noisy** (a 3.1× swing was observed on `accumulate` at one size). Row 6 is read as
  a collapse-to-near-zero, not a precise number.
- **Beta's clear is the model, and beta's oracle side stays populated.** If the rider "helpfully" also
  changes the oracle, that is out of scope and must be reverted — the ruling is that the oracle is never
  optimized.

## What would make me reject the strike outright

Any edit to `wat/rete.wat` beyond the `Session` record's **comment**; a clear placed inside
`to_persistent`; or a refactor of `kernel.rs:3206`.
