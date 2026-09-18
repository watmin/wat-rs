# DESIGN-STONE — fanout phase census at the grid ladder

> **Origin (2026-08-18).** Grid `T23-57-10Z`: 30/30 `:us`. Closest
> cell is **fanout `[40000]`** ratio **1.42** (141 ms / 200 ms).
> Accum leftovers do not rank this cell. HashJoin / token emit
> does. Measure before drawing a join strike.

## The measurement we do not have

`fanout_per_call_alpha_census` is one shot, no instrument subtract,
sized for compiled-conditions. The grid cell is 40k derived Pairs
(keys=100, fanout=20). We do not know which named phase owns the
141 ms.

## The algorithm

Reuse `render_phase_table`. Sizes are the grid ladder in seed
args: `(keys, fanout)` = `(25, 20)`, `(50, 20)`, `(100, 20)`.
Facts header: `keys * fanout * 2` (Left + Right).

Required floor: SETUP, ROUND LOOP, alpha, root-join, hash-join,
production. Discover the `hj:*` children. Do **not** require
accumulate/filter (this axis never reaches them).

## ★ THE ONE CONTRACT DECISION

**This stone prints the table. It does not change the engine.**
FIRE is not wall-gated. Non-vacuity: ROUND LOOP > 0 and
hash-join recorded at `[100 20]`.

## The gate

1. `fanout_fire_phase_census` exists. `--no-capture` prints the
   table at all three rungs.
2. rete lib (the new test green).
3. clippy `-D warnings`.

## Predicted win

A ranking: which `hj:*` / production / drop row owns the 141 ms.
Next strike is drawn from that table, not from accum leftovers.

## Blast radius

`src/rete/kernel/tests/` only. No fire-path change. No `.wat`.

## Out of scope = REJECTED

- Rewriting `right_idx` in the same diff.
- Persist. 297. HashSet insert.

## Sequencing

1. Helper + test on `render_phase_table`.
2. Run `--no-capture`. Rank. Stop.

## Weigh (2026-08-18) — LANDED

`fanout_fire_phase_census` `[100 20]` (40k Pairs) FIRE **96.66 ms**:

| phase | ms | share |
|---|---:|---:|
| **production** | **39.97** | **41.4%** |
| **`hj:catchup:probe`** | **30.87** | **31.9%** |
| `prod:compiled-rhs` net | 9.74 | 10.1% |
| alpha | 8.86 | 9.2% |
| `out:production` | 4.60 | 4.8% |
| `hj:catchup:right-idx` | 0.23 | 0.2% |

`right_idx` clone is not the row. Catch-up probe is `join_extend` × 40k,
and `join_extend` **always** `exec_compiled_under` even with no leftover
`SeedCmp`. Production is 40k RHS + `seen` insert of derived Pairs.

Next drawable: skip join rematch when `!has_seed_cmp` (fold-the-wall for
HashJoin). Not HashSet insert. Not persist.
