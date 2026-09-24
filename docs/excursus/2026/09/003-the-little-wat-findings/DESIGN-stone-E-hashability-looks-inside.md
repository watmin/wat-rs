# DESIGN — STONE E: the hashability check looks inside the key

**Drawn 2026-09-23.** Builder's ruling: *"the least amount of panics possible"*. Found by stone C's
executor after `cache.rs` reached zero `panic!`s.

## The defect, driven

```
Lru/put c (Option.Some <an Lru handle>) 1  ->  rc=2  panicked at src/value/value.rs:914:37
    internal error: entered unreachable code: Value::RustOpaque is not atomizable
```
`value_is_hashable` (`src/runtime.rs`, shared by HashMap, HashSet and the cache via
`value_is_key_hashable`/`value_is_set_hashable`) inspects only the key's OUTER variant.
`Option.Some(handle)` passes; `impl Hash for Value` (`src/value/value.rs:752`) then recurses into the
`Option` and reaches the handle's `unreachable!()` arm.

## What the tree already has — and why it is not enough

- **`impl Hash for Value`** recurses into: `Vec`, `wat__core__List`, `wat__std__HashSet`,
  `wat__std__HashMap`, `wat__core__PersistentMap`, `wat__core__PersistentVector`, `Tuple`, `Option`,
  `Result`, `Aggregate` (fields — unless an identity is stamped, see trap 2), `Enum`,
  `ForeignRecord`, `ForeignVariant` fields, `wat__core__clauses`. Fourteen variants are
  `unreachable!()`.
- **`Value::key_eligibility()`** (`value.rs:~1206`) — an EXHAUSTIVE per-variant classification
  (`Hashable` | `NeverAKey(InteriorMutable | OpaqueHandle | ExcludedByDesign)`), gate-tested by
  `all_key_eligibility()`. Careful and complete — but per VARIANT, i.e. shallow.
- **`value_is_hashable`** — a SECOND, hand-written list of the same fourteen variants. Also shallow.
  Two lists that can drift, and neither looks inside.

## The cure

**`value_is_hashable` becomes deep, and single-sourced:**

1. **Shallow test derived from `key_eligibility()`**, not from a hand list. ⚠ Read the reasons
   carefully: `ExcludedByDesign` variants (`List`, `PersistentMap`, `PersistentVector`) have REAL
   `Hash` arms — they are excluded from `is_atomizable`, not from hashing. The runtime question is
   "will `Hash` reach an `unreachable!()`", so only `InteriorMutable` and `OpaqueHandle` refuse.
2. **Recurse into exactly the variants `impl Hash` recurses into**, with an **exhaustive `match`
   and no `_ =>` arm** — so a new `Value` variant is a compile error in the predicate until someone
   decides whether it recurses. That is the top rung: drift has no representation.

Every caller — HashMap insert, HashSet conj, `Lru/put`/`get`, `edn/render.rs` — gets the deep check
for free, because they already call the shared predicate.

## The ONE contract decision

**The predicate mirrors `impl Hash`'s recursion, not `is_atomizable`'s.** A key the hasher can hash
is a legal key at runtime, whatever the holon rules say. Mirroring `is_atomizable` would refuse
`List`/`PersistentMap` keys that hash fine today — a rule that outlaws a truth.

## Out of scope — REJECTED

- A check-time (type-level) deep hashability rule — separate, and `is_atomizable` is not it.
- Where RAISED errors report their location (HashMap's `eval.rs:449`) — its own stone.
- Performance tuning. The deep walk is O(key size), the same order as hashing it; report a number
  if the floor moves, do not optimise.
