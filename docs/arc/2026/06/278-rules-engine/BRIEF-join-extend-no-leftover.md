# BRIEF — `join_extend` skips rematch when there is no leftover

## The work

Fanout catch-up rematches 40k keyed members. Skip when
`!has_seed_cmp()`. Same contract as fold-the-wall.

## Read in order

1. `DESIGN-STONE-fanout-phase-census.md` weigh.
2. `DESIGN-STONE-join-extend-no-leftover.md`.
3. `kernel.rs` `join_extend` (~1615). `compiled_cond.rs`
   `has_seed_cmp`.

## Sketch

```rust
if compiled.has_seed_cmp()
    && fact_bindings_under(el_fact, &tok.bindings, compiled, scratch).is_none()
{
    return Ok(None);
}
Ok(Some(extend_token(tok, el_fact, el_b, alpha_id)))
```

## STOP

1. **STOP-1** — skip rematch when `has_seed_cmp` (where-join-left).
2. **STOP-2** — rete differential red.
3. **STOP-3** — `right_idx` rewrite in this diff.

## Done

- Rematch gated. Census printed. rete + clippy.

Leave dirty.
