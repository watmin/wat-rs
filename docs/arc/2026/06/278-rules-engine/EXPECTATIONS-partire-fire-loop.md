# EXPECTATIONS — partire the fire loop

Written BEFORE the strike, so the result cannot move the goalpost.
Scored against my own re-run, never the executor's report.

## Per-pass scorecard — every row, every pass, before the next starts

| what | the command that checks it | expected |
|---|---|---|
| differential: oracle == native | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 363/363, `spec_equals_native_on_every_where_family` green |
| 3-stratum negation | `cargo nextest run --release -E 'test(differential_three_stratum_negation)'` ×3 | 3/3 |
| concurrency unaffected | `cargo nextest run --release -E 'test(probe_arc278_concurrent_retes)'` | 5/5 |
| the floor | `scripts/floor.sh` | GREEN, ≥4942 passed, no `ARM.txt` |
| rustc + clippy wall | `cargo clippy --release --workspace --all-targets -- -D warnings` | silent, exit 0 |
| the diff is a MOVE | `git show --stat` + read the diff | additions ≈ deletions; no logic edit inside the moved block |

## End-of-strike scorecard

| what | the command | expected |
|---|---|---|
| accuracy across the grid | `GRID_SKIP_ORACLE=1 GRID_RUNS=3 bash wat-scripts/perf/grid/run-all.sh` | 30/30 `:match`, 30/30 `:us` |
| no perf regression | compare vs the last committed grid | every big cell within ±1%; **no axis moving coherently across all three rungs** |
| the shape actually changed | `awk` the function span | `fire_fixpoint_delta_armed` ≤ ~200 lines; max nesting ≤ 7; no pass > 400 lines |

## Runtime prediction

Nine passes, one commit each. Each cycle is a build (~50 s), the rete cohort
(~55 s) and a floor (~275 s) — call it **8–10 min of gates per pass**, so
**~90 min of gate time** plus the extraction work itself. The floor dominates
and is not compressible; that is the price of a differential-gated refactor and
it is worth paying nine times.

## Perf prediction

**No measurable change, in either direction.** This is the falsifiable half. A
move that speeds something up has not been understood, and a move that slows
something down has introduced a copy. Either is a STOP.

## Trap doors, named now

1. **`RoundCtx` quietly clones.** The obvious way to make nine passes compile is
   to hand each one owned copies of the round state. That converts a
   readability problem into a performance one and would be invisible to the
   differential — every test would stay green while the engine got slower. The
   grid at the end is the only thing that catches it, which is why STOP-1 in
   the stone forbids inventing a clone to make a move compile.
2. **The census marks move with the code and change nesting.** `production`
   already reads ~7.5 ms of its own children's mark tax; if marks land in
   different parents the whole census tree becomes incomparable to every
   number in this arc. Marks move with their pass, unchanged.
3. **The catch-up take/restore invariant** (`delta.rs:666-766`, two restore
   sites, one nested 12 deep) travels inside the largest pass, extracted last.
   A `?` introduced during that move silently drops a beta memory and no test
   asserts it. Read that window line by line; do not reflow it.
4. **`build.rs` auto-discovery.** New files under `fire/pass/` need `mod`
   wiring; a missed one fails the build loudly, which is the good case. The bad
   case is a pass file that compiles but is never called — the differential
   catches that, because its work would simply not happen.
5. **A silent behaviour change inside a moved block.** The mitigation is the
   contract: the diff must be a move. If `git show --stat` does not show
   additions ≈ deletions, the commit is doing two things.

## What would make me abandon this

- A pass that cannot be extracted without cloning currently-borrowed state,
  after an honest attempt (STOP-1).
- The differential going red twice on the same pass after a clean revert —
  that would mean the seams are not where the comments say they are, and the
  map is wrong rather than the work.
