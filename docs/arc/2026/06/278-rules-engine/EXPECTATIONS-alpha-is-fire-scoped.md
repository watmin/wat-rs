# EXPECTATIONS v2 — alpha is fire-scoped (written BEFORE the strike)

> **v2:** v1's premise was falsified by the rider's STOP-4 (the oracle returns EMPTY alpha, so the
> re-point target was wrong). Scope narrowed to ONE clear site; 2b re-points to `fire-once'`; the gate
> gained a fifth assertion. Rows below updated accordingly.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the new gate passes all five | `cargo nextest run --release -E 'test(/alpha_is_fire_scoped/)'` | green; native alpha 0, oracle alpha 0, equal, **`fire-once'` alpha > 0**, derived counts equal and > 0 |
| 2 | 2b re-pointed, still asserting | `cargo nextest run --release -E 'test(/2b_insert_alpha/)'` | green, via `fire-once'` |
| 3 | the RESULT did not move | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all pass — **this is the load-bearing row** |
| 4 | the floor | `cargo nextest run --release` | 4208 + the new gate's tests, all green |
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

- **The gate could be vacuously green.** If the chosen workload matches nothing, assertions 1–3 all pass
  while proving nothing. That is exactly why assertion 4 (`fire-once'` alpha > 0) exists. If the rider
  reports the gate green but that anchor is 0, the gate is a lie — re-check the workload, not the code.
- **v1's lesson, encoded:** the claim "the oracle populates alpha" was built from four grep hits read in
  detail while a fifth — the decisive one — was on screen and skipped. Any row below that rests on what
  the ORACLE returns must be asserted by the gate, never assumed by me.
- **A count differential going red is the real alarm.** Rows 1/2 are cheap; row 3 is the one that says
  the RESULT is unchanged. Weigh it by my own re-run, never the rider's report.
- **`out:alpha` not falling to ~0** would mean the clear landed somewhere that is not on the measured
  path — a wrong-site edit that still compiles.
- **The phase census is noisy** (a 3.1× swing was observed on `accumulate` at one size). Row 6 is read as
  a collapse-to-near-zero, not a precise number.
- **Beta is NOT divergent** (I reported it as such; retracted — both fixpoint verbs return it empty). If
  the rider touches `wat/rete.wat` logic at all, that is out of scope and must be reverted: the oracle is
  never optimized *and never adjusted to suit the kernel*.

## What would make me reject the strike outright

Any edit to `wat/rete.wat` beyond the `Session` record's **comment**; a clear placed inside
`to_persistent`; or a refactor of `kernel.rs:3206`.
