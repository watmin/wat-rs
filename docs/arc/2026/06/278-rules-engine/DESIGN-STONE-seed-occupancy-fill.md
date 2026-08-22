# DESIGN-STONE — seed occupancy: reserve(n) + straight fill

> **Origin (2026-08-22).** Occupancy is still
> `FxHashMap<aid, Vec<Element>>` grown 80k times.
> Same matcher. Cheaper install: collect fact ids,
> `reserve(n)`, sequential fill. Not class-union
> (7strat red). Not leaf-set. Not shared `Vec<u32>`.

## The enemy

Seed installs occupancy with
`alpha.entry(aid).or_default().push(Element)` per
match. Vec doubling + HashMap on every push.
N−E was **~1.3 ms**. Candidate set unchanged.

## The algorithm

Seed only (delta still pushes live — small n).

```
pending: aid → Vec<fact_idx>     // skip-span matches
  reserve n_input per known alpha

activate skip-span:
    pending[aid].push(fact_idx)  // no Element, no d_alpha
activate Cmp / unpacked:
    old push                     // BindSpan still needed

after the PV walk:
    for (aid, ids) in pending:
        alpha[aid].reserve(n)
        push Element { idx, empty } in ids order
        d_alpha[aid] = start..start+n
```

Same occupants, same order on a packed bind-only
alpha (visit order). Mixed exec+skip-span on one
alpha would reorder (unpacked first). Acc and
7strat are packed i64. 7strat is the gate.

## ★ THE ONE CONTRACT DECISION

**The candidate set does not change.** We only
change how seed writes `alpha` / `d_alpha`.
Token still BindSpan. Session stays 8 Values.

## The gate

1. Skip-span seed matches are not pushed until
   the fill. Cmp path unchanged.
2. rete lib. `probe_arc278_7strat_native_differential`.
3. clippy `-D warnings` (`--lib`).
4. `accum_fire_phase_census` `[200 200]`. **Revert
   if FIRE regresses vs 17.8, or does not fall
   ≥ 1 ms.** Do not wall-gate the number.

## Predicted win

Independent guess (written first): N−E grow
dies. FIRE **17.8 → 16–17**. Seed **14.7 →
13–14**. Seen / tree / pack stay. If FIRE does
not fall ≥ 1 ms, leftover is not realloc —
say so; do not jump to leaf-set this stone.

## Weigh (2026-08-22) — interned, REVERTED

7strat green. Acc `[200 200]` FIRE **17.48**
(was 17.8), seed **14.19** (was 14.7). ROUND
LOOP range [16.0–18.1] swallows the cut. Not
≥ 1 ms. Realloc was not the leftover. Reverted.
Same matcher; cheaper install did not show.

## Blast radius

`fire/delta.rs` seed_round + `alpha_activate_fact`
pending. Tests pass `pending: None`. No `.wat`.
No crate. No `unsafe`. Freeze still Values.

## Out of scope = REJECTED this stone

- Class-union fill (reverted). Leaf-set column.
  Shared occupant `Vec<u32>`.
- Intra-fire rayon. SETUP intern_val walk.
- Skip Token BindSpan. SIMD. Session-`Vec`. 297.

## Sequencing

1. Write this stone. Predicted win first.
2. Pending skip-span. Fill after seed walk.
3. 7strat. Weigh FIRE. Revert on miss. Stop.
