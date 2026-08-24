# DESIGN-STONE — production iterates dirty parents only

> **Origin (2026-08-23).** `cascade-round-extra` LANDED print.
> A0 ROUND extra **+0.88**. Production extra **+0.57**. Hash-join
> extra +0.11 (dirty-join held). Root-join extra +0.13 stays
> under dirty-join-parents STOP. Kind lists already visit only
> ProductionNodes. Idle ones still `get_node` + name + cache
> lookup every round. 50 prods × 50 rounds.

## The enemy

```
for node_id in kind_ids.prod:     // 50 ids × 50 rounds
    get_node + kind_of + rule-name + compiled_rhs_cache.get
    for pid in parents:
        d_beta.get(pid) or continue
```

Empty `d_beta` already skips the token loop. The tax is
reaching that check. Dirty join-parents left production out
when extra was 0.22–0.39. This tier interned 0.5 ms honest
Instant that removes theater.

## The algorithm

Skip `get_node` unless some parent has non-empty `d_beta`
this round:

```
for node_id in kind_ids.prod:
    pids = parents_of[node_id]
    if no pid has non-empty d_beta: continue
    // existing body
```

Same visit set as today's inner `d_beta.get` continue.
`:or` still walks every parent that has tokens. Token stays
two spans. Do not dirty-agenda root-join.

1. **STOP intern** if A0 production extra does not fall
   ≥ 0.3 ms (named leftover is 0.57; a wash is a miss).
2. 7strat 3/3 including three-stratum.
3. Do not intern scratch. Do not intern `names`.

## ★ THE ONE CONTRACT DECISION

**The production pass iterates a ProductionNode only when a
parent is dirty this round.** A node with empty parent
deltas is not `get_node`'d.

## The gate

1. `a0_depth_cost_split_at_equal_work` prints production extra.
   Do not wall-gate FIRE.
2. 7strat 3/3 including three-stratum.
3. rete lib. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): production extra
**+0.57 → ~0.1**. ROUND extra **+0.88 → ~0.4**. Cascade
`[50 100]` without-query ROUND ~8.6 holds (real work).

## Blast radius

`fire/delta.rs` production loop. No `.wat`. No Session field.

## Out of scope = REJECTED

- Dirty-agenda root-join / filter. Session-Vec. Skip freeze.
  intern `names`. 297.

## Sequencing

1. Skip idle prods. Weigh A0 extra. Stop.
2. If extra does not fall, revert.

## Weigh (2026-08-23) — LANDED

`a0_depth_cost_split_at_equal_work` (10k derived both columns):

| lump | extra before | extra after |
|---|---:|---:|
| ROUND LOOP | +0.88 | **−0.20** |
| production | **+0.57** | **+0.15** |
| hash-join | +0.11 | −0.02 |
| root-join | +0.13 | +0.08 |

Production extra **−0.42 ms** (gate ≥ 0.3). ROUND extra gone.
7strat 3/3 including three-stratum. Clippy `--lib -D warnings` silent.
Did not dirty-agenda root-join. Checkpoint `8cdf40dd` if revert.
