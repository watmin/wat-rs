# DESIGN-STONE — `Token.bindings` is a `BindSpan`

> **Origin (2026-08-18).** RHS split: compiled-rhs is a pile
> (bind-get 30%, stamp 28%). Bind-by-slot would fatten Token (2e).
> The bigger named leftover on fanout is `hj:catchup:probe` **14.35**.
> 2i left `extend_token` × 40k. That is two heap objects per join:
> `matches` Vec + `PMap` Array `Arc<Vec>`.

## The measurement

Fanout `[100 20]`: 2,000 left × 20 right = 40,000 extends.
`extend_token` today:

```
new matches Vec          // alloc
PMap::extend(...)        // clone left pairs + Arc + intern
```

Root-join already copied the Element pairs into a fresh PMap.
The keyed bucket already unified shared `?var`s.

## The algorithm

```
Token.bindings: BindSpan          // same pool as Element
root-join: share el.binds         // no copy
extend_token: append to bind_pool // left pairs + right-only keys
PMap: EDN / query / accum / sample boundary only
```

Readers take `pool_slice(tok.binds)`. `exec_compiled_rhs` is
`B: Bindings` (same door as `exec_where`). Explain encodes the
token **while the pool lives**. `WhereSample` materializes PMaps
at capture.

## ★ THE ONE CONTRACT DECISION

**Token bindings are fire-scoped indices, not a PMap per join.**
Same pool, same `BindSpan`, same law as Element (2f). Clone of
a Token copies the span. Do not fatten Token. Do not `unsafe`.

## The gate

1. `Token` has `binds: BindSpan`. `extend_token` appends. Root
   seed shares `el.binds`. Read the diff.
2. `fanout_fire_phase_census` `[100 20]`: print probe. Do not
   wall-gate FIRE.
3. rete lib + `binary_id(wat::rete)` (explain + `where-join-left`).
4. clippy `-D warnings`.

## Predicted win

`hj:catchup:probe` 14.35 → **~8–11** (matches Vec remains).
FIRE 60.90 → **~54–58**. If probe barely moves, leftover is
`matches` Vec + `key_of` — say so. Do not inline matches.

## Blast radius

`kernel.rs` Token + extend + encode/decode + accum assoc +
query harvest + explain encode. `compiled_rhs.rs` Bindings.
No `.wat`.

## Out of scope = REJECTED

- Bind-by-slot on PMap. Matches SmallVec / pool. Persist. 297.
- Unsafe. Fatter Token.

## Sequencing

1. BindSpan + append. Weigh. Stop.

## Weigh (2026-08-18) — LANDED

Fanout `[100 20]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 60.90 | **47.48** |
| `hj:catchup:probe` | 14.35 | **10.99** |
| hash-join | 15.97 | **12.00** |
| production | 25.31 | **21.83** |
| `prod:compiled-rhs` net | 8.78 | **6.36** |

Predicted probe 8–11: hit 10.99. FIRE beat the 54–58
prediction — Token clone into `d_beta` / emit is now a span
copy plus the matches Vec. Leftover probe is `matches` Vec
+ `key_of`. Do not inline matches (2e). Gate: rete lib 68,
`binary_id(wat::rete)` 299, clippy `-D warnings` silent.
