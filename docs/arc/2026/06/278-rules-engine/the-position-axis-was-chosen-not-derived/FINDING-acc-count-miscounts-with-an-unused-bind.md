# FINDING — ⛔ `acc::count` returns a WRONG COUNT when the `:from` inner carries a bind nothing consumes

**Driven 2026-09-10 at `dfd883886`.** Repro committed beside this finding at
`wat-scripts/scratch-pad/arc278-acc-count-unused-bind/probe-acc-count-unused-bind.wat` — both arms
in one file, so the number can be interpreted.

```
3 readings inserted -> plain=3  with-unused-bind=1   (both must be 3)
```

| rule | `:from` inner | derived `n` |
|---|---|---|
| `plain` | `(:Reading (?loc <- :location))` | **3** ✅ |
| `with-unused-bind` | `(:Reading (?loc <- :location) (?v <- :value))` | ⛔ **1** |

The **only** difference is `(?v <- :value)` — a binding **nothing reads**: not the acc-form, not a
constraint, not the `:then`. Adding it silently changes the engine's answer from 3 to 1.

## Why this is the most serious thing in this chain

Everything else this arc has found is a **missing check** — a program that should have been refused
and was not. This is a **wrong value**: a well-formed, fully-checked, legal rule that compiles,
fires, and returns the wrong number, with no diagnostic anywhere. A rules engine's whole contract
is the answer it returns.

## Scope, measured not assumed

- **`acc::sum` over the SAME two binds is unaffected** — `10+20+30` sums correctly, and filtered
  sums are correct too. So this is specific to the **count** fold's handling of the inner's binding
  set, not to having two binds.
- **Independent of any constraint.** The repro's `:from` carries no predicate at all.
- **Not a type-checking question**, and explicitly outside the `:from` validator strike
  (`6ddccec63`), whose DESIGN excludes the engine/reducer body.

**Suspected home, not confirmed:** `src/rete/kernel/fire/acc.rs` / `fire/pass/accumulate.rs`. ⚠ I
have not read either. The mechanism is unmeasured — knowing a number is wrong does not license a
fix, and the diagnosis needs its own measurement.

## How it was found, and why nothing was looking

By accident, during the `accumulate :from` strike: an executor wrote a positive fixture asserting
an exact count, the count came back wrong, and the first assumption was that the fixture or the new
validator was at fault. It was neither. The fixtures were switched to `acc::sum` to get past it.

⭐ **`:wat::rete::accumulate` has ZERO uses in the entire `.wat` corpus.** Nothing has ever driven
this fold outside the tests, so nothing could have noticed. The same property that hid the
`:from` validator hole hid this — and this one is worse, because a type hole needs someone to write
a wrong program before it bites, while this bites a correct one.

## What is NOT established

- **The mechanism.** Unread, unmeasured.
- **The blast radius.** Only `acc::count` vs `acc::sum` were compared, with one inert bind, at one
  arity. Whether other acc-forms miscount, whether two inert binds differ from one, and whether a
  *consumed* second bind is also affected are all unmeasured.
- **Whether the oracle agrees.** The wat oracle (`wat/rete/oracle/`) was not run against this. A
  differential would say whether this is a native-only defect or shared — and per
  `[[when-two-engines-disagree-neither-is-the-referee]]`, that is the first thing to measure, not
  the last.
