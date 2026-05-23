# DESIGN — Arc 227 — User-defined types via classifier-wrap (wat-level type-declaration mechanism)

> **SPAWN-BLOCK STATUS (2026-05-22 late):** Arc 227 is spawned by arc 226 per `feedback_spawn_block_winding`. The last arc in the typed-entities doctrine implementation chain.
> - **Arc 227 BLOCKS arc 226's INSCRIPTION**
> - **Arc 227 has no anticipated spawn children**
> - The chain: arc 220 ← arc 221 ← arc 224 ← arc 225 ← arc 228 ← arc 230 ← arc 226 ← arc 227

**Opened:** 2026-05-22 (post-compaction)
**Status:** STUB — full DESIGN ratified after arc 226 closes
**Depends on:** Arc 226 (type predicates via VSA similarity) closes first — needs the predicate machinery to make user-defined types queryable.

## Mission

**User-defined types are unlimited via classifier-wrap. The substrate doesn't need to know about them.** Mint a wat-level type-declaration mechanism that creates new classifier atoms; instances of user types become `(Bind (Atom <UserClassName>) (Atom <data>))` compositions; queryable via the arc 226 predicate machinery.

```
(defclass Voltage Float)             ; user declares a new type
(Voltage 5.0)                        ; user constructs an instance
(is-Voltage? x)                      ; user queries via VSA similarity
```

Per the typed-entities doctrine: *user-surface is unlimited*. The 12 true substrate primitives suffice for any user-defined type universe.

## Triggering observation

User-articulated 2026-05-22 (post-compaction):

> *"user-defined types are unlimited. Users invent classifier names; the substrate doesn't need to know."*

`(Voltage 5.0)`, `(Celsius 273.15)`, `(BasisPoint 25)` — all first-class via classifier-wrap. No nominal type system; no class hierarchy; no method tables. Type-checking is VSA similarity.

## Scope (sketched; ratified post-arc-226)

1. **`defclass` / `deftype` wat-level macro** — accepts class name + optional base type; mints the classifier; registers prototype for similarity
2. **Constructor verb auto-generated** — `(defclass Voltage Float)` creates `:user::Voltage` (or similar) constructor
3. **Inheritance via classifier-chain** — `(defclass U8 Int)` produces `(Bind (Atom "U8") (Bind (Atom "Int") (Atom value)))` — instances queryable as either U8 or Int via similarity
4. **Method dispatch integration** — multimethods route by classifier-similarity per arc 226 machinery
5. **Documentation** — USER-GUIDE chapter on user-defined types via classifier-wrap

## What this arc does NOT do

- Modify substrate primitives (settled post-arc-230)
- Change type-checking core (arc 226's territory)
- Implement nominal type hierarchies (those are NOT the model)

## Cross-references

- arc 226 DESIGN.md — parent arc; predicate machinery this arc consumes
- [[typed-entities-doctrine]] memory — unlimited-user-types articulation
- INTERSTITIAL § 2026-05-23 evening — Voltage / Celsius / BasisPoint examples
