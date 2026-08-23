# DESIGN-STONE — split `out:query` without a Session rewrite

> **Origin (2026-08-22).** One-entry PMap harvest LANDED.
> harvest:query **16.91 → 7.71**. Leftover **out:query
> 3.08**. This stone prints the split. 2u already
> weighed the same 40k `VectorSync` on production:
> V−C **2.75**, from_iter **slower**, no intern.

## The enemy

`query_memory_to_pm` consumes `Vec<PMap>` and
`push_back_mut`s each as `PersistentMap` into an
`rpds::VectorSync`. Dual-impl WHAT is query-memory
= PersistentMap of name → vector of binding maps.
That vector is a PersistentVector. This stone does
not change it.

3.08 / 40,000 = **77 ns**/map. 2u's node-per-fact
was 81 ns. Same physics until a drop-in proves
otherwise.

## The algorithm

Tight loop. 40k pre-built one-entry PMaps (class-scan
shape). Mean of 3. Unscaled.

```
C  clone the 40k Vec<PMap>
V  clone + wrap + push_back_mut
H  clone + query_memory_to_pm          // authority
I  clone + VectorSync::from_iter
```

Treat **V − C** as the node-per-fact. Treat **H − V**
as the wrap (query-name map). Treat **V − I** as a
drop-in win.

1. If the largest drop-in is **< 1 ms**: stop. out:query
   is the rpds node. Do not change Session. Do not
   skip the freeze. Do not fuse harvest into
   VectorSync (that moves the mark, not the wall).
2. Else the one drop-in. Weigh `out:query`.

No new 40k phase marks. No fire-path change unless
step 2.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. Session still freezes
query-memory to a PersistentVector of binding maps.**
A faster build of that vector is the only intern this
stone may take.

## The gate

1. `out_query_cost_split` prints C / V / H / I.
   H > 0. Do not wall-gate FIRE.
2. If the stone implements: `fanout_three_leftover_split`
   still 40k maps. Do not wall-gate FIRE.
3. rete lib.
4. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): **V − C owns ~3 ms.**
H − V is small (one-entry query-name map, interned).
from_iter does not beat `push_back_mut` by 1 ms — 2u
already printed V−I **−0.68**. No intern. Say so.

## Blast radius

`kernel/tests.rs` only unless step 2. No `.wat`.
No Session field. No QueryMemory type change.

## Out of scope = REJECTED

- Native `Vec` in the frozen Session. Skip freeze.
- Intern `names`. Fuse harvest to move the mark. 297.
- A third PMap arm.

## Sequencing

1. Print. Rank.
2. Largest drop-in < 1 ms → stop.
3. Else the one intern. Weigh out:query. Stop.

## Weigh (2026-08-22) — LANDED, no intern

`out_query_cost_split` (40k one-entry PMaps, mean of 3):

| lump | ms |
|---|---:|
| C clone 40k Vec<PMap> | 2.15 |
| V clone + wrap + push_back_mut | 5.50 |
| H clone + query_memory_to_pm | 6.25 |
| I clone + from_iter | 5.99 |
| **V−C node-per-fact** | **3.36** |
| H−V wrap | 0.75 |
| V−I from_iter drop-in | **−0.49** |

Prediction held. V−C owns the 3 ms. Wrap < 1. from_iter is
*slower* than `push_back_mut`, same sign as 2u (−0.68).
No drop-in. Session stays a PersistentVector. Do not skip
the freeze. Do not fuse harvest (that moves the mark).
Do not intern `names`.

