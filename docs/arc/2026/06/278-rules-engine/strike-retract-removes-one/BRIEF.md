# BRIEF — `retract` removes ONE occurrence

Read `DESIGN.md` first. Replace `factbag::remove-every-equal` with `factbag::remove-one`, land the
redrawn proof axis in the **same commit**, and delete the old door. The axis is the cure's
acceptance test: it is RED today and must be GREEN when you are done.

## Read in order

1. `wat/rete/factbag.wat:57-73` — `remove-every-equal`, the door you are replacing. Its fold is
   today's semantics; `remove-one` is the same walk with a "have I dropped one yet" flag.
2. `wat/rete/oracle/stratify.wat:43-45` (`StratifyAcc`) — the house idiom for fold state: a small
   record with the payload plus a `bool`. Copy this shape; there is no `remove-first` primitive.
3. `wat/rete/oracle/insert.wat:100-125` — `retract`, the single caller. Its docstring already
   claims *"Symmetric with insert"* and *"value-precise"*; after this strike both are TRUE, so the
   docstring stops being a promise and becomes a description. Say that in the comment.
4. `wat/rete/oracle/fire.wat:316-322` — read before you write the fold. `retain`'s sub-multiset
   property carries the support-fixpoint's convergence proof. `remove-one` must be order-preserving
   for the same reason.
5. `wat-scripts/perf/grid/retract-multiplicity.wat` (in the tree, uncommitted) — the axis. Its
   `:rm::seed` fold conj's `F(i)` **twice for every i**; change it to conj `F(i)` once and add ONE
   extra `F(0)` after the fold. Mirror the same change in `retract-multiplicity.clj`'s `seed-facts`.
6. `tests/rete/wat_scripts_grid_port_check.rs` — the axis's `CORRECTNESS_SIZES` row: `want_n`
   **2 → 3**, and the prose below it describes the old seed. `wat_scripts_grid_axes_live.rs`'s
   `SIZED_AXES` row likewise.
7. `wat-tests/rete/differential-fuzz-tms.wat:26-30` — strike the sentence claiming `final-facts`
   replays the program. That function does not exist; the arms call the real verb. **No other
   change to that file.**

## Sketch

```
;; wat/rete/factbag.wat
(:wat::core::defrecord :wat::rete::FactBagDrop
  [items   <- (:wat::core::PersistentVector :- [:wat::core::Record])
   dropped <- :wat::core::bool])

;; remove-one — drop the FIRST value-equal fact; every other fact keeps its position.
;; Absent => unchanged. Symmetric with `add`: one call moves the multiplicity by one.
(:wat::core::defn :wat::rete::factbag::remove-one
  [b <- :wat::rete::FactBag  fact <- :wat::core::Record] -> :wat::rete::FactBag
  ;; foldl a FactBagDrop: keep f when (or dropped (not (= f fact))); otherwise skip it once
  ;; and set dropped.
  …)
```

## Blast radius

`wat/rete/factbag.wat` (one door replaced, one deleted) · `wat/rete/oracle/insert.wat` (comment
only) · `wat-scripts/perf/grid/retract-multiplicity.{wat,clj}` (the seed) ·
`tests/rete/wat_scripts_grid_{port_check,axes_live}.rs` (the two rows) ·
`wat-tests/rete/differential-fuzz-tms.wat` (one comment sentence). **No Rust engine change.**

## The prediction that keeps this safe

Every `retract` call site was measured for duplicates in flight and **none has any** (DESIGN,
"Blast radius"). So the cure is invisible everywhere except the redrawn axis. If any other value
moves, that measurement was wrong — STOP-1.

## Verdict to produce

Run `check-grid-three-way.sh retract-multiplicity` and quote the raw `#grid/Verdict` line. Expected
after the cure:

```
:accuracy :match  :oracle-accuracy :match  :port-accuracy :match
```

Also quote the **pre-cure** run of the redrawn axis (before you touch `remove-one`), so the strike
carries its own before/after: expected wat `[1 2]` vs clara `[0 1 2]`.

## STOP triggers

1. Any test outside the redrawn axis changes value → STOP; the blast-radius measurement is wrong.
2. `remove-one` cannot be written order-preserving → STOP; `fire.wat:316-322` depends on it.
3. The post-cure three-way is not `match / match / match` → STOP, quote the raw verdict.
4. `ci.yml:262` runs the three-way with no args, so a still-red axis blocks the push → if the axis
   will not go green, STOP rather than committing.

## Prior result to copy for shape

`../strike-factbag-one-owner/SCORE.md` — starred rows, verbatim quoted evidence, an honest
"load order" delta section, and the first-floor arms captured rather than re-run.
