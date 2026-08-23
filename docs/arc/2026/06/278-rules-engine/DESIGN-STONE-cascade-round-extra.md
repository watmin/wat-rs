# DESIGN-STONE — rank what remains of the cascade round loop

> **Origin (2026-08-23).** `cascade_query_harvest_split` `[50 100]`:
> SETUP **0.06**, harvest wrap **1.97**, ROUND **8.63** without
> queries. Kind lists and dirty join-parents already interned.
> This stone is the walk to the round-loop leftover: print A0 at
> equal work on **this** tip. Do not intern off 2026-08-20 extras.

## The enemy

A0 (2026-07-31): four full-network scans × D rounds, one level
can work. Kind lists killed empty-kind passes (ROUND extra
**+7.04 → +2.06**). Dirty join-parents killed idle hash-join
`get_node` (hash-join extra **+1.43 → +0.08**, ROUND extra
**+0.95**). What remains of ROUND extra at this tip is unnamed.

```
a0_depth_cost_split_at_equal_work
  10k derived both columns
  depth 10 × width 500   vs   depth 50 × width 100
  ROUND extra, and extras of hash-join / root-join / production
```

No fire-path change. Dual-impl WHAT unchanged.

## ★ THE ONE CONTRACT DECISION

**This stone prints the extras. It does not change the engine.**
A row is internable if honest extra ≥ 0.5 ms and is not
wrap / names / stamp / Session-Vec / already-rejected dirty
root-join.

## The gate

1. `a0_depth_cost_split_at_equal_work` prints. ROUND extra
   named. Do not wall-gate FIRE.
2. rete lib.
3. clippy `--lib -D warnings` (print only — already silent).

## Predicted win

Independent guess (written first): ROUND extra is **~1 ms**.
Hash-join extra stays ~0.1 (dirty-join held). Production or
root-join owns the rest. If no extra ≥ 0.5 ms, the 8.63 ROUND
is real work at equal derived count — stop.

## Blast radius

None unless a row earns intern. Print already exists.

## Out of scope = REJECTED

- Session-Vec. Skip freeze. intern `names`. 297.
- Dirty-agenda root-join (dirty-join-parents STOP).
- Dirty-agenda filter. Per-node timers.
- Harvest wrap of 10,200 maps.

## Sequencing

1. Print A0. Rank extras. Stop.
2. Next intern is the largest internable extra, a later stone.

## Weigh (2026-08-23) — LANDED print; no intern

`a0_depth_cost_split_at_equal_work` at this tip (10k derived
both columns):

| lump | depth 10×500 | depth 50×100 | extra |
|---|---:|---:|---:|
| ROUND LOOP | 8.78 | 9.65 | **+0.88** |
| production | 4.17 | 4.73 | **+0.57** |
| hash-join | 1.65 | 1.77 | +0.11 |
| root-join | 0.34 | 0.47 | +0.13 |
| SETUP | 0.02 | 0.08 | +0.07 |
| alpha | 2.46 | 2.45 | −0.01 |
| compiled-rhs | 1.71 | 1.73 | +0.02 |

Kind lists + dirty join-parents interned the idle scan.
ROUND extra **+0.88**. Production extra **+0.57** is the
largest remaining depth tax. Hash-join extra +0.11 held.
Root-join extra +0.13 stays under dirty-join-parents STOP.
Absolute ROUND ~8.6 is real work (same 10k derived):
production + hash-join + alpha. Do not intern wrap. Do not
dirty-agenda root-join in this stone. Production extra is
the next intern if we keep going on this cell.
