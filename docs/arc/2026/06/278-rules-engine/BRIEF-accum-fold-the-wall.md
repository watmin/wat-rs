# BRIEF — the fold is the wall

## The work

`accum:fold` is 68.49 ms of a 160.70 ms fire at `[200 200]` (42.6%).
The mark is the per-token bucket walk **plus** `accumulate_value`.
Count pays ~9.6 ms to rematch a keyed bucket into `Vec<&Element>`
and take `.len()`. Sum pays another ~9 ms of `acc_var_i64` /
`Bindings::get`. Skip rematch when the `:from` cond has no
`SeedCmp`. Fold `Element.bindings[slot]`. Count is `bucket.len()`.

## Read in order

1. **`NEXT-STRIKES-after-shadow.md`** — why this is #1, not persist.
2. **`DESIGN-STONE-accum-fold-the-wall.md`** — contract, leftover
   is the stop.
3. **`src/rete/kernel.rs` accumulate loop** (~4135–4274) — the
   `phase_start` of `accum:fold` through `accumulate_value`.
   Rematch is `fact_bindings_under` inside the bucket walk.
4. **`src/rete/kernel.rs` `acc_var_i64`** (~2314) — the 223 ns
   get. Replace on the fast path only.
5. **`src/rete/compiled_cond.rs` `CompiledCond`** — `ops`,
   `seed_reads`, `Op::SeedCmp`. Leftover is this, not a guess.
6. **`fold_cost_with_and_without_the_binding_lookup`** and
   **`accum_fire_phase_census`** — the instruments you re-point.

## Sketch

```rust
let leftover = from_compiled.map(CompiledCond::has_seed_cmp).unwrap_or(false);
if leftover {
    // today's rematch + acc_var_i64
} else if group_keys.is_empty() {
    match acc_fold {
        AccFold::Count => emit(tok, bucket.len() as i64),
        AccFold::Sum(var) => {
            let slot = operand_slot(from_elements, bucket, var);
            let s = bucket.iter().map(|&i| slot_i64(&from_elements[i], slot)).sum();
            emit(tok, s);
        }
        // min / max / mean: same slot, same empty-bucket contract
        _ => /* today's accumulate_value on a no-rematch gather */
    }
}
```

`operand_slot`: `from_elements[bucket[0]].bindings.iter().position(|(k,_)| k == var)`.
Empty bucket: count/sum emit 0; min/max/mean drop. No slot.

## Contract (do not "improve")

1. Leftover `SeedCmp` → rematch. Always.
2. Empty bucket → identity for count/sum, drop for min/max/mean.
3. Order = bucket index order = alpha insertion order.

## Blast radius

`src/rete/kernel.rs`. `compiled_cond.rs` only for `has_seed_cmp`.
No `.wat`.

## STOP

1. **STOP-1** — leftover path would have to change to make the
   census green. Report. Do not skip rematch "anyway."
2. **STOP-2** — any rete differential red, especially
   `where-accum-from-left` / leftover families.
3. **STOP-3** — slot stored on the interned `AccFold`. Derive
   from a live Element.

## Done

- `fold_cost_*` : sum ≤ 2× count; both far below today's 9.59 / 18.50.
- `accum_fire_phase_census` `[200 200]` : `accum:fold` mean < 25 ms.
- `cargo nextest run --release -E 'binary_id(wat::rete)'`.
- clippy `--all-targets -D warnings`.
- Report the printed fold table and the `[200 200]` fold ms.

Leave dirty. Orchestrator weighs.
