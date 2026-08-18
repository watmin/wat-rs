# DESIGN-STONE — `Aggregate` carries its structural fingerprint

> **Origin (2026-08-18).** After bind-pool, `[200 200]` FIRE **67.33 ms**.
> Largest leftover: `setup:seen` **~8.8 ms**. Hasher 2d cut SipHash mix
> (13.26 → 8.17). Leftover is the Hash *walk* of 40k Aggregates
> (`nature` + `class` + `fields`). 2d refused a precomputed hash as
> a `Value` change. This stone is that change.

## The measurement

`seen.insert(f.clone())` for 40k input Records. Clone is a refcount.
`Value::Aggregate` Hash walks the payload every time. First hash of
each fact is at fire SETUP, not at construction.

The walk is a function of immutable EDN data (arc 294.c.1). It can
run once, at birth.

## The algorithm

```
AggregateValue.identity: u64     // private
from_parts: FxHash(nature, class, fields) → identity
Value::Hash Aggregate arm: identity.hash(state)   // one u64
PartialEq: unchanged (nature + class + fields)
```

All construction funnels through `from_parts` (`struct_` / `record` /
`holon_record` / assoc rebuild). Debug omits `identity` (it is a
cache, not EDN).

## ★ THE ONE CONTRACT DECISION

**Hash of an Aggregate is the fingerprint of its EDN data, computed
once at construction.** Eq is still the walk. Equal data ⇒ equal
fingerprint (constructors are the only writers). We do not hash
`Arc` identity. We do not skip inputs in `seen`.

FIRE `setup:seen` stops walking. Seed construction pays the walk.
`:ratio` is fire-only — say so if whole-eval does not move.

## The gate

1. `AggregateValue` has private `identity`. Hash writes it. Eq does
   not. Debug golden still matches (identity omitted).
2. `accum_fire_phase_census` `[200 200]`: fold < 25, snapshot < 1.
   `setup:seen` printed, **not** wall-gated.
3. rete lib + `binary_id(wat::rete)`.
4. clippy `-D warnings`.

## Predicted win

`setup:seen` 8.8 → **~2–4 ms**. FIRE 67.33 → **~61–65**. If
`setup:seen` stays ~8, leftover is HashSet insert of 40k Arcs, not
the walk — say so; do not add a second hasher.

## Blast radius

`src/value/value.rs` (field, constructors, Hash, Debug).
`src/runtime.rs` `Record/assoc` rebuild (the one raw literal).
No `.wat`. `seen` stays `FxHashSet<Value>`.

## Out of scope = REJECTED

- Second hasher. Pointer-hash. Enum/Foreign fingerprint (facts are
  Records). Persist. 297. Skipping `seen` inputs.

## Sequencing

1. `from_parts` + private field. Assoc uses it. Hash writes identity.
2. Weigh `setup:seen`. Stop.

## Weigh (2026-08-18) — LANDED, small

Eager-hash of *every* Aggregate made `insert` O(n²): each new
Session hashed the growing facts PV. **Stamp only a shallow
payload** (scalars / already-stamped nested Aggregates). Session
keeps `identity = 0` and Hash walks if anyone hashes a Session.

Census `[200 200]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 67.33 | **63.10** |
| `setup:seen` | ~8.8 | **6.89** |

The walk of 40k Records was ~2 ms. Leftover `setup:seen` is
HashSet insert of 40k Arcs. Do not add a second hasher.
