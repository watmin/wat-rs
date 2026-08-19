# DESIGN-STONE — split `out:production` without a Session rewrite

> **Origin (2026-08-18).** 2t honest FIRE **28.42**. Drawable
> leftover that is not 2o / names / stamp: **OUT 3.26**, almost
> all `out:production`. That mark is `hashmap_to_pm(wm.production)`
> — one PersistentVector of 40k derived Pairs. Weigh before
> drawing. Do not intern `names`. Do not retry 2o.

## The measurement we do not have

`to_persistent` already uses `push_back_mut` (refcount-1
transient). 3.26 / 40,000 = **81 ns**/fact. That is an rpds
node per derived Pair, or the wrap (`from_trie` on a 1-key
map, which then demotes to the array arm). Rank V vs wrap
before touching the Session shape.

`production-memory` is a PersistentMap of PersistentVector.
That is the Session contract. This stone does not change it.

## The algorithm

Tight loop. 40k pre-built Pair records (same class as fanout).
Mean of 3. Unscaled — the count is the cell.

```
C  clone the 40k Vec                 // Arc bumps; subtract
V  clone + 40k push_back_mut
H  clone + hashmap_to_pm             // authority
I  clone + VectorSync::from_iter     // drop-in, if it compiles
```

Treat **V − C** as the node-per-fact. Treat **H − V** as the
wrap. Treat **V − I** as a drop-in win.

1. If the largest drop-in (V−I, or wrap if wrap ≥ 1 ms and
   replaceable) is **< 1 ms**: stop. OUT is the rpds node.
   Do not change Session. Do not skip the freeze.
2. Else the one drop-in. Weigh `out:production`. Token stays
   two BindSpans.

No new 40k phase marks. No fire-path change unless step 2.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. Session still freezes production
to a PersistentVector.** A faster build of that vector is the
only intern this stone may take.

## The gate

1. `out_production_cost_split` prints C / V / H / (I).
   H > 0. Do not wall-gate FIRE.
2. If the stone implements: Token is still two `BindSpan`s.
   `fanout_fire_phase_census` `[100 20]` prints OUT. Do not
   wall-gate FIRE.
3. rete lib.
4. clippy `-D warnings`.

## Predicted win

Independent guess (written first): **V − C owns ~3 ms.**
H − V is small (`from_trie` demote is an Arc clone of the
vector handle). from_iter does not beat `push_back_mut` by
1 ms. No intern. Say so.

## Blast radius

`kernel.rs` tests; `hashmap_to_pm` only if step 2. No `.wat`.
No Session field change. No Token field.

## Out of scope = REJECTED

- Native `Vec` in the frozen Session. Skip freeze. Skip stamp.
- Intern `names`. `Token.extra`. Persist. 297.

## Sequencing

1. Print. Rank.
2. Largest drop-in < 1 ms → stop.
3. Else the one intern. Weigh OUT. Stop.

## Weigh (2026-08-18) — LANDED, no intern

`out_production_cost_split` (40k Pairs, mean of 3):

| lump | ms |
|---|---:|
| C clone 40k Vec | 2.74 |
| V clone + push_back_mut | 5.49 |
| H clone + hashmap_to_pm | 6.27 |
| I clone + from_iter | 6.18 |
| **V−C node-per-fact** | **2.75** |
| H−V wrap | 0.78 |
| V−I from_iter drop-in | **−0.68** |

Prediction held. V−C owns the 3 ms. Wrap < 1. from_iter is
*slower* than `push_back_mut`. No drop-in. Session stays a
PersistentVector. Do not skip the freeze. Do not intern
`names`. Do not retry 2o.

The fanout cell has no drawable intern ≥ 1 ms that is not
2o-dead, names, stamp, or a Session rewrite.
