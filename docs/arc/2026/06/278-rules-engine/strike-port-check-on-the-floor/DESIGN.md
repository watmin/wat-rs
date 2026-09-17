# DESIGN — the spec check the arc never ran costs five seconds, not four hours

## Why

Work-list **C9**: *23 recorded grids prove the fast path matches Clara; none prove it matches its own
spec.* It was parked as a ~4-hour sweep. **That price was for the wrong half.**

`run-axis.sh` names three pairings and they do not cost the same:

| pairing | diagnoses | needs |
|---|---|---|
| `oracle` vs `clara` | **the SPEC is wrong** | the JVM — expensive |
| `native` vs `clara` | the fast path is wrong | the JVM — 23 grids have done this |
| **`native` vs `oracle`** | **a PORT bug** | **nothing but `target/release/wat`** |

Every axis `.wat` emits `:derived` **and** `:oracle-derived` **in one process**. Driven, all eleven
axes at correctness sizes:

```
min-finding match · negation match · leading-exists match · neg-consumer match · asym-join match
strat-neg match · user-reduce match · node-share match · accum match · deep-cascade match · fanout match
TOTAL 5s
```

**11/11 match, five seconds, no JVM.** The check that had never been run is a floor gate, not a
scheduling problem.

## ⛔⛔ AND IT WOULD NOT HAVE CAUGHT D7

D7 — this arc's worst defect, a silent fact-drop — **was a native-vs-oracle divergence**. Exactly the
pairing above. It was found by hand because nothing ran it.

But running it would not have been enough. Measured: **0 of 185 `defrecord` forms across the whole
axis corpus declare a parametric record** (`:- [T]`). D7's trigger is *a parametric record erasing its
type argument into one runtime class whose instances differ in packability*. **That shape is absent
from the grid**, so a green port check would have said nothing about it.

**A differential is only as good as its corpus, and this corpus has a hole shaped exactly like the
bug the arc just spent a day on.** Landing the gate without widening the corpus repeats the miss with
a green light on top.

## The contract decision, pinned

**The port check goes on the floor, and its corpus gains the shape that beat it.**

- A gate runs every axis at *correctness* sizes — not the perf ladder — and compares `:derived`
  against `:oracle-derived`, failing with both sets named.
- **The corpus gains a parametric axis**: one class, instances of mixed packability, the D7 shape.
  Without it the gate is green on a corpus that cannot express the defect.
- **⚠ `fanout` emits `#fan/QuerySplit`, not `#grid/Result`** (driven). The gate must handle that or
  say why it is excluded — a silent skip is how an axis goes dark.

## Out of scope = REJECTED

- **The Clara half (`oracle` vs `clara`, "the SPEC is wrong").** Still needs the JVM and still costs
  hours. It is the pairing that catches a flaw the oracle and its port **share**, so it is not
  redundant — it is separately scheduled, and this strike does not pretend to cover it. **C9 stays
  open for that half.**
- Changing the perf ladder. These sizes are for correctness; the ladder's sizes are the perf
  artifact and must not drift.
