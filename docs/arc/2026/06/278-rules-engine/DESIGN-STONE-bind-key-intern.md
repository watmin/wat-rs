# DESIGN-STONE — bind_pool keys are a fire-scoped index

> **Origin (2026-08-19).** 2ac: match_pool is `(u32, i64)`. Isolated
> leftover drop is **B 0.78** — `bind_pool` of 80,400 `(String, i64)`
> pairs. 2aa already named B as the key Arc. This stone thins the
> key, not the value. Do not intern `names` (record field-name
> Arc). Do not put facts in `bind_pool`.

## The measurement

`bind_pool: Vec<(Value, Value)>`. Populate does
`slot_keys[i].clone()` per Element. Extend copies the pair.
Drop decrements 80k String Arcs. The unique keys on accum are
`?g` and `?v` — two Values. The extra refs are the bill.

A process-lifetime intern of record `names` is a different
stone and is rejected. A fire-scoped table of bind *variable*
keys is the 2ab analog.

## The algorithm

```
bind_keys: Vec<Value>            // unique, intern on write
bind_pool: Vec<(u32, Value)>     // key id, filler

intern(k): linear scan bind_keys (tiny); clone k once
populate:  pool.push((intern(slot_keys[i]), v))
extend:    copy (u32, Value); compare ids
drop-memories: bind_pool.clear(); bind_keys.clear()
```

Readers take `BindView { keys, pairs }` which impls `Bindings`.
Token stays two `BindSpan`s.

## ★ THE ONE CONTRACT DECISION

**A bind pair does not own its key.** A `u32` names a slot in
the fire-scoped `bind_keys` table. Unique keys are cloned
once. We do not skip Drop of the table. We do not intern
record `names`. We do not put facts in `bind_pool`.

## The gate

1. `bind_pool` is `Vec<(u32, Value)>`. Populate intern-writes.
2. `accum_fire_phase_census` `[200 200]`: fold < 25,
   snapshot < 1. drop printed, **not** wall-gated.
3. rete lib.
4. clippy `-D warnings` (`--lib`).

## Predicted win

Isolated B 0.78 → **~0**. Isolated D 0.77 → **~0**. in-fire
drop 1.18 → **~0.4** (fire context). FIRE 53.83 → **~52–53**
(populate `key.clone()` dies). If FIRE does not fall, leftover
is bind *value* Drop / fire context — say so. Do not revert
on alpha-instrument wash (2ac).

## Blast radius

`kernel.rs` (`bind_pool`, `bind_keys`, `BindView` readers,
extend, encode/decode, drop census). `compiled_cond.rs`
populate. `matcher.rs` `Bindings` for `BindView`.
`compiled_rhs.rs` slot walk. No `.wat`. No crate. No `unsafe`.

## Out of scope = REJECTED

- Intern `names` (record field-name Arc). Facts in `bind_pool`.
- Process-lifetime intern of `?var` strings.
- Inline-enum (2e). Two-span get (2o). Arena-and-forget.
- Persist gather. 297.

## Sequencing

1. Table + `(u32, Value)` pool. Populate intern-writes.
2. Weigh FIRE and drop. Stop.

## Weigh (2026-08-19) — LANDED

Isolated `drop_memories_cost_split` (40,200, mean of 3):

| lump | before | after |
|---|---:|---:|
| A | 0.00 | 0.00 |
| **B drop bind_pool** | **0.78** | **0.32** |
| M | 0.00 | 0.00 |
| T | 0.00 | 0.00 |
| D | 0.77 | **0.37** |

Key Arc died. Leftover B is 80k `Value::i64` Drops, not keys.

Census `[200 200]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 53.83 | **48.52** |
| `round:drop-memories` | 1.18 | **0.46** |
| fold | 1.50 | **1.05** |
| snapshot | 0.00 | **0.00** |

Populate `slot_keys[i].clone()` died too — FIRE fell 5.31. Token stayed two spans. Do not intern record `names`. Do not put facts in `bind_pool`. Leftover isolated drop is value Drop (0.32).
