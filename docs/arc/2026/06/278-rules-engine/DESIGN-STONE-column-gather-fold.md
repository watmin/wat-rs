# DESIGN-STONE — column gather/fold, then skip BindSpan

> **Origin (2026-08-22).** Fire-i64-columns LANDED: bind-only
> skips `exec_ops`, pack on first activate. FIRE **17.8**,
> seed **15.1**. Isolated E−K still 8.05 (old door).
> Skipping `exec_ops` was **1.5 ms**, not 7 of E−K.
> Pool push + `intern_val` stay on populate. This cut
> points gather/fold at the packed row, then **skips
> BindSpan** on bind-only populate. Scout 2 is the
> warning: skip the pool only after gather/fold already
> read the columns.

## The enemy

Seed is still **15.1 ms (85% of FIRE)**. Bind-only
populate still `intern_val` + `pool.push` per
(fact, alpha) — 80,200 spans. Gather unary walks
those spans; fold `slot_i64` walks them. Acc index
+ fold is 1.3 ms. The intern that kills seed is
skipping the span, not a faster Bind.

## The algorithm

```
I64Row { n, fields[8], vids[8] }

first activate:
    intern_val each i64 field once → vids
    // fields already in hand; NOT a SETUP walk

bind-only populate:
    Element { fact, binds: empty }   // no pool.push

gather unary / append:
    vid = i64_by_fact[el.fact].vids[field_of(?g)]
    // Cmp / unpacked: still pool_slice

fold Sum/Min/Max/Mean:
    i64 = i64_by_fact[el.fact].fields[field_of(?v)]

gather_join_keys:
    empty Element binds → α slot_keys ∩ token keys
    // not cartesian (scout 2)

root-join seed:
    empty Element binds → write Token span from row vids
    // Group ?g; 200 spans, not 80k

hash-join right / join_extend:
    empty Element binds → key / append from row vids

leading exists distinct:
    empty binds → pairs from row vids
```

Cmp / BindCheck / unpacked rows keep BindSpan and
the old pool readers.

## ★ THE ONE CONTRACT DECISION

**Gather/fold read `i64_by_fact` before populate
skips BindSpan.** Token still has BindSpan (written
at join birth from the row). Session stays 8 Values.
We do not skip Token binds. We do not invert Cmp.

## The gate

1. Bind-only packed Elements have `binds.len == 0`.
   Token binds nonempty on Group root-join.
2. rete lib. `probe_arc278_7strat_native_differential`.
3. clippy `-D warnings` (`--lib`).
4. `accum_fire_phase_census` `[200 200]`. **Revert if
   FIRE regresses vs 17.8** (scout 2 was 70). Do not
   wall-gate the number.

## Predicted win

Independent guess (written first): skip 80k
`pool.push`. Seed **15.1 → 8–11**. FIRE **17.8 →
11–14**. Index/fold stay ~1.3 or dip slightly.
If FIRE rises toward 70, gather missed a BindSpan
reader — revert. If FIRE does not fall ≥ 1 ms,
leftover is intern_val of vids at first activate
— say so; do not skip Token spans.

## Weigh (2026-08-22) — interned, predicted cut missed

Acc `[200 200]`, release, mean of 3. 7strat green.

| | FIRE | seed | accum index | fold |
|---|---:|---:|---:|---:|
| fire-i64-columns | **17.8** | 15.1 | 0.65 | 0.67 |
| **this cut** | **17.8** | 14.7 | 0.85 | 0.80 |

Not scout 2 (FIRE 70). Skip BindSpan is not
the 15 ms of seed. Pool push was not E−K.
Index/fold +0.3 (column vid vs `pool_slice`).
Leftover of seed is the 80k activate walk
(seen, tree, intern_val of vids, Element
push). Keep — FIRE did not regress. Do not
skip Token spans. Do not intern a faster
Bind.

## Blast radius

`session.rs` (`I64Row.vids`). `compiled_cond.rs`
(bind-only empty span). `fire/delta.rs` (activate,
root-join seed, hash-join right, exists).
`fire/mod.rs` (gather, join_extend, GatherIntern).
`fire/acc.rs` (fold). No `.wat`. No crate. No
`unsafe`. Freeze still Values.

## Out of scope = REJECTED this stone

- SETUP PV walk (weighed: FIRE 19→25).
- Insert-time SoA. SIMD. Invert Cmp.
- Skip Token BindSpan. Slots on Element (scout 3).
- i64 `exec_ops` twin. Populate-without-materialize
  without column readers (scout 2).
- Session-`Vec`. Facts in `bind_pool`. 297.

## Sequencing

1. Write this stone. Predicted win first.
2. Column readers (gather/fold/join/exists).
3. Bind-only populate empty span. Token seed from row.
4. Weigh FIRE / seed / index / fold. Revert on
   FIRE regression. Stop.
