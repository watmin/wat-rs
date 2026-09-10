# FINDING — ⛔ `acc::count` returns a WRONG COUNT when the `:from` inner carries a bind nothing consumes

**Driven 2026-09-10 at `dfd883886`.** Repro committed beside this finding at
`wat-scripts/scratch-pad/arc278-acc-count-unused-bind/probe-acc-count-unused-bind.wat` — both arms

> # ⛔ CORRECTED 2026-09-10 — "ACCUMULATE HAS ZERO CORPUS USES" IS FALSE. IT WAS MY GREP.
>
> I grepped for `rete::accumulate`. **The form has no `accumulate` keyword.** It is spelled
> `(?result <- (<acc-form>) :from (<inner>))` — and `src/rete/clause.rs:67` states that spelling
> verbatim, in a line I had already read and quoted. Measured properly: **37 `.wat` files carry a
> `:from` accumulate condition — 24 under `wat-scripts/`, 9 under `tests/`, 4 under `wat/`, and
> nineteen of them are GRID CELLS** (`accum.wat`, `accum-lead-derived.wat`, `accum-over-derived.wat`
> and siblings, each with a `.clj` Clara twin).
>
> **⭐ THIS MAKES THE QUESTION BETTER, NOT SMALLER.** The old framing — *"nothing was looking
> because nothing uses it"* — was a comfortable non-explanation. The truth is harder and more
> useful: **the grid HAS accumulate cells, compared three ways against Clara, and this defect
> survived them.** So the live question is not *why was nobody looking* but **what SHAPE does this
> defect need that no existing cell has** — an inert bind in the `:from` inner. That is `peragrare`
> exactly: the instrument was never asked this question, and a green from an instrument that was
> never asked is silence, not proof.
>
> ⛔ Fourth instrument error in one day, and the most consequential: this claim was load-bearing in
> two findings, a DESIGN, a probe header and two commit messages before it was checked. The others
> cost a re-run; this one shipped. `[[a-throwaway-sweep-is-an-instrument]]` — and the anchor I
> failed to build was the cheapest possible one: grep the corpus for the form's REAL spelling,
> which the type's own doc comment had given me.

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
