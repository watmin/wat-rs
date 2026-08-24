# DESIGN-STONE — promoting PersistentVector

> **Origin (2026-08-22).** `out:query` split: V−C **3.36 ms**
> is 40k RRB `push_back_mut`. `from_iter` of `VectorSync`
> is the same loop (V−I **−0.49**). Session stays a
> PersistentVector. This stone internes the vector the
> way `PMap` interned the map.

## The enemy

`Value::wat__core__PersistentVector` holds
`rpds::VectorSync` directly. Freeze of query-memory
(and production) is one RRB assoc per element.
PMap already has an array arm; PVec does not.

Vectors index. There is no “must stay ≤ 8” rule.
Bulk `from_vec` can stay `Arc<Vec<Value>>` at any
length. Persistent `conj` (`push_back`) of a large
array promotes to the RRB so wat-level conj stays
O(log n), not O(n²).

## The algorithm

```
PVec
  Array(Arc<Vec<Value>>)   from_vec / unique push_back_mut / from_iter
  Tree(VectorSync)         after persistent conj past 8

from_vec(xs)     → Array, any length
push_back_mut    unique Array stays Array (Vec::push)
push_back        Array len ≥ 8 → Tree then RRB conj
Eq / Hash        elements in order, never the arm
```

Freeze calls `from_vec`. Dual-impl WHAT is unchanged.

## ★ THE ONE CONTRACT DECISION

**Representation is unobservable.** Two vectors with
the same elements in the same order are equal
whichever arm holds them. Session still holds a
PersistentVector. Do not Session-Vec. Do not skip
freeze.

## The gate

1. `from_vec` of n equals a Tree of the same
   elements. Hash agrees. A vector used as a map
   key is found across arms.
2. `out_query_cost_split` H−C drops ≥ 1 ms vs the
   4.10 ms (6.25−2.15) authority wrap+tree, **or**
   `fanout_three_leftover_split` out:query drops
   ≥ 1 ms vs 3.08. query-maps 40,000.
3. rete lib. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): out:query
**3.08 → ~0.8** (wrap into Vec + one Arc). with-query
wall **40.35 → ~38**. production freeze the same
shape if it calls `from_vec`.

## Blast radius

`src/value/pvec.rs` (new). `Value` variant.
collection eval/transform. EDN. rete sites that
named `VectorSync`. No `.wat`. No Session field type.

## Out of scope = REJECTED

- Native `Vec` on the Session. Skip freeze.
- Intern `names`. A third PMap arm. 297.

## Sequencing

1. PVec. Cross-arm Eq/Hash. Value variant.
2. Freeze `from_vec`. Weigh out:query. Stop.

## Weigh (2026-08-22) — LANDED

`fanout_three_leftover_split` `[100 20]`, mean of 3.
instrument 102.2 ns/pair.

| lump | ms |
|---|---:|
| without-query wall / FIRE | 23.43 / 23.39 (was 26.13) |
| with-query wall / FIRE | **34.88 / 34.84** (was 40.35) |
| harvest:query | 9.51 |
| **out:query** | **0.00** (was 3.08) |
| **out:production** | **0.00** (was 4.12) |
| query-maps | 40,000 |

`out_query_cost_split`: H (`query_memory_to_pm`) **2.25** vs V RRB **5.48**
(H ≈ C clone 2.31 — freeze itself is the Arc wrap).
`out_production_cost_split`: H **2.54** vs V RRB **5.80**.

Prediction 3.08 → ~0.8 was shy; the mark fell through
the floor. Cross-arm Eq/Hash + vector-as-key green.
EDN round-trip still PersistentVector. Clippy `--lib
-D warnings` silent.
