# DESIGN-STONE — fire-scoped i64 columns (kill E−K)

> **Origin (2026-08-22).** Packed-rows three scouts
> reverted. Isolated E−K **7.94 ms** is 80,200 scalar
> `exec_compiled_with_key_ids` on bind-only Group /
> Reading. Scout 2 deleted populate intern and **FIRE
> 19→70** (gather/fold re-paid `fact_at`). Scout 3
> stamped slots on Element, still paid `exec_ops`,
> FIRE 19→23. This stone is the bounded intern those
> scouts were aiming at: **pack i64 fields on first
> activate, invert bind-only populate, keep BindSpan.**

## The enemy

80,200 calls. Acc `[200 200]`: 200 Group + 40,000
Reading, two Reading alphas. Every cond is
`Op::Bind` only. `exec_ops` + scratch `Option<Value>`
+ `materialize_into` per (fact, alpha). `intern_val`
of the same i64 hits twice on Reading `:g`.

A faster Bind does not delete the call (scout 1,
−0.80). Deleting populate intern without a matching
gather representation inverts the cell (scout 2).
Slots on Element still pay `exec_ops` (scout 3).

## The algorithm

Fire-scoped, not a Session field. Not insert-time
SoA. Not SIMD. Not Cmp invert.

```
I64Row { n, fields[8] }              // Copy
FireSession.i64_by_fact[fact_idx]    // Option<I64Row>

SETUP:
    bind_only[alpha] = output field_idxs
        iff every Op is Bind and field < 8
    // no second PV walk — weighed, see below

populate (alpha_activate_fact):
    first touch of fact_idx:
        pack_i64_row(fact_fields)      // fields already in hand
    bind-only AND packed row
        → intern_val from row.fields
        → write BindSpan
        → no exec_ops, no scratch
    else exec_compiled_with_key_ids   // Cmp / mixed / unpacked

gather / fold:
    still read bind_pool BindSpan      // populate still wrote it
Token:
    still BindSpan at join birth
```

Pack is once per fact, from the fields slice
`candidates_into` already holds. Not a second
walk of the facts PV.

## ★ THE ONE CONTRACT DECISION

**Populate still writes BindSpan.** Gather/fold
already have interned slots. We do not skip the
pool. We do not put columns on Session. We do
not invert Cmp. Bind-only invert is class-match-
enough on this cell (the tree already over-
approximates; Bind always holds).

## The gate

1. `Element` still `{ fact, binds }`. Session still
   8 Values. `i64_by_fact` is FireSession only,
   cleared at fire start. Not packed at SETUP.
2. rete lib. `probe_arc278_7strat_native_differential`.
3. clippy `-D warnings` (`--lib`).
4. `accum_fire_phase_census` `[200 200]` prints FIRE
   and seed. **Revert if FIRE regresses** vs 19.3
   beyond noise. Do not wall-gate the number.

## Predicted win

Independent guess (written first): SETUP grows
**+1–2 ms** (40,200 intern_val). Seed drops by
most of E−K minus pool push — **16.6 → 9–12**.
FIRE **19.3 → 14–16**. Isolated E still calls
`exec_compiled` (old door); isolated A / in-fire
seed are the interned door. If FIRE does not fall
≥ 1 ms, leftover is the 80k call + pool push —
say so; do not reach for SIMD or Cmp invert.

## Weigh (2026-08-22) — LANDED, not the predicted cut

Two SETUP cuts inverted FIRE. Then pack-on-activate
landed a modest win. Acc `[200 200]`, release, mean
of 3.

| cut | FIRE | seed | SETUP pack |
|---|---:|---:|---:|
| baseline (packed-rows) | **19.3** | 16.6 | — |
| intern_val at SETUP walk | **25.0** | 14.2 | 6.50 |
| extract-only SETUP walk | **23.0** | 14.6 | 4.07 |
| **pack on first activate** | **17.8** | **15.1** | 0.02 |

Leftover split, interned A: isolated A **14.49**,
in-fire seed **15.06**, FIRE **17.82**. Isolated
E−K still **8.05** (old `exec_compiled` door).

Predicted 14–16 missed: skipping `exec_ops` is
~1.5 ms, not 7 of E−K. Pool push + intern_val
stay on populate. Extra PV walk costs 4–6 ms —
do not pack at SETUP. Do not skip BindSpan this
cut. Gather/fold still 1.3.

FIRE fell ≥ 1 ms. Keep. Next cut only if a named
leftover ≥ 1 ms remains (column gather/fold, skip
pool).

## Blast radius

`session.rs` (`I64Row`, `FireSession.i64_by_fact`,
pack). `compiled_cond.rs` (`bind_only_fields`,
`populate_bind_only`). `fire/delta.rs` (SETUP pack,
activate invert, derived pack). Census mark
`setup:i64-cols`. Leftover-split A packs so it
tracks activate. No `.wat`. No crate. No `unsafe`.
Token stays two spans. Freeze still Values.

## Out of scope = REJECTED this stone

- Insert-time SoA / `i64_facts_by_class` walk.
- Gather/fold reading `i64_by_fact` (next cut if
  this lands and index/fold remain).
- Skip BindSpan. Slots on Element (scout 3).
- i64 `exec_ops` twin (scout 1). Populate without
  materialize (scout 2).
- Invert Cmp / BindCheck / Or / Not.
- Session-`Vec`. Facts in `bind_pool`. Intern
  `names`. 297. SIMD. Scratch-as-new-repr.

## Sequencing

1. Write this stone. Predicted win first.
2. Pack + bind-only invert. BindSpan stays.
3. Weigh FIRE / seed / isolated A. Revert on
   FIRE regression. Stop. Next cut (column
   gather/fold, skip pool) only if this lands
   and a named leftover ≥ 1 ms remains.
