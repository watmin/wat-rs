# DESIGN-STONE — fire-path passes iterate kind lists on the arm

> **Origin (2026-08-20).** After 24: accum internable ≥ 1 is
> scratch STOP. A0 at equal work (10k derived both columns):
> ROUND LOOP **+7.04** at depth 50 vs 10. Five named children
> each +~1 ms: hash-join +1.95, filter / join-after-filter /
> root-join / production / accumulate +~1.05. A0 named this
> in 2026-07-31: four full-network scans × D rounds, one
> level can work. Dirty-node agenda is the class. This stone
> is the smaller intern: **do not `get_node` a node of the
> wrong kind.**

## The measurement we have

`a0_depth_cost_split_at_equal_work`: ROUND extra **+7.04**.
Accumulate on a cascade that has **zero** AccumulateNodes
still costs +1.04 — that is HAMT `get_node` of the whole
network, every round, to discover kind. Filter and
join-after-filter are the same shape if cascade has no
Test/Neg/Exists. Hash-join / root-join / production still
visit every node of *that* kind (dirty agenda, not this
stone).

`get_node` is a PersistentMap HAMT lookup. 5 passes × D
nodes × D rounds is O(D²) lookups. Kind lists make the
empty-kind passes a zero-length loop.

## The algorithm

At arm build, partition `node_ids` (already topo-sorted)
by `kind_of`. Store on `ReteArm`:

```
alpha          AlphaNode
join_parent    RootJoinNode ∪ HashJoinNode
acc            AccumulateNode
filter         TestNode ∪ NegationNode ∪ ExistsNode
prod           ProductionNode
filter_or_acc  merge(filter, acc)   // join-after-filter, topo
```

`fire_fixpoint_delta` passes iterate the matching list.
Census loops stay on `node_ids` (test-only). Oracle
`fire_once` / `production_pass` untouched. Token stays
two spans. No dirty set.

1. **STOP intern** if A0 ROUND extra does not fall ≥ 1 ms.
2. Do not skip idle *same-kind* nodes this stone.
3. Do not intern scratch. Do not intern `names`.

## ★ THE ONE CONTRACT DECISION

**Fire-path passes iterate kind-partitioned id lists
interned on the arm, not `node_ids` + `get_node` +
`kind_of`.** Topological order is preserved (each list
is a subsequence of `node_ids`). A node the list omits
is never `get_node`'d by that pass.

## The gate

1. `cascade_kind_list_split` prints list sizes at cascade
   depth 50. Lists are disjoint, each id ∈ `node_ids`.
   `a0_depth_cost_split_at_equal_work` prints ROUND extra.
   Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): empty-kind passes
(accum / filter / join-after-filter) each → **~0**.
ROUND extra **7.04 → ~4**. Cascade `[50 100]` FIRE
**18.06 → ~15**. Hash-join leftover remains for dirty
agenda. Accum `[200 200]` FIRE wash (few nodes).

## Blast radius

`kernel.rs` `ReteArm` + `build_rete_arm` / slice / export
constructors + six `fire_fixpoint_delta` loops. No `.wat`.
Token stays two spans.

## Out of scope = REJECTED

- Dirty-node agenda. Per-node timers. Intern `names`.
- 2e / 2o. 297. Insertion. Session-`Vec`. Scratch.
- `production_pass` / `fire_once`. Census loops.

## Sequencing

1. Lists on the arm. Six passes. Weigh A0 extra. Stop.
2. If extra still ≥ 1 on hash-join / root-join /
   production: name dirty agenda. Do not intern it here.

## Weigh (2026-08-20) — LANDED

`cascade_kind_list_split` depth 50: node_ids **250**,
alpha 100, join_parent 100, acc **0**, filter **0**,
prod 50. Disjoint, topo. Gate: rete lib 99, clippy
`-D warnings` silent.

`a0_depth_cost_split_at_equal_work`:

| lump | before | after |
|---|---:|---:|
| ROUND extra | **+7.04** | **+2.06** |
| accumulate extra | +1.04 | **+0.00** |
| filter extra | +1.05 | **+0.00** |
| join-after-filter extra | +1.05 | **+0.00** |
| hash-join extra | +1.95 | **+1.43** |
| root-join extra | +1.05 | **+0.46** |
| production extra | +1.07 | **+0.22** |

Predicted extra → ~4; measured **2.06** (beat).

`honest_cell_rank_after_arm`:

| cell | FIRE before | FIRE after | honest before | honest after |
|---|---:|---:|---:|---:|
| deep-cascade `[50 100]` | 18.06 | **11.76** | 13.25 | **7.32** |
| accum `[200 200]` | 19.25 | 18.71 | 18.91 | 18.50 |
| fanout `[100 20]` | 26.78 | 27.66 | 13.45 | 11.22 |

Cascade FIRE **−6.3**. Accum wash (few nodes). Hash-join
extra **1.43** remains — dirty same-kind, not this stone.
Scratch STOP. Do not intern names. Do not start 297.
