# DESIGN-STONE — rustc-hash on fire-path maps that hash `Value`

> **Origin (2026-08-18).** Weigh after 2c. `setup-seen-once`
> landed the one-copy worklist. `[200 200]` FIRE **86.26 ms**.
> SETUP **13.40 ms** of which **`setup:seen` 13.26 ms**. That
> row is one SipHash + HashSet insert of 40k input Aggregates
> (~331 ns/insert). This stone is the hasher 2c named.

## The measurement

`Value::Aggregate` Hash is structural (`nature` + `class` +
`fields`, `value.rs`). `seen` is `HashSet<Value>` — std SipHash.
40k inserts, capacity reserved, no resize. Clone is a refcount.
The 13.26 ms is mix + probe, not a second `Vec`.

`accum:index` hashes the same kind of key (`Vec<Value>` join
tuples) into `GatherIndex`. Same hasher, same tax, different
mark.

## The algorithm

`rustc-hash` FxHash on the fire-path maps whose keys are
`Value` (or `Vec<Value>`):

```
seen:          FxHashSet<Value>
GatherIndex:   FxHashMap<Vec<Value>, Vec<usize>>
GatherCache:   FxHashMap<(i64, Vec<Value>), GatherIndex>
```

Membership is still `Value: Hash + Eq`. Inputs still enter
`seen` before any derived fact. First alpha pass still walks
the facts PV.

`i64`-keyed maps (`wm.alpha`, `d_alpha`, `d_beta`) stay std.
SipHash of an `i64` is not the 13 ms.

## ★ THE ONE CONTRACT DECISION

**`seen` is still a structural set of `Value`.** FxHash replaces
SipHash on that set and on the gather maps. We do not hash
`Arc` identity. We do not drop inputs from `seen`. We do not
precompute a hash on `Aggregate` (that is a `Value` Hash
change, not this stone).

Fire-path maps are filled with facts the session already
accepted. DOS-resistance is not the contract here.

## The gate

1. `seen` is `FxHashSet<Value>`. `GatherIndex` / `GatherCache`
   use `FxHashMap`. Read the diff.
2. `accum_fire_phase_census` `[200 200]`: fold < 25, snapshot
   < 1. `setup:seen` is printed, **not** wall-gated.
3. rete lib + `binary_id(wat::rete)`.
4. clippy `-D warnings`.

## Predicted win

`setup:seen` 13.26 → **~5–8 ms**. `accum:index` may move a
few ms (same hasher, 80k keys). Quiet FIRE 86 → **~78–82**.
If `setup:seen` stays ~13, leftover is the Hash *walk* +
HashSet insert, not the SipHash mix — say so; do not add a
second hasher.

## Blast radius

`wat-rs/Cargo.toml` (`rustc-hash = "2"`). `src/rete/kernel.rs`:
`seen`, `GatherIndex`, `GatherCache`, `build_gather_index`.
No `.wat`. No `impl Hash for Value` change.

## Out of scope = REJECTED

- Persist gather. Cross-call TM.
- Rewriting every `HashMap<i64, _>` in the kernel.
- Pointer-hash / omitting inputs from `seen`.
- Precomputed hash on `Aggregate`.
- `ahash` (AES-NI, not what 2c named).

## Sequencing

1. Add the crate. Alias the three maps.
2. Weigh `setup:seen` and `accum:index`. Stop.
