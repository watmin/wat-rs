# BRIEF — leftover `setup:seen` is the facts-vector walk

## The work

Print PersistentVector iter vs Vec iter on 40,200 stamped
Records plus `seen_insert`. If P − V ≥ 1 ms *and* D + V wins
by ≥ 1 ms, first worklist is a transient Vec. Frozen Session
stays a PersistentVector.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2z is this stone.
2. `DESIGN-STONE-seen-identity-set.md` weigh (leftover 4.31).
3. `DESIGN-STONE-setup-seen-once.md` (first worklist is facts).
4. `DESIGN-STONE-seen-pv-walk.md`.
5. `kernel.rs` `setup:seen` / `input_facts.iter()`.

## Sketch

```rust
fn seen_pv_walk_split() { /* W I V P D */ }
```

## STOP

1. **STOP-1** — P − V < 1 ms, or D + V does not beat P by 1 ms:
   do not change facts representation.
2. **STOP-2** — Vec in frozen Session / skip `seen` / persist.
3. **STOP-3** — gate FIRE on a wall.

## Done

- Table printed. If implemented: Token still Copy.
  `setup:seen` printed. rete + clippy.

Leave dirty.
