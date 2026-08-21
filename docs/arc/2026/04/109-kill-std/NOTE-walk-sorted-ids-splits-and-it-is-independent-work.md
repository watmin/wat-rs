# NOTE (arc 109) — `walk-sorted-ids` should split into four. It is INDEPENDENT of the vec-of-types change.

**Filed 2026-08-20. MEASURED.** Answers: *"is this a bug fix the rete builder can work before they get
the parametrics-with-vec-of-types change on their branch?"*

**Yes.** It needs no new syntax, touches no `src/`, and can land on a branch that predates the bracket
work entirely. When the bracket change arrives, its annotations migrate like any other line.

## Why it exists — it is what killed stone ②a

Typing rete's memories made the floor go **2198/2620**, 2,907 occurrences of one root cause:

```
:wat::core::if: parameter else-branch
    expects  PersistentMap<i64, PersistentVector<wat::rete::Token>>
    got      PersistentMap<i64, PersistentVector<wat::core::Record>>     wat/rete.wat:2563
```

`walk-sorted-ids` threads ONE `acc` through phases returning three different memory types. Its own
comment records the constraint: *"Walking by index keeps Acc unparameterized."* **The memories cannot
be typed while the walker is polymorphic over them.** ~60 sites are gated on this one decision.

## ★ The polymorphism is STATIC — `phase` is a literal at every call site

```wat
new-amem      (walk-sorted-ids 0 facts network rules empty empty node-ids 0 empty)
new-bmem      (walk-sorted-ids 1 facts network rules new-amem empty node-ids 0 empty)
filtered-bmem (walk-sorted-ids 2 facts network rules new-amem new-bmem node-ids 0 new-bmem)
new-pmem      (walk-sorted-ids 3 facts network rules new-amem filtered-bmem node-ids 0 empty)
```

`wat/rete.wat:2619-2622`. Plus one recursive self-call (`:2564`) that threads `phase` through
unchanged. **There is no dynamic phase anywhere** — four monomorphic uses wearing one signature.

## The shape of the fix

Split into four, each with a concrete `acc`, and the `phase` parameter and its `cond` both disappear:

| new fn | phase body today | `acc` type |
|---|---|---|
| `walk-alpha-ids` | `activate-alpha facts network acc node-id` | map of node-id → `PV<Element>` |
| `walk-beta-ids` | `root-join-pass amem network acc node-id` | map of node-id → `PV<Token>` |
| `walk-filter-ids` | `hash-join-pass ∘ filter-pass ∘ accumulate-pass` | map of node-id → `PV<Token>` |
| `walk-prod-ids` | `production-pass network bmem rules acc node-id` | map of node-id → `PV<Record>` |

Each can also drop the parameters its phase never reads — phase 0 uses neither `amem` nor `bmem`;
phase 1 uses only `amem`; phase 3 only `bmem`. Optional, but it is the same edit.

★ **Bonus, and it may matter more to a rete builder than the typing does:** this removes a 4-arm
runtime `cond` from the engine's innermost recursive walker. Every node, every phase, every fire.

## ⚠ Why this is independent of the bracket change

The types can be written in **today's angle syntax**:

```wat
acc <- :wat::core::PersistentMap<wat::core::i64,wat::core::PersistentVector<wat::rete::Element>>
```

That checks now. The bracket migration will rewrite it like every other angle-form site — it is one
more line for a codemod, not a merge conflict of substance. **Nothing about this work needs to wait,
and doing it first is what unblocks ~60 sites of ②a.**

⚠ **One thing to verify before typing, because it is the trap ②a fell into:** the `alpha-memory` /
`beta-memory` doc comments at `wat/rete.wat:180`/`:185` describe a NESTED shape
(`node-id → {join-bindings → [Element …]}`) that **does not exist in the code**. Verified:
`join-bindings` appears exactly twice in the file, and both are those comments. The real shape is
FLAT — `activate-fact` assocs an `Element` directly under `alpha-id`. **Trust constructors and
consumers, not the prose.**

## ⚠ NOT part of this, and still open

`bindings`' value type is a separate, unresolved question. `Value` was tried and is WRONG — rete
compares binding values with `<`/`>`, uses them as map keys, and conjes them into vectors, which an
opaque `Value` refuses (arc 278 R7). See `SCORE-STONE-2a-rete-declares-its-types.md`. Do not fold that
question into this split.
