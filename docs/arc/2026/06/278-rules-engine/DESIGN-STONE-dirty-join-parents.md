# DESIGN-STONE — hash-join iterates dirty join-parents only

> **Origin (2026-08-20).** After 25: empty-kind scans are
> dead. A0 ROUND extra **+2.06**. Hash-join extra **+1.43**
> — every join-parent every round (`get_node` of 100 at
> depth 50 × 50 rounds). One level can work. A0 named the
> class: dirty-node agenda. This stone is that intern, on
> the **hash-join pass only**. Root-join extra 0.46 under.
> Production extra 0.22 under. Do not intern those.

## The measurement we have

`a0_depth_cost_split_at_equal_work` after 25:

| lump | extra |
|---|---:|
| ROUND | +2.06 |
| hash-join | **+1.43** |
| root-join | +0.46 |
| production | +0.22 |

Kind lists still visit every `join_parent`. Idle
`get_node` + `kind_of` + empty `d_beta`/`d_alpha`
continue. First-keying catch-up uses cumulative
`wm.beta`/`wm.alpha`; it fires the round the **second**
side arrives, and that side's delta is non-empty.

## The algorithm

Invert `feeding_alpha_of` at arm build:

```
joins_fed_by: alpha_id → [HashJoin id]
```

Each round, dirty join-parents are the union of:

1. `d_beta` keys that are in `join_parent` (left dirty)
2. parents-of (in `join_parent`) of HashJoins fed by a
   non-empty `d_alpha` alpha (right dirty)

Seed a dirty *set* from (1)+(2). Hash-join still walks
`join_parent` in topo order but **skips `get_node`** when
the id is not dirty. When a HashJoin emits tokens, the
child id is inserted (middle join: J1's tokens dirty J1
as parent of J2 this same round). A snapshot-then-iterate
list would miss that. Join-after-filter (Test→HashJoin)
untouched. Token stays two spans.

First-keying: the round the second side arrives, (1) or
(2) contains the parent. After `join_keys` is cached,
empty-delta rounds skip. No third "unkeyed but both
sides full" visit — that state cannot arise if the
second-side round was visited.

1. **STOP intern** if A0 hash-join extra does not fall
   ≥ 0.5 ms (named leftover is 1.43; a wash is a miss).
2. Do not dirty-agenda root-join / production / filter.
3. Do not intern scratch. Do not intern `names`.

## ★ THE ONE CONTRACT DECISION

**The hash-join pass iterates dirty join-parents, not
every `join_parent`.** A parent with empty left-delta
and no HashJoin child whose feeding alpha has right-delta
is not `get_node`'d this round. First-keying still runs
the round the second side arrives.

## The gate

1. `a0_depth_cost_split_at_equal_work` prints hash-join
   extra. Do not wall-gate FIRE.
2. rete lib (hash-join unit tests + cascade).
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): hash-join extra
**1.43 → ~0.2**. ROUND extra **2.06 → ~0.8**. Cascade
`[50 100]` FIRE **11.76 → ~10.5**. Accum wash (one join).

## Blast radius

`kernel.rs` `ReteArm.joins_fed_by` + three constructors
+ dirty-set + one loop. No `.wat`. Token stays two spans.

## Out of scope = REJECTED

- Root-join / production / filter dirty sets.
- Per-node timers. Intern `names`. Scratch. 2e / 2o.
- 297. Insertion. `fire_once`. Join-after-filter rewrite.

## Sequencing

1. Invert. Dirty set. Hash-join loop. Weigh extra. Stop.

## Weigh (2026-08-20) — LANDED

Gate: rete lib 99, clippy `-D warnings` silent.
`beta_write_read_traffic` tri middle join still reads
(grow-on-emit held). Hash-join unit tests green.

`a0_depth_cost_split_at_equal_work`:

| lump | before | after |
|---|---:|---:|
| ROUND extra | +2.06 | **+0.95** |
| hash-join extra | **+1.43** | **+0.08** |
| root-join extra | +0.46 | +0.45 |
| production extra | +0.22 | +0.39 |

Predicted hash-join extra → ~0.2; measured **0.08**.

`honest_cell_rank_after_arm`:

| cell | FIRE before | FIRE after | honest before | honest after |
|---|---:|---:|---:|---:|
| deep-cascade `[50 100]` | 11.76 | **10.35** | 7.32 | **6.72** |
| accum `[200 200]` | 18.71 | 19.64 | 18.50 | 19.30 |
| fanout `[100 20]` | 27.66 | 26.75 | 11.22 | 13.15 |

Cascade FIRE **−1.4**. Accum wash. Root-join extra 0.45
under — do not dirty-agenda it. Scratch STOP. Do not
intern names. Do not start 297.
