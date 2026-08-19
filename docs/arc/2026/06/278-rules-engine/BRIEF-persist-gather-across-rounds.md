# BRIEF — persist gather indexes across rounds

## The work

`gather_cache` lives outside the round loop, like P6
`left_idx`. After alpha, append `d_alpha` into existing
buckets. `ensure_gather` builds only on miss. Fire-scoped,
not a Session field. Do not persist to dodge the fold.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 3 is this persist.
2. `DESIGN-STONE-gather-index-cache.md` — key is
   `(alpha_id, join_keys)`; round-scoped was the wall.
3. `DESIGN-STONE-P6-persistent-keyed-memories.md` — append Δ.
4. `DESIGN-STONE-keyed-gather.md` out-of-scope (persist was
   a second stone).
5. `DESIGN-STONE-persist-gather-across-rounds.md`.
6. `kernel.rs` round loop: `gather_cache` vs `left_idx`.

## Sketch

```rust
let mut gather_cache: GatherCache = Default::default(); // outside loop
// after alpha:
append_d_alpha(&mut gather_cache, &d_alpha, wm);
```

## STOP

1. **STOP-1** — persist to dodge the fold. Session-stored.
2. **STOP-2** — key on `alpha_id` alone. Rebuild not append.
3. **STOP-3** — intern `names` / facts in `bind_pool` / 2e / 2o.
4. **STOP-4** — gate FIRE on a wall. 297. Fact insertion.

## Done

- Census names rounds on accum `[200 200]`.
- `gather_cache` outlives the round. Append `d_alpha`.
- `accum:index-builds` ≤ 2 on that cell. rete lib green.
  clippy `-D warnings` silent.

Leave dirty.
