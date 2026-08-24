# DESIGN-STONE — occupancy leaf-set column (not class-union)

> **Origin (2026-08-22).** Class-union fill reverted:
> 3-stratum Safe 2 vs 1. That intern filled
> `kind_ids.alpha ∩ class`. This stone fills only
> the **restricted tree's leaves** for a class that
> has **no equality children**. Same large write.
> Different vertex set. 3-stratum is the gate.

## The enemy

Seed still visits 80k `(aid, fact_idx)` to install
occupancy that is just fact ids. Group/Reading
trees have no discriminator: every packed fact of
the class sits in a known leaf list.

## The algorithm

```
undiscriminated class:
    tree root has no children, no wildcard
    leaves = candidate set for EVERY fact of that class
    ALL those leaves bind-only, no fact_bind

SEED pass 1:
    seen_insert
    if class is a leaf-set:
        pack row
        if packed: column[class].push(fact_idx)
        else: old activate
    else: old activate

SEED pass 2:
    n == 0 → skip (do not insert-wipe)
    els = column.map(|i| Element { i, empty })
    for aid in leaves:                 // tree leaves, not kind_ids
        alpha[aid] = els.clone()
        d_alpha[aid] = 0..n
```

`:not Warn` is class Warn's leaves, filled from
Warn facts. Not A's column. That was the coarseness.

## ★ THE ONE CONTRACT DECISION

**Fill the tree's candidate set for an
undiscriminated class, not every alpha of that
class in `kind_ids`.** Candidate set must match
`candidates_into` for any packed fact of the class.
3-stratum green or revert.

## The gate

1. 7strat including `differential_three_stratum_negation`.
2. rete lib. clippy `--lib -D warnings`.
3. `accum_fire_phase_census` `[200 200]`. Revert if
   FIRE regresses vs 17.8. Predicted drop is the
   80k visits; weigh. Do not wall-gate.

## Predicted win

Independent guess (written first): seed
**14.7 → 9–12**. FIRE **17.8 → 12–15**. Seen and
pack stay. If 7strat red, revert — leaf-set was
still too coarse. Do not intern (3) on a red (2).

## Weigh (2026-08-22) — interned, REVERTED

Same 3-stratum red as class-union: native
**[1, 2, 2]** vs oracle **[1, 2, 1]**. Filling
restricted **leaves** is not enough. The miss is
skipping per-fact `candidates_into` / activate
for a whole class, not which id list we used.
Reverted. Shared occupant vec (3) is the dual
pointer of this fill — do not intern it on a
red matcher. (3) after a correct per-match
install is a memory intern, not a FIRE intern.

## Recolligere (2026-08-22) — occupancy is not the miss

`n3_leaf_set_vs_occupancy` on the 7strat
compile-all (n3 rules + n/n3 queries). Four
fires (3 strata + query harvest). **extra=0
missing=0** every stratum. Predicted leaf-set
occupancy ≡ what activate installed. Safe k=[2]
on the activate path.

So fill ⊃ `candidates_into` is **false**. There
is no hidden per-fact prune on this class. The
3-stratum red was a **side effect of skip-
activate** (pack / `i64_by_fact` / `d_alpha`
install), not the vertex set. Attack that, not
a coarser or finer leaf list.

## Weigh (2026-08-22) — LANDED (pack-all + push, then Arc share)

Skip-activate with **pack every fact in visit
order** and **`entry` push not `insert` wipe**.
7strat 3-stratum **green**. Acc `[200 200]`:

| | FIRE | seed |
|---|---:|---:|
| before (skip BindSpan) | 17.8 | 14.7 |
| leaf fill + pack-all | **13.75** | **10.45** |
| + Arc shared occupancy | **13.69** | **10.49** |

Predicted 12–15. Met. Arc share is the dual
pointer of the fill (no extra FIRE). Occupancy
census gated (always-on HashSet was FIRE 28).

## Blast radius

`alpha_tree.rs` (undiscriminated leaves).
`fire/delta.rs` seed_round. No `.wat`. No crate.
No `unsafe`. Token BindSpan stays.

## Out of scope = REJECTED this stone

- Class-union / `kind_ids ∩ class` (reverted).
- Shared occupant `Vec<u32>` (stone 3, after this
  is 7strat-green).
- Pending+fill realloc cheat (weighed, not ≥ 1 ms).
- Rayon. SIMD. Session-`Vec`. 297.

## Sequencing

1. Leaf-set fill. 7strat. Weigh FIRE.
2. Green → shared occupant vec (3).
3. Red → revert. Do not do (3) on this intern.
