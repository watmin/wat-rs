# BRIEF — rustc-hash on fire-path maps that hash `Value`

## The work

`setup:seen` is 13.26 ms of SipHash + HashSet insert of 40k
input Aggregates. Put `rustc-hash` FxHash on `seen` and on
the gather maps whose keys are `Value`.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2c landed; this is 2d.
2. `DESIGN-STONE-setup-fxhash.md`.
3. `kernel.rs` `seen` fill, `GatherIndex` / `GatherCache`.

## Sketch

```rust
use rustc_hash::{FxHashMap, FxHashSet};

type GatherIndex = FxHashMap<Vec<Value>, Vec<usize>>;
type GatherCache = FxHashMap<(i64, Vec<Value>), GatherIndex>;

let mut seen: FxHashSet<Value> =
    FxHashSet::with_capacity_and_hasher(input.len(), Default::default());
for f in input.iter() { seen.insert(f.clone()); }
```

## STOP

1. **STOP-1** — pointer identity, or inputs omitted from `seen`.
2. **STOP-2** — rete differential red.
3. **STOP-3** — every `HashMap<i64, _>` rewritten, or `Value` Hash changed.

## Done

- `seen` + gather maps use FxHash. Census still green on
  fold/snapshot. rete + clippy.

Leave dirty.
