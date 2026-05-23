# DESIGN — Arc 226 — Type predicates as VSA similarity substrate operations

> **SPAWN-BLOCK STATUS (2026-05-22 late):** Arc 226 is spawned by arc 230 per `feedback_spawn_block_winding`. Surfaced as the type-system implication of the typed-entities doctrine.
> - **Arc 226 BLOCKS arc 230's INSCRIPTION**
> - **Arc 226's spawn children: arc 227** (user-defined types) spawns from arc 226 closure
> - The chain: arc 220 ← arc 221 ← arc 224 ← arc 225 ← arc 228 ← arc 230 ← arc 226 ← arc 227

**Opened:** 2026-05-22 (post-compaction, after typed-entities doctrine landing)
**Status:** STUB — full DESIGN ratified after arc 230 closes
**Depends on:** Arc 230 (substrate variant retirement) closes first — needs uniform classifier-wrap encoding across all typed entities (no PRIM_TAG-seeded variants left to differentiate from Bind-composition encoded forms).

## Mission

**Type checking emerges from VSA similarity.** Implement `(is-X? value)` predicates as substrate operations that measure similarity between an instance's classifier-atom vector and the prototype class-atom vector.

```
(is-Map? x)        ≡  similarity(extract-classifier(x), prototype-of("Map"))
(is-Keyword? x)    ≡  similarity(extract-classifier(x), prototype-of("Keyword"))
(is-MyType? x)     ≡  similarity(extract-classifier(x), prototype-of("MyType"))  ; works for any classifier
```

No class hierarchy. No method tables. No discrete dispatch. **Continuous answer.** Duck typing where the duck has a measurable shape.

## Triggering observation

User-articulated 2026-05-22 (post-compaction):

> *"declaring the classifier of the Bundle is brilliant... this means we can do type checking in holon space... we can ask 'is this thing a map?' and get a measurement answer - that's fucking insane - this is ruby on steroids"*

The type system fuses with the algebra. Polymorphic dispatch routes by classifier-similarity, not class-tree lookup.

## Scope (sketched; ratified post-arc-230)

1. **Classifier extraction primitive** — `extract-classifier(HolonAST) -> Option<String>` (or `Option<HolonAST>` if classifier is a composed name)
2. **Prototype class-atoms** — deterministic vector for each well-known classifier name; cached
3. **`is-?` predicate verb family** — `:wat::holon::is?` polymorphic OR per-type predicates (`is-Map?`, `is-Vector?`, etc.)
4. **Similarity threshold** — how close is "close enough"? Threshold tunable per call OR fixed substrate-default
5. **Polymorphic dispatch** — multimethod dispatch by classifier-similarity (extends arc 146/147 dispatch machinery)
6. **Test cascade** — every existing type-check site migrates from variant-pattern-match to similarity-probe

## What this arc does NOT do

- Add user-facing type declaration syntax (arc 227's territory)
- Touch substrate primitives (those are settled post-arc-230)
- Change parser or evaluator core (this is substrate-operation work)

## Cross-references

- arc 230 DESIGN.md — parent arc; pure-algebra encoding this arc operates on
- arc 227 DESIGN.md — spawn child; user-defined types build on this arc's predicate machinery
- arc 146/147 — existing dispatch machinery; this arc may extend it
- [[typed-entities-doctrine]] memory — the type-as-algebra doctrine
- INTERSTITIAL § 2026-05-23 evening — Ruby-on-steroids articulation
