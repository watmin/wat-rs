# DESIGN — the third pairing was never expensive; it was run at the wrong volume

## Why

Work-list **C9's remaining half**: *"`oracle` vs `clara` — the SPEC is wrong — still needs the JVM
and still costs hours."* It has been parked on that price since the row was filed.

**The price was for the wrong volume, not the wrong pairing.** Builder's ruling, 2026-09-03:

> *"the wat oracle will take many hours to fully exercise — clara vs wat native is the typical
> measurement — wat oracle vs wat native needs to use low volume tests so we don't waste hours"*

That is the split, and it is now half-built. `clara vs native` is the perf grid at ladder sizes (47
recorded). `oracle vs native` landed today as a floor gate at **low volume — 12 axes, 12.2 s, no
JVM** (`ed555d02e`). What is missing is the third pairing at the **same low volume**.

## Driven at HEAD `ed555d02e`, before this was drawn

All 11 perf axes, three ways, at the port gate's own correctness sizes — `probe-threeway.sh.txt`
beside this file:

```
min-finding    clara=49  native=49  oracle=49   ALL THREE MATCH
negation       clara=25  native=25  oracle=25   ALL THREE MATCH
leading-exists clara=20  native=20  oracle=20   ALL THREE MATCH
neg-consumer   clara=25  native=25  oracle=25   ALL THREE MATCH
asym-join      clara=200 native=200 oracle=200  ALL THREE MATCH
strat-neg      clara=75  native=75  oracle=75   ALL THREE MATCH
user-reduce    clara=5   native=5   oracle=5    ALL THREE MATCH
node-share     clara=20  native=20  oracle=20   ALL THREE MATCH
accum          clara=50  native=50  oracle=50   ALL THREE MATCH
deep-cascade   clara=200 native=200 oracle=200  ALL THREE MATCH
fanout         clara=400 native=400 oracle=400  ALL THREE MATCH
parametric-erasure  ⛔ NO CLARA TWIN
--- total 43s ---
```

**11/11 agree, 43 seconds, one JVM start per axis.** The check C9 calls *"never once run"* is a
43-second job. **No Clara program needs changing** — the 11 `gen-<axis>.sh` generators already emit
`:derived` in the same canonical encoding, byte-comparable to wat's.

## ⛔ AND THE ONE AXIS THAT MATTERS MOST IS THE ONE WITHOUT A TWIN

`parametric-erasure` is the axis that landed today carrying D7's trigger — the defect that cost this
arc a day. It has no Clara twin, so **the shape that beat us is the one shape Clara cannot referee.**

The first draft of this design excluded it, reasoning *"Clojure `defrecord` has no type parameters,
so the parametric declaration is not expressible."* **Builder struck that:**

> *"clojure doesn't have holon's vsa/hdc tooling either — we need to push our boundaries where they
> make sense to do so"*

And the objection does not survive contact. **Clara is a referee for rule semantics, not for wat's
type system.** The erasure is what wat does to the *declaration*; the *facts* are ordinary facts and
Clojure is dynamically typed, so `Box` instances with heterogeneous `v` are its native case. What
Clara must reproduce is the **derived set**, and that is exactly the ground truth that would have
named D7 independently. An absence in the reference tool is not a licence to leave a shape unchecked.

## The contract decision, pinned

**The third pairing becomes a low-volume gate, and the parametric axis gets a Clara twin — as a
STATIC `.clj`, not a `gen-` script.**

- One harness runs each axis at the **port gate's sizes** (not the perf ladder), compares
  **clara | oracle | native**, and names *which pair* diverged — the three diagnose differently
  (`oracle≠clara ⇒ SPEC wrong`, `native≠clara ⇒ fast path wrong`, `oracle≠native ⇒ PORT bug`).
- **`parametric-erasure.clj` is STATIC.** Verified on disk: `run-all.sh:85` discovers a perf axis as
  `<axis>.wat` **WITH** `gen-<axis>.sh`, so a static `.clj` creates no perf axis and needs **no
  LADDER rung**; `check-where-shapes.sh:140` globs `where-*.wat` only, so it is never swept there.
  Nothing in the tree globs `*.clj` broadly (checked). The 38 existing static `.clj` are the
  convention this follows.
- **Non-vacuity is a gate requirement**, carried over from the port gate: an empty set compares
  equal to an empty set and reports agreement while proving nothing.

## Out of scope = REJECTED

- **A `gen-parametric-erasure.sh`.** It would make the axis a *perf* axis, and `run-all.sh:88-99`
  then exits 2 without a LADDER rung — dragging a correctness shape into the perf artifact, whose
  sizes are deliberate and must not drift. The static `.clj` gets the coverage without the drift.
- **Running the oracle on the perf ladder.** That is the four hours, and it is the builder's named
  prohibition. These sizes are for correctness only.
- **Re-timing anything.** This strike compares SETS. `:clara-ns` and `:ratio` belong to the perf
  grid and are not this artifact's business.
