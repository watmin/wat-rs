# DESIGN-STONE — bind_pool fillers are a fire-scoped index

> **Origin (2026-08-19).** 2ad: keys are `u32`. Isolated leftover
> drop is **B 0.32** — 80,400 `Value::i64` Drops in `bind_pool`.
> This stone thins the filler. Do not intern record `names`.
> Do not put facts in `bind_pool`.

## The measurement

`bind_pool: Vec<(u32, Value)>`. Populate moves a `Value` per
pair. Extend clones it. Drop visits 80k enum slots (~4 ns
each = 0.32 ms). On accum the unique fillers are `?g` and
`?v` i64s — hundreds, not 80k. The extra copies are the bill.

Key intern was a linear scan (two `?var`s). Filler intern
cannot be: unique i64s are ~1k, 80k × linear is a FIRE
disaster. `FxHashMap<Value, u32>` is the 2d hasher we
already pay for `seen`.

## The algorithm

```
bind_keys:    Vec<Value>                 // unique keys (2ad)
bind_vals:    Vec<Value>                 // unique fillers
bind_val_ids: FxHashMap<Value, u32>      // intern
bind_pool:    Vec<(u32, u32)>            // key id, val id — Copy

intern_val(v): map get; else clone once into vals
populate:  pool.push((intern_key(k), intern_val(v)))
extend:    copy (u32, u32)
drop-memories: pool / keys / vals / val_ids clear
```

Readers take `BindView { keys, vals, pairs }`. Token stays
two `BindSpan`s. `?p` fact-bind intern-writes the fact
Value into `bind_vals` (that is a filler, not a fact store).

## ★ THE ONE CONTRACT DECISION

**A bind pair does not own its filler.** A `u32` names a slot
in the fire-scoped `bind_vals` table. Unique fillers are
cloned once. We do not skip Drop of the table. We do not
intern record `names`. We do not put facts in `bind_pool`.

## The gate

1. `bind_pool` is `Vec<(u32, u32)>`. Populate intern-writes.
2. `accum_fire_phase_census` `[200 200]`: fold < 25,
   snapshot < 1. drop printed, **not** wall-gated.
3. rete lib.
4. clippy `-D warnings` (`--lib`).

## Predicted win

Isolated B 0.32 → **~0**. Isolated D 0.37 → **~0**. in-fire
drop 0.46 → **~0.1**. FIRE 48.52 → **~48–49** (i64 clone was
cheap; extend becomes Copy). If FIRE does not fall, leftover
is fire context — say so. Do not revert on alpha-instrument
wash (2ac).

## Blast radius

`kernel.rs` (`bind_pool`, `bind_vals`, `bind_val_ids`,
`BindView` readers, extend, encode/decode, drop census).
`compiled_cond.rs` populate. `matcher.rs` `BindView`.
`compiled_rhs.rs` tests. No `.wat`. No crate. No `unsafe`.

## Out of scope = REJECTED

- Intern `names`. Facts in `bind_pool`.
- Process-lifetime intern of fillers.
- Linear-scan intern of values (FIRE disaster).
- Inline-enum (2e). Two-span get (2o). Arena-and-forget.
- Persist gather. 297. Fact insertion (parked).

## Sequencing

1. Table + `(u32, u32)` pool. Populate intern-writes.
2. Weigh FIRE and drop. Stop.

## Weigh (2026-08-19) — LANDED

Isolated `drop_memories_cost_split` (40,200, mean of 3):

| lump | before | after |
|---|---:|---:|
| A | 0.00 | 0.00 |
| **B drop bind_pool** | **0.32** | **0.00** |
| M | 0.00 | 0.00 |
| T | 0.00 | 0.00 |
| D | 0.37 | **0.00** |

Copy pairs. Unique fillers live in `bind_vals`; isolated B does not time that table and went to 0.

Census `[200 200]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 48.52 | **49.13** |
| `round:drop-memories` | 0.46 | **0.01** |
| fold | 1.05 | **0.68** |
| snapshot | 0.00 | **0.00** |

FIRE wash (alpha instrument). Isolated B is the ranking. Do not revert (2ac). Token stayed two spans. Do not intern record `names`. Do not put facts in `bind_pool`. Isolated drop of bind storage is exhausted.
