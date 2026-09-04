# DESIGN — a counter that adds calls to pairs, pinned as a call count, equal to another test's number

## Why

Work-list **C14**: *"`compiled:calls` is not a call count, and its name says it is."* The row names a
multiplicative bulk add and calls it `[[a-right-number-vouches-for-a-wrong-label]]` in the substrate.

**Driven at HEAD `d7464c95e`, the defect is larger than the row states — by exactly the amount that
matters.**

## What is asserted today

| test | assertion | its own words |
|---|---|---|
| `accum_alpha_cost.rs:41` `accum_alpha_memory_shape` | `last.alpha_elements == 80_200` | *"alpha memory holds one element per (fact, matching alpha) pair"* |
| `accum_cost.rs:66` `accum_matcher_op_census` | `compiled:calls == 80_200` | *"the compiled path ran a different number of times for the same (200, 200) workload"* |

Three sites bump `compiled:calls`: `fire/delta.rs:78` and `compiled_cond.rs:928` add **one per call**;
`fire/pass/alpha.rs:195` adds **`ids.len() × aids.len()`** — a product of two lengths.

## The drive

Rename **only** the `alpha.rs:195` key to `compiled:leaf-fill-pairs`; touch nothing else. Then:

```
thread '…accum_cost::accum_matcher_op_census' panicked at accum_cost.rs:41:5:
compiled:calls is zero — occupancy fill / skip-span / exec_compiled never counted
```

**`compiled:calls` drops to ZERO.** Two consequences, neither in the row:

1. **100% of the pinned 80,200 is the product.** Not "one contributor among three" — the *only*
   contributor on this workload.
2. **`delta.rs:78` and `compiled_cond.rs:928` fire ZERO times here.** The accum axis never enters
   the compiled path at all.

And because the product `ids × aids` **is** the alpha-element count, the two assertions above pin the
**same quantity**. `accum_matcher_op_census` is not an independent check of how often the compiled
path ran; it is `alpha_elements` recomputed through a differently-named counter, in another file.

**Two tests, one quantity, and one of them calls it a call count.**

## Why the liveness guard cannot see it

`assert!(calls > 0, "occupancy fill / skip-span / exec_compiled never counted")` names three
mechanisms and can only observe one. Delete the skip-span arm and the exec_compiled arm entirely and
this guard stays green, because the occupancy fill alone supplies every unit. **A guard whose named
population is larger than what it can observe is a guard that reports on two mechanisms it never
touches** — the same shape as C16's differential agreeing with the corruption by construction.

## The contract decision, pinned

**Give the product its own key, and let each counter carry one unit.**

- `fire/pass/alpha.rs:195` emits **`alpha:leaf-fill-pairs`** — a pair count, named as one.
- `compiled:calls` keeps only the two genuine per-call sites, and therefore becomes an honest call
  count.
- **`accum_matcher_op_census` must then assert what is true**: on this workload the compiled path is
  entered **zero** times, and that fact is the finding, not an inconvenience to paper over. The test
  states it and says why, or names the workload that does enter it.
- `accum_alpha_memory_shape` keeps `alpha_elements == 80_200`. The duplicate pin under the wrong
  name is what goes.

⛔ **This is NOT the split C10 forbade.** C10's comment says *"Do NOT split this counter"* about
distinguishing the two **delta arms** — a hot-path branch added for an instrument's benefit, whose
discrimination already exists one file over in `accum_alpha_cost.rs`. Giving one existing
`census_count_n` site its own key is one string at one site: no branch, no hot-path edit, and it
separates two **units**, not two arms. The ruling stands; this is a different act.

## ⛔ AND THE C10 CURE'S OWN CITATION HAS ROTTED

`accum_cost.rs:52` cites the bulk add at **`fire/pass/alpha.rs:122`**. That line is now inside
`pack_i64_row`; the site is **`:195`**, moved by D7's cure. A comment written specifically to stop
the next reader misattributing this counter now sends them to the wrong function.

## Out of scope = REJECTED

- **Splitting the two delta arms.** C10 ruled it out on the merits and the discrimination exists
  elsewhere. Unchanged.
- **Making the accum axis exercise the compiled path.** That is a workload change to a recorded perf
  axis; the sizes are the artifact. If a call-count assertion needs a different workload, name it —
  do not re-dial this one.
- **Every other census key.** This strike fixes the one that is driven and measured. A sweep for
  other unit-mixing counters is a separate strike and must be measured before it is drawn.
