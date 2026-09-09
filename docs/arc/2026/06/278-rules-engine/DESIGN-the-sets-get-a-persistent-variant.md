# DESIGN — the sets get a persistent variant

**Substrate stone.** Mint `:wat::set::` over `rpds::HashTrieSetSync`. Road-paving for the
persistent-by-default migration, at the builder's ruling.

## The asymmetry

```
:wat::hashmap::   clones the whole map on assoc     →  :wat::map::   PersistentMap   SHARES   ✓
:wat::hashset::   clones the whole set on conj      →  (nothing)                              ✗
```

`src/collection/eval.rs:333-335` — the cloning set insert, sibling of the map's at `:367-369`:

```rust
let mut out: HashSet<Value> = (**s).clone();
out.insert(item.clone());
Ok(Value::wat__std__HashSet(Arc::new(out)))
```

★★ **The maps got a persistent variant; the sets kept the clone.** Same shape as this arc's headline
finding — *the reactor got the reads; the writes kept the 1970s* — a family where one member received
the fix and its sibling was never revisited.

## The doctrine is already written down

`src/intrinsic/map.rs`'s module doc, from arc 255 Stone E-i:

> *"WHY `PersistentMap` GETS THE UNMARKED `:wat::map::` NAME — this is the stone's whole point, not a
> style pick. The builder is moving to a persistent-backed default… Naming this family `:wat::map::`
> NOW means it never moves again once that swap lands — its name already IS what the default will be
> called."*

So this stone does not argue for the naming; it **applies a ruling already made** to the one family
that never received it. `:wat::set::` is persistent; `:wat::hashset::` keeps the cloning
implementation under its marked name.

## The dependency is already present and half-used

```
Cargo.toml:123        rpds = "1"          (resolved: rpds-1.2.1)
in use                rpds::HashTrieMapSync  ×38     rpds::VectorSync  ×15
available, unused     rpds::HashTrieSetSync          rpds::RedBlackTreeSetSync
PersistentSet / HashTrieSet in src/ or wat/    ZERO occurrences
```

`Value::wat__core__PersistentMap` and `Value::wat__core__PersistentVector` exist.
**`PersistentSet` is the missing third member of a family that is otherwise complete.**

## The surface

Mirror the shapes that already exist — `:wat::hashset::`'s four verbs plus the map's `dissoc` analogue:

| verb | mirrors |
|---|---|
| `:wat::set::conj` | `hashset::conj` / `map::assoc` |
| `:wat::set::disj` | `map::dissoc` |
| `:wat::set::contains?` | `hashset::contains?` / `map::contains-key?` |
| `:wat::set::empty?` | both |
| `:wat::set::length` | both |

**Five verbs. No more.** `:wat::map::` has eight because `get`/`keys`/`values` are map-shaped; a set has
no analogue and inventing one would be scope, not symmetry.

## The one contract decision

**`conj` and `disj` return a new set sharing structure with the old.** That is the entire point — an
`O(log n)` insert against the current `O(n)` clone. Everything else is a read.

## What this immediately unblocks

`every tier reports its backlog` introduced `seen-ids` as a `:wat::core::HashSet`, growing unbounded
with a whole-set clone per insert — measured at **no regression at n=2000**, but a quadratic term in an
instrument built to measure scaling. It reached for a cloning set because that was the only set on
offer.

⚠ **This stone does not fix that counter.** The better fix there is to count the visibility-expiry
transition the queue already performs, rather than infer a redelivery from set membership. Named,
separate, and behind this.

## OUT OF SCOPE — REJECTED

- **Retiring `:wat::hashset::`.** It keeps its marked name and its semantics, exactly as
  `:wat::hashmap::` did.
- **`RedBlackTreeSetSync`** — an ordered set is a different type with a different contract. Not now.
- **Migrating existing `:wat::hashset::` callers.** A separate, mechanical pass, and it belongs to a
  `wat/fix.wat` codemod rather than to this stone.
- **Fixing `seen-ids`.** Behind this, and the right fix there is not a set at all.

## Files

`src/intrinsic/set.rs` (new, mirroring `intrinsic/map.rs`), its registration, `src/collection/eval.rs`
(the five ops), `src/check.rs` (TypeSchemes), and a new `Value` variant — which ripples to every
exhaustive `match` on `Value`. **The compiler names those; the property is that it compiles and
round-trips through EDN, not that a list is complete.**
