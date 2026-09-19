# DESIGN — a grid axis for `acc :from` a DERIVED type, at depth, with Clara live

## Why — and NOT the reason you would guess

L2-3 established (`aa10ef8bd`) that native and oracle assign **different strata** to a rule that
bags a type its own rule set derives: native 1, oracle 0. The facts agree everywhere anyone has
looked. The two engines reach that agreement by **different mechanisms** — native by stratifying
(the bag is complete before it is counted), the oracle by supersession (`fire-support-fixpoint`,
`16f504e14`, which lets the count run early then re-derives and drops what lost support).

**⛔ The tempting justification is FALSE and must not be written into this axis's header.** It is
NOT *"nothing covers this shape."* `tests/rete/probe_arc278_oracle_accumulate_supersedes.{wat,rs}`
bags `:oas1::Out` / `:oas2::Out`, **both derived**, across three growth profiles (empty, 0→1,
0→1→2), asserts native against oracle, and is GREEN. That probe is real coverage and it is
evidence *against* a latent flaw.

**The true gap is DEPTH and a LIVE referee, and it is narrow:**

1. **The probe pins this at n ≤ 2 — three rounds, two intermediate states.** The oracle's cure is a
   shrink-to-fixpoint loop, `F := F ∩ (base ∪ D(F))`, whose work scales with the number of
   intermediate states it must retract — and that count is exactly what the stratum difference
   controls (native creates none; the oracle creates one per round). Nothing in the tree exercises
   that past two.
2. **The probe's Clara figure is a dated comment** (2026-08-31, hand-derived once). The grid runs
   Clara in a JVM on every check, and its three pairings separate the faults:
   `oracle != clara` = the spec is wrong · `native != clara` = the fast path is wrong ·
   `oracle != native` = a port bug.

**Measured, and anchored:** **0 of 13** accumulate axes in `wat-scripts/perf/grid/` bag a type the
same rule set derives. The sweep was anchored on two known positives (`probe_arc278_derived_exists_acc`,
`arc278-l2-3-stratify-numbers.wat`); both light up, so the zero is a measurement. ⚠ A first pass
reported one hit — `where-fact-bind` — which was the regex capturing the field accessor
`(:wfb::Temp/c ?t)` as a produced type. Corrected before it reached the builder.

## ⚠ AND IT IS NOT A SIZE SWEEP — one correctness size per axis

`CORRECTNESS_SIZES` carries **one size vector per axis**; `strat-neg`'s `&[3, 50]` is two *dials*
(strata, items), not two sizes. A real sweep is the perf ladder, which needs a `gen-<axis>.sh`, and
`retract-multiplicity.clj`'s header states why a correctness axis must stay off it: a generator
drags the proof onto `check-grid-speed.sh`, where `:accuracy :MISMATCH` is a gate failure. So the
gain here is **one depth well past the probe's two**, not a curve. Say that and nothing more.

## The shape

Modelled on `deep-cascade.wat`, which already proves multi-round recursion works in this tree and
how it is bounded — it **generates `depth` rules**, one per level, rather than one self-recursive
rule:

```
  Seed                                   [input, 1 fact]
  Step(0)                                [input, level 0]
  Step(k) :- Step(k-1)        for k in [1, depth]     — `depth` generated rules
  Tally(n) :- Seed AND [?n <- (acc::count) :from Step]
```

`Step` is produced by the rule set, so `rule_bag_consumes` puts it in `exists_and_from_types` and
the native `+1` fires — `Tally` at stratum 1, evaluated once, after `Step` is closed. The oracle
folds the `:from` into `rule-consumes` (`req-pos`, NOT +1), so `Tally` sits at stratum 0 and is
re-evaluated as `Step` grows: **`depth` intermediate tallies**, every one of which supersession
must retract.

**The anchor is a `Seed` condition, deliberately.** A *leading* accumulate would entangle this with
`conferre` L2-1 (the unguarded leading-accumulate re-seed), which is a different open row and must
not be braided into this measurement.

## The one contract decision

**`:derived` is the sorted, NOT deduped, encoded vector of every derived `Step` fact AND every
`Tally` fact** — `enc kind level id`, mirroring `deep-cascade.wat` / `accum.wat`. A leaked
intermediate tally is then an EXTRA ELEMENT: `[… Tally(3) Tally(9)]` instead of `[… Tally(9)]`.

This is what makes both gates bite. `wat_scripts_grid_port_check.rs` compares native against oracle
element-wise **and** checks the count as its ANTI-VACUITY instrument — *"re-derived from the shape
the axis's own doc-comment states, not read off a run."* With this encoding the expected count is
`depth + 1` (levels 1..depth, plus one Tally), so it moves with the dial instead of being a
constant 1 that no size could falsify.

## Files

- new `wat-scripts/perf/grid/accum-over-derived.wat`
- new `wat-scripts/perf/grid/accum-over-derived.clj` — **static twin, NO `gen-` script**
- `wat-scripts/perf/grid/check-grid-three-way.sh` — one `CORRECTNESS_SIZES` row
- `tests/rete/wat_scripts_grid_port_check.rs` — one row (stem, size, expected count, derivation)
- `tests/rete/wat_scripts_grid_axes_live.rs` — one row

## Out of scope = REJECTED

- **Any cure, and any `+1` moved on either side.** If the axis is green, the finding stays what
  L2-3 made it: a false lockstep claim in two headers.
- A `gen-accum-over-derived.sh` and any perf-ladder rung.
- `conferre` L2-1's leading accumulate — a different row; the `Seed` anchor keeps it out.
