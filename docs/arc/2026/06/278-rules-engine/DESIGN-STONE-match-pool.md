# DESIGN-STONE — `Token.matches` is a span

> **Origin (2026-08-18).** Token bind-pool landed. Fanout `[100 20]`
> FIRE **47.48**. Probe **10.99**. That mark is 2,000 `key_of`s and
> **40,000** `extend_token`s. `key_of` is the small count. The 40k
> row is still a `matches` Vec per join.

## The measurement

`extend_token` after 2m:

```
new matches Vec          // alloc × 40k
extend_from_slice
push (el_fact.clone(), alpha_id)
append binds into bind_pool
```

2e inlined width 0–2 and fattened the value. 2f / 2m thinned
with indices. Same law here. Do not SmallVec.

## The algorithm

```
WorkingMemory.match_pool: Vec<(Value, i64)>   // append-only during fire
Token { matches: BindSpan, binds: BindSpan }  // both Copy

root-join: push one edge; Token.matches is that span
extend_token: append left edges + (fact, alpha_id)
PMap / Vec: EDN / explain boundary only
```

Explain encodes while the pool lives (already true for binds).
`WhereSample` does not need matches.

## ★ THE ONE CONTRACT DECISION

**Token edges are fire-scoped indices, not a Vec per join.**
Same pool law as bindings. Clone of a Token copies two spans.
Token is `Copy`. Do not inline `[T; 2]`. Do not skip matches
on `fire-rules`.

## The gate

1. `Token.matches` is `BindSpan`. `match_pool` exists. Root
   seed pushes one edge. Extend appends. Read the diff.
2. `fanout_fire_phase_census` `[100 20]`: print probe. Do not
   wall-gate FIRE.
3. rete lib + `binary_id(wat::rete)` (explain walks matches).
4. clippy `-D warnings`.

## Predicted win

`hj:catchup:probe` 10.99 → **~6–9**. FIRE 47.48 → **~42–45**.
Leftover is `key_of` + fact clone + bind append. If probe
barely moves, say so.

## Blast radius

`kernel.rs` Token + extend + encode/decode + explain.
No `.wat`. No SmallVec.

## Out of scope = REJECTED

- SmallVec / `[T; 2]`. Skip matches on fire-rules. Persist. 297.
- Unsafe. Bind-by-slot.

## Sequencing

1. Span + pool. Weigh. Stop.

## Weigh (2026-08-18) — LANDED as type; probe wash

Fanout `[100 20]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 47.48 | **44.23** |
| `hj:catchup:probe` | 10.99 | **11.39** |
| production | 21.83 | **19.34** |
| hash-join | 12.00 | **12.17** |
| `hj:catchup:emit` | 0.25 | **0.09** |

Token is `Copy`. Probe did not fall (spread [8.79–14.25]). The
40k matches Vec was not the 11 ms. Leftover probe is bind-pool
append + `key_of` + fact clone. FIRE fell ~3 ms on Token copy
into `d_beta` / emit. Do not SmallVec. Do not retry matches.

Stale `node_share_fire_phase_census` filter-dominates wall
dropped (ShadowNode already killed that share). Gate: rete
lib 68, `binary_id(wat::rete)` 299, clippy silent.
