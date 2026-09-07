# DESIGN — the `userfn-head` grid axis: prove the dropped fact stays gone

## Why — the builder's standing rule

> *"we need a grid example for this.. everytime we find a flaw.... we get a grid to prove its gone."*
> — 2026-09-07

`21a5f8514` cured the oracle. **The cure is not the deliverable; the standing fixture is.** Today the
only things holding that cure are two scratch `.wat` files and one in-crate test — none of which run
Clara, and none of which sit in the three-way where a future regression would be diagnosed.

## The gap, measured and anchored

**0 of 46 grid axes have a user-fn `:then`/`:rhs` head.** The sweep was anchored on the two fixtures
that DO have one (`arc278-produced-type-userfn-facts.wat`, `probe_arc278_then_user_forms_userfn.wat`)
— both light up, so the zero is a measurement.

⭐ **The sharpest form of it: the colon-strip was CORRECT for all 46 axes.** Every axis constructs
its facts with a record-type head, so the grid exercised only the shape in which the bug is
invisible. The instrument was never at fault — `wat_scripts_grid_port_check` compares native against
oracle `:derived` element-wise and would have named the missing `Out` outright.

**Third instance today of one class**, and the reason this axis exists rather than another probe:

| flaw | fixture gap |
|---|---|
| F2 — `retract` removed every equal fact | the grid fixtures stage no duplicate |
| accumulate over a derived type | 0 of 13 accumulate axes bagged a derived type |
| **this** | **0 of 46 axes had a user-fn head** |

Honest instrument, fixtures that do not reach the shape — every time.

## The shape

The `a2` rule set that drove the defect, lifted onto the grid contract with a swept dial:

```
  Src(k)                          for k in [0, items)          [input]
  Bad(k)  :- Src(k), k = -1                                    never fires; keeps the negation live
  Rate    :- Src(k), (not Bad(k)), :then [(mk-rate ?k)]        ★ USER-FN HEAD
  Out(n)  :- Rate(n)
```

`mk-rate` is a `:wat::rete::core::defn` returning `:Rate`, body exactly `(:Rate :count k)` — a
bound var, not a computed mint, so it is admitted (`stratify.rs:418-424`: the purity refusal is
FALSE today, driven 2026-08-28).

**Pre-cure this axis returns `Out` empty on the oracle and full on native — a MISMATCH the port
check names.** Post-cure all three agree. That is the regression test.

## The one contract decision

**`:derived` carries `Rate` AND `Out`, sorted and not deduped** — `enc kind id`, mirroring
`deep-cascade` / `accum-over-derived`. Carrying `Out` alone would make the count `0` on a
pre-cure oracle and `items` after, which reads fine; but carrying `Rate` too means the witness
**distinguishes "Out was dropped" from "nothing derived at all"**. The defect drops exactly one of
the two, and a witness that cannot tell them apart would pass a cure that broke both.

Expected count: `items` (Rate, one per key) + `items` (Out) = `2 * items`.

## Files

- new `wat-scripts/perf/grid/userfn-head.wat`
- new `wat-scripts/perf/grid/userfn-head.clj` — static twin, **no `gen-` script**
- `check-grid-three-way.sh` — one `CORRECTNESS_SIZES` row
- `tests/rete/wat_scripts_grid_port_check.rs` — one row (stem, size, expected count, derivation)
- `tests/rete/wat_scripts_grid_axes_live.rs` — one row

## Out of scope = REJECTED

- Any change to `rule-produces`, `produced_type`, or the cure. This axis PROVES the cure; it does
  not revisit it.
- A `gen-userfn-head.sh` / perf-ladder rung — a generator drags a correctness proof onto
  `check-grid-speed.sh`, where `:accuracy :MISMATCH` is a gate failure.
- `probe_arc278_then_user_forms_userfn.wat:12-22`'s stale purity claim — still rowed, still its own
  strike.
