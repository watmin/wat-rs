# SCORE — measured all nine; none earn the budget; nothing added

**Outcome: `.config/nextest.toml` is unchanged.** All nine measure comfortably under the 30s kill
even at the recorded band's pessimistic top (`alone × 4.4`), and their actual measured contention
on this floor (1.40x–1.80x) sits well *below* the recorded 3.5x–4.4x band, not inside or above it.
Per the brief's own STOP-1 and the task's framing — *"if the numbers say the budget is not needed,
that is a correct and valuable result"* — the correct action is to record the numbers and add
nothing. No `.rs` file, and no line of `.config/nextest.toml`, was touched.

## Scorecard

| # | EXPECTATIONS row | result |
|---|---|---|
| 1 | the nine share the measured test's binary | **HOLD.** `cargo nextest list -E 'test(<name>)'` for each of the nine reports binary `wat` (the lib binary), never `wat::rete`. STOP-3 does not fire. |
| 2 | nine alone-times, six samples, minimum taken | **HOLD.** Table below; all six-sample spreads ≤0.08s. |
| 3 | loaded times, one floor | **HOLD.** One `scripts/floor.sh` (`.floor/2026-09-09T00-50-29Z/`), all nine's durations read from `clean.log`. |
| 4 | contention per test, inside/outside the 3.5x–4.4x band | **HOLD, all nine OUTSIDE — below, not above.** Measured contention ranges 1.40x–1.80x, roughly half the band's floor. |
| 5 | verdict is arithmetic (`alone × 4.4` vs 30s) | **HOLD.** All nine land 3.5s–18.1s projected; none approaches 30s. |
| 6 | only earning names added | **HOLD, vacuously.** Zero names earned addition; zero were added. |
| 7 | order preserved (rete-cohort rule stays below `binary_id(wat::lint)`) | **HOLD.** No edit was made, so the file's order is untouched by construction. `git diff .config/nextest.toml` is empty. |
| 8 | all three profiles (`grep -c <name>` = 3 per added name) | **N/A, not vacillating-HOLD.** No name was added to any profile, so this gate has no subject. Stated plainly rather than silently marked HOLD. |
| 9 | config still parses | **HOLD.** `cargo nextest list --release > /dev/null` rc=0. |
| 10 | MUTATION — the budget is reachable | **N/A.** No budget was added for any of the nine, so there is nothing to drop a `period` on and mutation-prove. The existing rete-cohort filter's own mutation-proof (dated 2026-08-26, in the file) is untouched and was not re-derived — out of scope per DESIGN ("raising any existing budget… not this strike's numbers"). |
| 11 | floor, 5480, 0 fail | **HOLD on 0 fail; count differs.** `.floor/2026-09-09T00-50-29Z/`: `Summary [ 452.835s] 5484 tests run: 5484 passed, 19 skipped` — **0 failed, 0 slow**. EXPECTATIONS was drafted against a 5480-test tree; this branch (6 commits ahead, the quote-door/parser work) has grown by 4 tests since. The load-bearing number, 0 fail, holds; the population count in EXPECTATIONS is stale, not wrong-in-kind. |

## The measured table

Alone: `cargo test --release <path> -- --exact --nocapture`, nextest bypassed, six samples,
minimum taken. Loaded: single `scripts/floor.sh` run, `.floor/2026-09-09T00-50-29Z/clean.log`.

```
#   test                                          alone(min)  6-sample spread   loaded   contention   alone×4.4   verdict
    accum_matcher_op_census .................       0.799s    0.799–0.822s      1.195s     1.50x        3.52s     under (no budget needed)
    accum_leftover_split ....................       2.068s    2.068–2.140s      3.395s     1.64x        9.10s     under (no budget needed)
    accum_seen_fire_context_split ...........       2.615s    2.615–2.648s      4.615s     1.76x       11.51s     under (no budget needed)
    accum_alpha_leftover_split ..............       2.980s    2.980–3.016s      5.045s     1.69x       13.11s     under (no budget needed)
    accum_alpha_seed_after_fold_split .......       3.014s    3.014–3.075s      5.097s     1.69x       13.26s     under (no budget needed)
    cell_rank_after_fanout ..................       4.026s    4.026–4.105s      7.262s     1.80x       17.71s     under (no budget needed)
    cell_rank_after_grid ....................       4.055s    4.055–4.119s      7.049s     1.74x       17.84s     under (no budget needed)
    honest_cell_rank_after_arm ..............       4.111s    4.111–4.142s      7.134s     1.74x       18.09s     under (no budget needed)
    gather_index_is_built_once_per_alpha_and_keyset  0.800s   0.800–0.819s      1.122s     1.40x        3.52s     under (no budget needed)
```

