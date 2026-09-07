# BRIEF — land the `userfn-head` grid axis, three ways

## The work

One CORRECTNESS grid axis whose rule head is a **user fn**, so the flaw cured in `21a5f8514` has a
standing three-way fixture. Today 0 of 46 axes have such a head, which is why the grid never saw it.

## Read in order

1. `wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat` — **the rule set to lift.** Run
   it first (`cargo run --release --bin wat -- <path>`); it prints the strata and both engines'
   facts. This axis is that shape with a swept `items` dial.
2. `wat-scripts/perf/grid/accum-over-derived.wat` — **the model**, landed hours ago under the same
   contract. Copy its `:grid::Result` record, `derived-vector` (sorted, NOT deduped), the `staged`
   variable name, and its `:user::main`.
3. `wat-scripts/perf/grid/accum-over-derived.clj` — the static-twin shape and why there is no
   `gen-` script.
4. `wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.clj` — **the Clara referee for this
   exact rule set already exists.** Its header states the two modelling judgements and their
   falsifier: the user fn is inlined as `(insert! (->Rate ?k))` because `mk-rate`'s body is exactly
   that, and `Bad` never fires so the negation is vacuously true. Carry both statements into the
   axis's `.clj` — with `items` swept instead of a single `Src(1)`.
5. `tests/rete/wat_scripts_grid_port_check.rs:84-91` — the row contract: the count is the
   ANTI-VACUITY instrument, *"re-derived from the shape the axis's own doc-comment states — not
   read off a run."*

## Sketch

```
;; userfn-head.wat — CORRECTNESS AXIS: the :then head is a USER FN, not a record constructor.
;;   Src(k) for k in [0,items);  Bad :- Src, k = -1  (never fires, keeps the negation live);
;;   Rate   :- Src(k), (not Bad(k)), :then [(mk-rate ?k)]   <- the user-fn head
;;   Out(n) :- Rate(n)
;; Pre-cure the oracle assigned the stratum to `mk-rate`, left Rate at 0, and Out -- consuming
;; Rate -- sat below its producer, so Out was DROPPED. `21a5f8514` cured it.
;; :derived carries BOTH Rate and Out, sorted, not deduped: a witness carrying Out alone
;; cannot tell "Out dropped" from "nothing derived".  Expected count = 2 * items.
```

## Registration — all four, or the axis is invisible to a gate

`check-grid-three-way.sh` (`CORRECTNESS_SIZES`), `wat_scripts_grid_port_check.rs`,
`wat_scripts_grid_axes_live.rs`, and the static `.clj`. `port_check` asserts the on-disk axis set
equals its table, so landing them out of step REDs.

## ⭐ Prove the axis would have caught the flaw

An axis that is green after the cure proves nothing on its own. **Re-introduce the defect and show
this axis reddens**: revert `rule-produces` in `wat/rete/oracle/stratify.wat` to the colon-strip
(`git show 21a5f8514^:wat/rete/oracle/stratify.wat` has it), rebuild, run the port check, and quote
the MISMATCH naming the missing `Out` rows. Then restore and re-run green.

⛔ Verify the revert **actually changed the file** before trusting its red — an empty mutation reads
exactly like a working cure.

## STOP triggers

1. **If the axis is green under the reverted cure** — STOP. The axis does not discriminate and is
   worthless; report the `:derived` both engines produced.
2. **If Clara disagrees with the wat engines post-cure** — STOP. Either the `.clj` mis-models the
   rule set or the cure is wrong, and both outrank landing an axis.
3. **If `compile-all` refuses the user-fn head** (`MayNotTerminate`, or a purity refusal) — STOP and
   report the verbatim text. `mk-rate`'s body must stay a bound-var construction; a computed mint
   IS refused and that is a different shape.
4. Do not touch the cure, `produced_type`, or `rule-produces`. Do not add a `gen-` script.

## Blast radius

Two new files in `wat-scripts/perf/grid/`, one `.sh` row, two `tests/rete/` rows.
**No `src/rete/kernel/`. No `wat/rete/`.**

## Prior comparable

`../strike-grid-accum-over-derived/SCORE.md` — same four registration points, same static-`.clj`
decision, and its mutation (disabling supersession) is the shape of the proof asked for above.
