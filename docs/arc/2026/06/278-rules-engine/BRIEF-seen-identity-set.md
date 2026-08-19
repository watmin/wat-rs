# BRIEF — `seen` of stamped facts is the fingerprint

## The work

Print `FxHashSet<Value>` vs `FxHashSet<u64>` on 40,200
stamped Records. If the cut is ≥ 1 ms, `seen` stores the
fingerprint for `identity != 0`. No second hasher.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2x is this stone.
2. `DESIGN-STONE-accum-leftover-split.md` weigh (`setup:seen` 7.43).
3. `DESIGN-STONE-aggregate-identity.md` weigh (leftover is insert).
4. `DESIGN-STONE-seen-identity-set.md`.
5. `kernel.rs` `setup:seen` / production `seen.insert`.

## Sketch

```rust
fn seen_identity_set_split() { /* C clone  S Value-set  I u64-set */ }

// only if S−I ≥ 1 ms:
// Seen { ids: FxHashSet<u64>, rest: FxHashSet<Value> }
```

## STOP

1. **STOP-1** — cut < 1 ms: do not touch `seen`.
2. **STOP-2** — second hasher / pointer-hash / skip inputs.
3. **STOP-3** — gate FIRE on a wall. Persist gather.

## Done

- Table printed. If implemented: Token still Copy.
  `setup:seen` printed. rete + clippy.

Leave dirty.
