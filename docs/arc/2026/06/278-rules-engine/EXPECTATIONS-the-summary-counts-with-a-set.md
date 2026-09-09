# EXPECTATIONS — the summary counts with a set

Written **before** the strike. `wat-scripts/fanout/circuit.wat` only, `:fanout::summarize` only.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **`distinct` and `dup` are unchanged** | n=2000 `vis-ms=1000` | `distinct=8000`, `dup=0`. A data-structure change with zero semantic change |
| 2 | ★ **`workers` still reports the same shape** | the same run | in the 8–12 range this arc's SCOREs span. It counts **distinct worker ids in outcomes**, not live processes |
| 3 | ★ **no cloning verb remains in `summarize`** | `grep` the function | zero `:wat::hashmap::` between `:fanout::summarize`'s open and close |
| 4 | ★ **the harness's unaccounted time is measured** | the same run | report `total` **minus the sum of** `setup+fill+arm+drain+collect+stop`, beside the ~560 ms baseline. This is the number the stone moves |
| 5 | **wall clock and phases hold** | the same run | within run-to-run variance of the 40 s / fill 15.5–17.3 s band. **No speedup is expected at n=2000** |
| 6 | **the other reported fields are untouched** | the per-tier line and the header line | `total`, `empty`, `seen-recorded`, `seen-skipped`, the counters, `full-retries` all behave as before |
| 7 | **blast radius** | `git diff --stat` | `circuit.wat` **only**, and the diff confined to `:fanout::summarize` |
| 8 | **the corpus loads** | `every_wat_scripts_file_loads` | PASS |
| 9 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5237 passed, 22 skipped, **0 FAIL, 0 TIMEOUT** |

## The rows that carry it

★★ **Row 1 is the stone's whole risk.** `distinct` is the correctness signal for the entire circuit — it
is how loss and duplication are detected. If `distinct` shifts by one, the change is wrong, no matter
what the timings say. Report the number, not "unchanged".

★ **Row 4 is what the stone is for**, and nobody has measured it before. `total` exceeds the sum of the
phase timers by roughly 560 ms at n=2000, and `summarize` lives in that gap. Print both figures and the
difference; do not assert an improvement, because at n=2000 the term is smaller than fill's own spread —
which is row 5.

⚠ **Row 5 forbids the win this stone cannot honestly claim.** The O(N²) term is small at n=2000. The
payoff is that it stops growing before the n=4000 measurement. A SCORE that reports a speedup here has
measured variance.

★ **Row 3 is greppable on purpose** — a property I can check without trusting a report, in a file where
four other cloning sites deliberately remain (the scenario deftests), so a whole-file grep would be the
wrong instrument.

## Runtime prediction

**20–35 minutes.** Two folds, one `count`, and the type ascriptions the accumulator carries — expect the
same "more edits than the sketch names" that the last stone found, because the accumulator type appears in
the closure signature and the return type. One n=2000 run at ~40 s. The floor is the long pole.

## Trap-doors

- **`w-map` becomes a set too**, and its consumer counts it the same way. Do not leave one fold as a map.
- **`distinct` reads `:wat::set::length`**, not `count (keys …)`. The `keys` call disappears.
- **`:wat::set::` has five verbs** — `conj disj contains? empty? length`. There is no `keys`.
- **The accumulator's type appears three times** — closure parameter, closure return, and the fold's
  initial value. The last stone's sketch named half its sites for exactly this reason.
- **Do not touch the four `:user::` scenario deftests.** They keep their cloning verb by decision, so a
  file-wide `grep -c hashmap` will not read zero and should not.

## What this stone does NOT claim

⚠ It does **not** speed up n=2000. It stops a harness term from growing as the axis under study.
⚠ It does **not** touch the queue, the topic, admission, or the cap.
⚠ It does **not** clean up the four scenario deftests or anything under `wat/`.
