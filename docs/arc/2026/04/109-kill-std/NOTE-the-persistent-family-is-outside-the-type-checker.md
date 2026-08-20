# ⛔ NOTE — the Persistent collections are not under-annotated. They are OUTSIDE THE TYPE CHECKER. 13 verbs.

**Filed 2026-08-20. MEASURED, with controls.** Found while answering a narrow question — *"`{?var →
value}`… that's `PersistentMap<String,Value>`?"* — during step ②a's first role determination.

## The measurement

Every `PersistentMap/*` and `PersistentVector/*` verb, handed SEVEN arguments, against their std twins:

```
16  HashMap/* · Vector/*              ArityMismatch     ✓ enforced, real TypeSchemes
13  PersistentMap/* · PersistentVector/*   check PASSES ⛔ blanket-accepted

    PersistentMap/    assoc · contains-key? · dissoc · empty? · get · keys · length · values
    PersistentVector/ conj · contains? · empty? · get · length
```

`HashMap/get` is registered at `check.rs:19990` — `∀K,V. HashMap<K,V> × K -> Option<V>`.
**`PersistentMap/get` is not registered anywhere.** It falls through `check.rs:5561`'s
*"silent-by-intent — no scheme found for multi-arg form; accept and pass"*, which returns a **fresh
type variable**: args unchecked, arity unchecked, V never propagated.

## ★ Consequence: the type annotation is not merely absent, it is INERT

Same probe, matched accessors, value used as a `String` when declared `i64`:

```
HashMap        V=i64  → exit 1    ✓ REJECTED
PersistentMap  V=i64  → exit 0    ⛔ ACCEPTED
```

So **annotating rete's 238 bare heads would be COSMETIC** — a `(PersistentMap [String i64])` is not
enforced at any accessor today. Wall 2 alone documents; it does not constrain.

⚠ **My first probe of this reported "V is not enforced for EITHER family" — that was FALSE.** It
called `PersistentMap/get` on a `HashMap`; the mismatched accessor fell through to a fresh var, so the
probe measured nothing. The positive control (`string::trim 42` → exit 1) is what exposed it. Eighth
instrument error of the session, same family as the rest.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

## ★★ THIS IS ARC 255's #110 — thirteen named victims beside `peer-pid`

The 294 seam records `peer-pid` as the blanket-accept's *"first named victim"* — 18 call sites, zero
mentions in `check.rs`, on the capability path. **These 13 are the same defect at family scale**, and
they carry the substrate's hottest data structure: rete's Token bindings, alpha/beta memories, and the
node network all flow through verbs the checker does not see.

`255.1b-iv` (#110) is already ruled as preceding the `wat.core/` flip. **It now also precedes any
honest fix to the persistent collections.**

## ⛔ Consequence for ORDERING — step ②a must not proceed as drawn

`DESIGN-STONE-wall-2-the-unwritable-238.md` schedules the 238 bare heads first because their errors
carry no remedy. That reasoning still holds. **But the stone assumed the annotations would constrain
something.** They will not, until the 13 verbs have schemes.

Two honest orders, and this is the builder's call:

1. **Schemes first.** Give the 13 verbs real `TypeScheme`s (mirroring their std twins — `HashMap/get`
   at `check.rs:19990` is the worked template), THEN annotate the 238. The annotations land into a
   checker that enforces them, and every wrong one screams immediately.
2. **Annotate first anyway.** The 238 become true documentation now and enforced later. Cheaper per
   step, but nothing validates them at the time of writing, so a wrong K/V ships silently and looks
   compliant — the exact failure mode the wall-2 stone's contract decision exists to prevent.

**Recommendation: (1).** Writing 238 annotations that nothing checks is how the persistent family got
here in the first place — a convention that was never a check. Doing it again, deliberately, with the
mechanism known and available, would be the same heresy in better handwriting.

## Held open, deliberately unanswered

The question that started this — **what IS the honest V for rete's `bindings`?** — is NOT answered
here, and `Value` is not the answer:

> `types.rs:1160`, arc 278 R7, a recorded CORRECTION of exactly this move: *"declared the payload as
> the bare `:wat::core::Value` (the universal top)… **That was wrong: UP is free, DOWN is CHECKED**,
> so a `:wat::core::Value` payload can be PRODUCED but never CONSUMED — no accessor type-checks
> against an opaque `Value` receiver, by design."* Its resolution was to go **parametric over `T`**.

K is settled: **`:wat::core::String`** — `rete.wat:146` already declares `result-var <-
:wat::core::String` for the same `?var` names, and every literal key measured is a String
(`"?count"`, `"?fact"`, `"?inner"`, `"?label"`, `"?s"`, `"?t"`).

V is a real design question: a rete binding holds whatever a fact's field held. Under R7 that is
neither `Value` nor any single concrete type, which points at `Token<V>` — and that cascades through
the engine. **That is a finding, not a fill-in**, exactly as the wall-2 stone's contract decision
requires.
