# BRIEF — gather buckets index `wm.alpha`, no clone

## The work

`GatherCache` clones `wm.alpha[id]` on every miss so buckets can
index a private `Vec`. After step 1 that vec does not grow this
round. Store only the `GatherIndex`. Probe `wm.alpha[id][i]`.
`accum:snapshot` at `[200 200]` must fall below 1 ms.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — this is #2.
2. `DESIGN-STONE-gather-no-snapshot.md` — contract.
3. `src/rete/kernel.rs` `type GatherCache` (~2220),
   `ensure_gather` (~2244), accumulate miss (~4262),
   `any_seeded_keyed` / `seeded_bindings_keyed`.
4. Confirm `wm.alpha.entry().push` is step 1 only in the
   fixpoint (today `:3854`). If a third push exists, STOP.

## Sketch

```rust
type GatherCache = HashMap<(i64, Vec<Value>), GatherIndex>;

fn alpha_elements(wm: &WorkingMemory, id: i64) -> &[Element] {
    wm.alpha.get(&id).map(Vec::as_slice).unwrap_or(&[])
}

fn ensure_gather(...) -> Vec<Value> {
    let els = alpha_elements(wm, alpha_id);
    let join_keys = gather_join_keys(sample, els);
    cache.entry((alpha_id, join_keys.clone())).or_insert_with(|| {
        census_count("accum:index-builds");
        census_count_n("accum:index-elements", els.len() as u64);
        build_gather_index(els, &join_keys)
    });
    join_keys
}
```

Accumulate: call `ensure_gather`, then `alpha_elements` under
the snapshot mark (no clone), then probe. Do not keep a second
miss path.

## Contract

1. Cache still keyed on `(alpha_id, join_keys)`.
2. Round-scoped. Never longer.
3. Empty alpha → empty gather. Empty bucket → identity / drop
   as today.
4. Order = `wm.alpha[id]` insertion order.

## STOP

1. **STOP-1** — a push to `wm.alpha` after step 1 in the same
   round. Report the site. Do not ship a stale suffix.
2. **STOP-2** — any rete differential red.
3. **STOP-3** — snapshot mark deleted to satisfy < 1 ms.

## Done

- `[200 200]` `accum:snapshot` < 1.0 ms.
- Builds still 2 / ≤ 80,000.
- Keyed-gather ratio ≤ 2.0.
- `binary_id(wat::rete)` green.
- clippy `--all-targets -D warnings`.

Leave dirty.
