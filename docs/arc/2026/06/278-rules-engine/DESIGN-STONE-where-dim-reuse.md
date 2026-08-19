# DESIGN-STONE — do not `exec_where` a proven `(= dim lit)`

> **Origin (2026-08-19).** 3 persist-gather did not move accum.
> NEXT **4**: `(b)` already routes tokens; we still `exec_where`
> the residue. Node-share `[50 200]` FIRE **1.71**, filter
> **0.50**. The dim was already executed in the walk.

## The measurement

`dispatch_where_tests` asks the tree for candidates, then
`exec_where` on every hit. Node-share `where` is one
`(= lit dim)` per rule. The walk already ran `exec_dim` and
took the matching child. The second eval rebuilds a frame
and runs the same Call. `(b)` cut M×N → M. This stone
cuts the remaining M.

`(b)` contract: the tree may over-approx, never under-approx.
`exec_where` stays authority on **maybe** candidates and on
raises. A dim that raised in the walk walked every child —
those ids are **not** proven. Wildcard ids are **not**
proven. A `where` that is not only `And` of `(= dim lit)`
is **not** proven.

## The algorithm

```
walk: proven vs maybe
  Ok(v) + equality child → descendants stay proven
  Err(_)                 → every child, maybe
  wildcard               → maybe

skip exec_where iff
  id ∈ proven  AND  where is pure (= dim lit) conj
else exec_where (authority)
```

Census: `filter:test-reuse` on skip; still `filter:test-pass`.
Do not count skip as `filter:test-evals`.

## ★ THE ONE CONTRACT DECISION

**A proven equality residue does not re-enter `exec_where`.**
Maybe / wildcard / impure `where` still do. Raises stay on
the maybe path. The tree still over-approximates; skip is
only the under-approx-safe subset.

## The gate

1. `node_share_filter_eval_census`: passes ≈ M; reuse or
   evals non-zero; evals not M×N.
2. rete lib.
3. clippy `-D warnings` (`--lib`).
4. `node_share_fire_phase_census` prints FIRE. **Not**
   wall-gated. Accum `[200 200]` unchanged in kind
   (no TestNode residue there).

## Predicted win

Node-share `[50 200]` filter 0.50 → **~0**. FIRE 1.71 →
**~1.2–1.5**. Accum FIRE wash. If filter does not fall,
the residue was not pure-eq — say so.

## Blast radius

`where_tree.rs` (proven walk, `pure_eq`). `kernel.rs`
`dispatch_where_tests` + node-share census. No `.wat`.
No crate. No `unsafe`. Token stays two spans.

## Out of scope = REJECTED

- Skip maybe / wildcard. Skip impure `where`.
- Range edges (that's **5**). Session-stored tree.
- Intern `names`. Facts in `bind_pool`. 2e / 2o.
- Persist gather to dodge the fold. 297. Fact insertion.

## Sequencing

1. Proven vs maybe in the walk. Pure-eq set at build.
2. Skip `exec_where` on proven ∩ pure-eq.
3. Weigh node-share census + FIRE. Stop.

## Weigh (2026-08-19) — LANDED, small

`node_share_filter_eval_census`:

| N | evals | reuse | passes |
|---:|---:|---:|---:|
| 10 | 0 | 200 | 200 |
| 25 | 0 | 200 | 200 |
| 50 | 0 | 200 | 200 |

Every node-share residue skipped `exec_where`. `cell_rank`
`[50 200]` FIRE **1.62**, filter **0.43** (was 1.71 / 0.50).
The leftover filter is the tree walk (`exec_dim`), not a
second `exec_where`. Accum `[200 200]` **48.09**. Do not
range-edge here — that's **5**.
