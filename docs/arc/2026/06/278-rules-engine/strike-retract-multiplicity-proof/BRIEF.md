# BRIEF — add the retract-multiplicity correctness axis; record the violation

**Floor GREEN when you are done.** The deliverable is a recorded `:MISMATCH`, not a failing test.

## Read in order

1. **`DESIGN.md`** — the expected three-way signature is `MISMATCH / MISMATCH / match`.
2. **`wat-scripts/perf/grid/check-grid-three-way.sh`** — the pairing table, the `SIZES` table, and
   the completeness check that errors when a sized `.wat` has no row.
3. **`wat-scripts/perf/grid/negation.wat`** — a three-way-capable axis. **Copy its shape**: the
   `#grid/Result` carries `:derived`, `:native-ns`, `:oracle-derived`, `:oracle-ns`.
4. **`wat-scripts/perf/grid/gen-negation.sh`** — the Clara-translation generator's shape.
5. **`wat-scripts/perf/grid/where-not-fact.clj:26-33`** — `n-partial`, the row that *should* have
   caught this. Note it stages two DISTINCT facts and answers `(count (set …))`.
6. **`tests/rete/wat_scripts_grid_port_check.rs`** — `CORRECTNESS_SIZES` must gain the row too.

## The workload

Insert `F(k)` **twice** and `G(k)` once for each `k`; rule `Out(k) :- F(k) AND G(k)`; fire; then
**retract `F(k)` ONCE**; re-fire; report the `Out` keys.

**⛔ Both sides must report MULTIPLICITY, not a set.** No `set`, no `distinct`, no `(count (set …))`
on the Clara side — the whole defect is a count, and collapsing it reproduces the blindness this
axis exists to remove. Sort for a canonical order; do not dedup.

## The work

1. `retract-multiplicity.wat` — emits all four fields, `:oracle-derived` from `fire-rules$oracle`.
2. `gen-retract-multiplicity.sh` — the same workload in Clara, multiplicity preserved.
3. A `SIZES` row in `check-grid-three-way.sh` **and** a `CORRECTNESS_SIZES` row in
   `wat_scripts_grid_port_check.rs`. Keep them identical — the script checks.
4. Run `check-grid-three-way.sh` for this axis and **record the verdict verbatim** in the SCORE.
5. Run the `.wat` gates (`every_wat_scripts_file_loads`,
   `every_rete_name_in_wat_scripts_code_resolves`) and the floor.

## STOP triggers

1. **If the verdict is NOT `MISMATCH / MISMATCH / match`, STOP and report it.** `:port-accuracy
   :MISMATCH` would mean retract is not the shared verb I verified — the DESIGN is then wrong.
   `:accuracy :match` would mean the workload does not reach the defect.
2. **If either side collapses multiplicity, the axis proves nothing.** Check your own output before
   claiming a verdict.
3. **If the floor reddens, STOP.** A recorded `:MISMATCH` must not become a red floor — that is the
   entire reason the grid is the vehicle.
4. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-gather-predicted-visits/` — a strike whose deliverable was a number that indicts the
engine, reported plainly.
