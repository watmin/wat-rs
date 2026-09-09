# BRIEF — the sets get a persistent variant

## The work, in one paragraph

`:wat::map::` is `PersistentMap` and shares structure; `:wat::hashmap::` clones. The set family only has
the cloning half. Mint `:wat::set::` over `rpds::HashTrieSetSync` — five verbs, a new `Value` variant,
EDN round-trip — leaving `:wat::hashset::` exactly as it is. The dependency is already in `Cargo.toml`
and already supplies the map and vector we use.

## Read in order

1. **`src/intrinsic/map.rs`** — **the exemplar, end to end.** Arc 255 Stone E-i: how a collection family
   gets its own top-level namespace, how the verbs register, and — in its module doc — the recorded
   rationale for why the *persistent* member takes the **unmarked** name. Copy that structure.
2. **`src/collection/eval.rs:322-335`** — the current cloning set insert, including the guard that
   rejects opaque handles because they are not `Hash`. **Mirror that guard**; do not mirror the clone.
3. **`src/collection/eval.rs:360-370`** — `hashmap::assoc`'s clone, and the `PersistentMap` path near
   `:594` that shares. The contrast is the whole stone in ten lines.
4. **`src/runtime.rs:8977`** — where a `Value` maps to its wat type name. Your new variant needs a row.
5. **`rpds-1.2.1`** — `HashTrieSetSync` is the alias to use; `HashTrieMapSync` (38 sites) is how the
   codebase already holds one.

## Implementation sketch

```rust
// the variant, beside its siblings
Value::wat__core__PersistentSet(Arc<rpds::HashTrieSetSync<Value>>)

// conj — O(log n), shares structure. NO (**s).clone() anywhere.
let out = (**s).insert(item.clone());
Ok(Value::wat__core__PersistentSet(Arc::new(out)))

// disj — absent element returns the set unchanged, not an error
let out = (**s).remove(&item);
```

Five verbs only: `conj`, `disj`, `contains?`, `empty?`, `length`.

## Blast radius

`src/intrinsic/set.rs` (new), its registration, `src/collection/eval.rs`, `src/check.rs`,
`src/runtime.rs`, and **every exhaustive `match` on `Value`** — rendering, freezing, observation. Plus
one probe under `wat-scripts/scratch-pad/`.

⚠ **I am not giving you a list of the match sites, deliberately.** The compiler names them exactly and
my enumerations have been wrong three times in this arc. The property is that it compiles **without a
`_ =>` arm added to silence it** — see STOP-2.

## STOP triggers

**STOP-1** — if `conj` or `disj` requires cloning the whole set, **STOP**. That is the current
implementation with a new name, and the stone has no purpose.

**STOP-2** — do **not** add a catch-all `_ =>` arm to a previously-exhaustive `Value` match to make it
compile. Each site either handles the new variant or states at that site why it cannot. A wildcard hides
the rendering and freezing paths that genuinely need it.

**STOP-3** — do **not** touch `:wat::hashset::`, its verbs, or its callers. Its migration is a later
`wat/fix.wat` pass.

**STOP-4** — do **not** add `get`, `keys`, `values`, or an ordered set. Five verbs.

**STOP-5** — if the value cannot round-trip through EDN, **STOP and say why.** A collection that cannot
be written and re-read is not storable in a `defrecord` and cannot cross a service boundary, which
would make it useless for the migration this paves.

**STOP-6** — on any red floor arm: capture whole, name the arm, do not re-run.

## What "done" looks like

A probe in `wat-scripts/scratch-pad/` drives all five verbs and asserts an EDN round-trip.
`grep` of the diff finds **no whole-set clone** on the `:wat::set::` path. `:wat::hashset::` is absent
from the diff. `every_wat_scripts_file_loads` passes. Floor Summary reads 5235 plus the tests you add —
**state that number** — with 22 skipped, 0 FAIL, 0 TIMEOUT.

The SCORE should state which `Value` match sites needed the new case, and confirm no wildcard was added.
