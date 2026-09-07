# BRIEF — land the `accum-over-derived` grid axis, three ways

## The work

Add one CORRECTNESS grid axis that bags a type the rule set derives, at a depth where the oracle
must retract `depth` intermediate accumulate results, and check it Clara | oracle | native. Today
**0 of 13** accumulate axes bag a derived type; the shape's only coverage is a probe pinned at two
intermediate states with a hand-derived Clara number in a comment.

## Read in order

1. `wat-scripts/perf/grid/retract-multiplicity.wat` — **the model.** Newest correctness axis, built
   under current conventions. Copy its whole shape: the `:grid::Result` record with
   `oracle-derived`/`oracle-ns`, the `defquery`, `build-rules` returning `:wat::rete::Rule` values
   directly, the `derived-vector` that **sorts and does not dedup**, and `:user::main` reading
   `[dials]` from `readln`. ⛔ Note its comment that the session variable must be named `staged` so
   `GRID_SKIP_ORACLE` / `axes_live` rewrite the right fire.
2. `wat-scripts/perf/grid/retract-multiplicity.clj` — the Clara twin's shape, and its header on why
   a static `.clj` with no `gen-` script is the correct form for a correctness axis. It does **not**
   self-invoke `-main`: `check-grid-three-way.sh` requires it into a shared JVM.
3. `wat-scripts/perf/grid/deep-cascade.wat:73-79` — **how a bounded cascade is built here.**
   `build-rules depth` folds `build-rule k` over `(range 1 depth+1)`, generating one rule per level
   with literal level constants. Use this; do not write a self-recursive rule with a `where` bound.
4. `wat-scripts/perf/grid/deep-cascade.wat:96-99` — `enc kind level id`, the single-i64 witness
   encoding to mirror.
5. `tests/rete/probe_arc278_derived_exists_acc.wat:22-31` — the tally rule shape with a **`Seed`
   anchor** (not a leading accumulate).
6. `tests/rete/wat_scripts_grid_port_check.rs:84-91` — the row contract, and the sentence that
   governs your expected count: it is *"re-derived from the shape the axis's own doc-comment
   states — not read off a run."* Derive it from the formula, then let the run agree with you.

## Sketch

```
;; accum-over-derived.wat — CORRECTNESS AXIS: acc :from a type THIS SET derives.
;;   Seed;  Step(0);  Step(k) :- Step(k-1) for k in [1,depth];
;;   Tally(n) :- Seed AND [?n <- (acc::count) :from Step]
;; Native puts Tally at stratum 1 (Step is derived) and evaluates it ONCE, after Step closes.
;; The oracle puts it at stratum 0 and re-evaluates each round, so `depth` intermediate
;; tallies are created and must be SUPERSEDED away.
;; :derived = sorted, NOT deduped: enc(0,k,0) per derived Step level + enc(1,0,n) per Tally.
;; A leaked intermediate tally is an EXTRA ELEMENT.
```

`build-rules depth` folds one `Step(k) :- Step(k-1)` per level plus the single tally rule.
`:user::main` takes `[depth]`, seeds `Seed` + `Step(0)`, fires native into `staged`/`fired`, then
fires `fire-rules$oracle` on the same `staged`, and prints one `#grid/Result`.

## Registration — all four, or the axis is invisible to a gate

| file | what to add |
|---|---|
| `check-grid-three-way.sh` | one `CORRECTNESS_SIZES` row, `[accum-over-derived]="<depth>"` |
| `tests/rete/wat_scripts_grid_port_check.rs` | `(stem, &[depth], expected_count, derivation)` |
| `tests/rete/wat_scripts_grid_axes_live.rs` | `(stem, &[depth], rationale)` |
| `wat-scripts/perf/grid/accum-over-derived.clj` | the static Clara twin |

`port_check` asserts the on-disk axis set equals its table, so a `.wat` with no row REDs and a row
with no `.wat` REDs. Expect that if you land them out of step.

## Blast radius

`wat-scripts/perf/grid/` (two new files, one `.sh` row) and two rows in `tests/rete/`.
**No `src/`. No `wat/`. No `+1` moved on either side.**

## STOP triggers

1. **If the axis comes back with native and oracle DISAGREEING on `:derived`** — STOP and surface
   it immediately, before any tidying. That is a live divergence at depth, it outranks the rest of
   this strike, and Clara is then the referee for which side is wrong.
2. **If Clara disagrees with BOTH wat engines** — STOP. That is the axis being wrong, not the
   engines, and a fixture that mis-models the shape would manufacture a finding.
3. **If the oracle does not terminate, hits the round cap, or takes minutes** at your chosen depth
   — STOP and report the depth. Do not raise a cap and do not quietly shrink the depth to whatever
   passes; the depth is the entire point of the axis and its value must be a stated choice.
4. **If `compile-all` answers `MayNotTerminate`** — STOP. Report the rule set; do not restructure
   the shape to get past it.
5. Do not add a `gen-accum-over-derived.sh`. Do not touch `stratify.rs`, `stratify.wat`, or
   `probe_arc278_oracle_accumulate_supersedes`.

## Prior comparable

`../strike-retract-removes-one/SCORE.md` — the last axis landed into this grid, same four
registration points, same static-`.clj` decision.
