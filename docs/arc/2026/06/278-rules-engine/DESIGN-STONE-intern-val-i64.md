# DESIGN-STONE — intern `intern_val` for i64

> **Origin (2026-08-19).** After 12: `setup:seen` ~3.9 is
> 2z fire context (do not intern without a new split).
> Largest internable leftover: `intern_val` **2.77** —
> FxHashMap get of `Value`, 80,200 successes, warm tax
> 0.22. Hashing `Value` (discriminant + i64) every time.
> Accumulators intern `?g` / `?v` i64s. This stone measures
> `Value` map vs `i64` map vs a small-int table, then
> interns the winner if the cut is ≥ 1 ms.

## The measurement we do not have

```
V  intern_val  FxHashMap<Value, u32>     // engine
I  intern_val  FxHashMap<i64, u32>
A  intern_val  slot table if range fits
```

Same 80,200 interned payloads, mean of 3. Print unique
count and i64 min/max. `V − min(I,A)` is the cut.

## The algorithm

1. Isolated V / I / A on the real interned Values.
2. **STOP** if the cut is **< 1 ms**. Leftover is HashMap
   get of a small key either way.
3. Else intern: i64 payloads use the winning table;
   other `Value`s stay on `FxHashMap<Value, u32>`.
   `BindView` still reads `bind_vals[id]`. Token stays
   two spans. Do not intern `names`. Do not put facts
   in `bind_pool`. Do not intern `seen`.

## ★ THE ONE CONTRACT DECISION

**A filler's intern id still names `bind_vals[id]`.** We
do not store i64 in the pool pair. We do not hash
`Arc` pointers. We do not skip intern for non-i64.

## The gate

1. `accum_intern_val_i64_split` prints V/I/A, unique,
   range. V > 0. Do not wall-gate FIRE.
2. If intern: V−K on `accum_materialize_split` falls.
   rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **I wins ~1.5–2 ms**
(no discriminant). A wins only if every interned i64 is
a small nonnegative. If V−I < 1, leftover is the get —
say so; do not intern.

## Blast radius

`compiled_cond.rs` `intern_val` + `WorkingMemory`
intern tables. `kernel.rs` twin `intern_val`. Tests
passing `FxHashMap::default()`. No `.wat`.

## Out of scope = REJECTED

- Tagged i64 in the u32 pool slot. Intern `names`. Facts
  in `bind_pool`. `seen`. 2e / 2o. 297. Insertion.
- Per-fact timers. `beta` hasher. Alpha-tree ranges.

## Sequencing

1. Print V/I/A. Rank.
2. Cut < 1 → stop.
3. Else intern the winner. Weigh V−K. Stop.

## Weigh (2026-08-19) — LANDED, small-int table

120,200 i64 fillers, min 0 max 999, other 0.

| lump | ms |
|---|---:|
| V FxHashMap\<Value\> | 1.81 |
| I FxHashMap\<i64\> | 0.44 |
| **A slot table** | **0.29** |
| V−A cut | **1.52** |

Prediction: I would win; **A won**. Interned `ValIntern`:
nonnegative i64 < 4096 index a `Vec<u32>`; else
`FxHashMap<Value>`. Pool pair still two u32s.
`accum_materialize_split` V−K **2.77 → 0.44**. `seen`
untouched. Next leftovers: `setup:seen` ~3.9, ops 1.90,
clone 1.02.
