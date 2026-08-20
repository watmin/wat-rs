# DESIGN-STONE — intern cond slot keys once per fire

> **Origin (2026-08-20).** After 21: isolated M−T pile. Scratch
> **1.57 STOP**. **intern_key K−C 1.18** crossed 1 ms (stone 9:
> 0.86). 80,200 `intern_key` linear scans of two `?g`/`?v`
> Values. Unique keys intern once at SETUP; materialize copies
> `u32` ids. Not a process-lifetime intern of `?var` strings
> (2ad refused that).

## The measurement we have

`accum_materialize_split` today:

| lump | ms |
|---|---:|
| R−T scratch | 1.57 STOP |
| **K−C intern_key** | **1.18** |
| C−O clone | 0.75 |
| intern_val | 0.22 |

## The algorithm

At fire SETUP, after the arm:

```
for (id, cond) in compiled_conds {
    cond_key_ids[id] = intern_cond_keys(cond, bind_keys)
}
```

`intern_cond_keys` is `fact_bind?` then `slot_keys`, each
`intern_key` once. `materialize_into` takes `&[u32]` and
does not scan. Tests may pass `None` (still `intern_key`).
`bind_keys` still fire-scoped; still cleared. Token stays
two spans. Do not intern record `names`.

## ★ THE ONE CONTRACT DECISION

**A cond's bind keys are interned once per fire, not once
per matching fact.** Ids are fire-scoped `u32`s into
`bind_keys`. The arm does not store them.

## The gate

1. `accum_materialize_split` still prints K−C (isolated
   still scans). `accum_leftover_split` prints FIRE.
   Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): fire materialize drops
**~1.2 ms**. Isolated K−C stays ~1.2 (that arm still
scans). FIRE 20.93 → **~19.7**. `setup:seen` untouched.

## Blast radius

`compiled_cond.rs` `intern_cond_keys` + `materialize_into`.
`kernel.rs` SETUP table + `exec_compiled` callers on the
fire path. Isolated T/K arms unchanged. No `.wat`.

## Out of scope = REJECTED

- Process-lifetime `?var` intern. Intern `names`. Facts in
  `bind_pool`. Scratch repr. 2e / 2o. 297. Insertion.

## Sequencing

1. Table. Fire uses ids. Weigh FIRE. Stop.

## Weigh (2026-08-20) — LANDED

`accum_leftover_split` / `accum_materialize_split`, mean of 3.
Gate: rete lib 97, clippy `-D warnings` silent.

| lump | before | after |
|---|---:|---:|
| FIRE | 20.93 | **19.95** |
| honest_FIRE | 20.60 | **19.60** |
| isolated K−C intern_key | 1.18 | **0.97** (still scans) |

Predicted −1.2 on FIRE; measured **−0.98**. Isolated K still
calls `intern_key` per fact (STOP-3). Fire SETUP intern's
once; materialize copies `u32`. Scratch untouched. `bind_keys`
still fire-scoped.

Next leftover: scratch 1.57 STOP; clone 0.79; `accum:index`
1.97. Do not intern names. Do not start 297.