Sources for `path` used in the alone runs (verified by `grep -n` against DESIGN.md's table before
measuring, all lines matched exactly):

- `rete::kernel::tests::accum_cost::accum_matcher_op_census` — `accum_cost.rs:33`
- `rete::kernel::tests::accum_cost::accum_leftover_split` — `accum_cost.rs:606`
- `rete::kernel::tests::accum_cost::accum_seen_fire_context_split` — `accum_cost.rs:1659`
- `rete::kernel::tests::accum_alpha_cost::accum_alpha_leftover_split` — `accum_alpha_cost.rs:75`
- `rete::kernel::tests::accum_alpha_cost::accum_alpha_seed_after_fold_split` — `accum_alpha_cost.rs:383`
- `rete::kernel::tests::rank_and_instrument::cell_rank_after_fanout` — `rank_and_instrument.rs:1149`
- `rete::kernel::tests::rank_and_instrument::cell_rank_after_grid` — `rank_and_instrument.rs:1233`
- `rete::kernel::tests::rank_and_instrument::honest_cell_rank_after_arm` — `rank_and_instrument.rs:1319`
- `rete::kernel::tests::gather_probe_cost::gather_index_is_built_once_per_alpha_and_keyset` — `gather_probe_cost.rs:31`

## What surprised me — the counter-evidence is stronger than DESIGN stated

DESIGN carried the counter-evidence as *"an argument, not a measurement"*: two green floors had not
tripped any of the nine, against a ~452s floor roughly double the ~220s the 3.5x–4.4x band was
derived on. **This driving turns that argument into a measurement, and the measurement goes further
than DESIGN guessed:**

1. **None of the nine sits anywhere near the band**, loaded or projected. The worst (`cell_rank_after_fanout`,
   1.80x) is still under half the band's own floor (3.5x).
2. **The named calibration test itself measured far below its own recorded band on this same floor.**
   `accum_fire_phase_census` (the test the 3.5x–4.4x band was *derived from*, at `clean.log:336`)
   loaded at **10.580s** this run — contention **1.30x** against its own alone-time of 8.13s, not the
   recorded 4.35x it was budgeted for. This wasn't one of the nine and I did not re-measure its
   alone-time (that would touch scope — "raising any existing budget… not this strike's numbers" cuts
   both ways, I did not lower it either), but its *loaded* number came for free out of the same floor
   log I was already reading, and it says the whole cohort is running under markedly less contention
   today than when the band was written (2026-08-17). That is a plausible reason the nine — same
   binary, same general workload class — also came in low, and it is worth handing up as its own
   finding: **the 3.5x–4.4x band may itself be stale for this box's current scheduling behavior**,
   which is a separate question from whether the nine need budgets (they don't, at either the old or
   the new contention reading).
3. Zero of the nine tripped even the *default* 15s slow-warn, let alone the 30s kill. The floor log
   shows no `SLOW` annotation on any of the nine, and no `(N slow)` at all in the run's Summary line
   — the first floor in this cohort's file history with zero slow lines.

## What this did not do

- **No edit to `.config/nextest.toml`.** Every name measured safely under budget; adding any of them
  to the filter — even with a correct row above it — would have granted headroom nothing here needs,
  which is the raise-a-number-anyway failure mode DESIGN names, just aimed the other direction.
- **No re-measurement of `accum_fire_phase_census`'s alone-time.** Its loaded time fell out of the
  floor log for free and is reported above as counter-evidence; touching its budget is explicitly out
  of scope.
- **No mutation-proof of a new filter entry** — there is no new entry to prove reachable.
- **No `.rs` file touched**, per the brief's blast radius.
- **No re-run on the floor.** One floor, read once, per the brief's method ("that is one floor for all
  nine, not nine floors").

## One process note against myself

Mid-measurement I ran a single `cargo test --release rete::kernel::tests::accum_cost::accum_fire_phase_census`
as a sanity check *while `scripts/floor.sh` was already running in the background* — a violation of
"one cargo invocation at a time." It produced one contaminated data point (10.211s, not usable as
either an alone or loaded number) which is discarded and does not appear in the table above; the
clean 10.580s loaded figure quoted in the counter-evidence section came from the subsequent
uncontended floor's own log, not from that invocation. No other overlap occurred.

## Final floor

`.floor/2026-09-09T00-50-29Z/`: `Summary [ 452.835s] 5484 tests run: 5484 passed, 19 skipped` — 0
failed, 0 slow. `git status --porcelain -- .config/nextest.toml` and `git diff -- .config/nextest.toml`
are both empty; this SCORE.md is the only file this strike adds.
