# BRIEF — do not `exec_where` a proven `(= dim lit)`

## The work

The where-tree walk already executed the dim. Proven
equality residues skip `exec_where`. Maybe / wildcard /
impure still eval. Token stays two spans.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 4 is this skip.
2. `where_tree.rs` contract (over-approx only).
3. `kernel.rs` `dispatch_where_tests`.
4. `DESIGN-STONE-where-dim-reuse.md`.

## Sketch

```
candidates → { proven, maybe }
if proven && pure_eq { pass; reuse++ } else { exec_where }
```

## STOP

1. **STOP-1** — skip maybe / wildcard / impure `where`.
2. **STOP-2** — count skip as `filter:test-evals`.
3. **STOP-3** — intern `names` / facts in `bind_pool` / 2e / 2o.
4. **STOP-4** — FIRE wall. Range edges (5). 297.

## Done

- `node_share_filter_eval_census` prints reuse; passes ≈ M.
- rete lib green. clippy `-D warnings` silent.

Leave dirty.
