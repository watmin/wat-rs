# DESIGN-STONE — split `exec_ops` (scratch vs Bind)

> **Origin (2026-08-19).** After 13: intern_val **0.44**.
> Leftovers ≥ 1 ms: `setup:seen` ~3.9 (2z fire context —
> no intern without a new split), **ops 1.90**, scratch
> clone 1.02. This stone is ops. Do not intern `seen`.

## The measurement we do not have

`exec_compiled` does `scratch.clear(); scratch.resize(n,
None)` then `exec_ops` (Bind clones a field into a slot;
Cmp reads slots). O−T **1.90** is unsplit. 80,200 calls.

## The algorithm

Same fixture. Mean of 3. Stacked on candidates:

```
T   candidates
R   T + scratch clear/resize
O   T + scratch + exec_ops     // 8's O
```

Deltas: `R−T` scratch reset, `O−R` exec_ops body.

1. **STOP intern** if neither lump is ≥ 1 ms.
2. If `R−T` ≥ 1: keep capacity, `fill(None)` instead of
   `clear`+`resize` (same slots, no leak).
3. If `O−R` ≥ 1: say so; do not change Bind to a second
   scratch representation this stone.
4. Do not intern `seen`. Token stays two spans.

## ★ THE ONE CONTRACT DECISION

**Scratch still starts empty every call.** `fill(None)` is
the same as `resize(n, None)` after `clear` when capacity
already holds `n`. We do not skip clearing leftover slots.

## The gate

1. `accum_exec_ops_split` prints T/R/O and deltas. O > 0.
   Do not wall-gate FIRE.
2. If intern: O−T printed. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **`R−T` (scratch
reset) owns ≥ 1 ms** of 1.90. `fill(None)` may or may
not beat `clear`+`resize` — measure after intern. If
`O−R` is the row, leftover is Bind clone / Cmp; say so.

## Blast radius

`compiled_cond.rs` `exec_compiled` scratch reset only if
intern. One kernel test. No `.wat`. No `seen`.

## Out of scope = REJECTED

- Intern `seen`. Bind scratch as i64-only. Facts in
  `bind_pool`. Intern `names`. 2e / 2o. 297. Insertion.
- Per-fact timers. Tagged pool i64.

## Sequencing

1. Print T/R/O. Rank.
2. Neither ≥ 1 → stop.
3. Else intern scratch fill if R−T wins. Weigh. Stop.

## Weigh (2026-08-19) — LANDED, no intern

`accum_exec_ops_split`, 40,200 facts, mean of 3.

| lump | ms |
|---|---:|
| R−T scratch clear/resize | **1.71** |
| F−T scratch fill(None) | **1.78** |
| O−R exec_ops body | **0.12** |
| O−T ops lump | 1.83 |

Scratch owns ops. Bind/Cmp is dead. `fill(None)` is **not**
faster than `clear`+`resize` (1.78 vs 1.71). Intern tried
and reverted. A second scratch representation is STOP.
`seen` untouched.

Next leftovers: `setup:seen` ~3.9 (2z), scratch clone 1.02
(materialize, not this reset). Scratch reset 1.71 is the
ops lump and is not internable without a new representation.
