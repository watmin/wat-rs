# DESIGN-STONE — populate `range_children`

> **Origin (2026-08-19).** 4 skipped `exec_where` on proven
> `(= dim lit)`. `range_children` is still `Vec<()>` —
> `< > <= >=` ride the wildcard. Node-share is equality.
> The named cell is `where-numeric` / chaos-engine, not
> Clara's ranking grid.

## The measurement

`(b)` analyzes only `=`. A `where` of `(> dim lit)` is
wildcard → maybe → `exec_where` every token. The lab's
`ShadowNode` walks `(op, threshold)` guard edges without
exploding into equality children. Ours reserved the slot
and never filled it.

Alpha-tree range edges are a **different** stone
(`DESIGN-STONE-alpha-tree-range-edges`). This stone is
the **where**-tree.

## The algorithm

```
analyze: (= dim lit) | (< > <= >= dim lit)
  lit on the left → flip the op
  two constraints on one dim → wildcard (over-approx)

build: equality children as today
       range_children: (CmpKind, threshold) → subtree

walk Ok(v):
  equality child if any
  each range edge: holds? walk proven : skip
  incomparable → walk maybe
walk Err: every equality + range child, maybe
wildcard: maybe
```

`pure_cmp` = And of dim-lit eq **or** range. Proven ∩
`pure_cmp` skips `exec_where` (same as 4). Raises stay
on the maybe path.

## ★ THE ONE CONTRACT DECISION

**Range edges are guards, not equality fan-out.** The tree
still over-approximates. Two constraints on one dim ride
the wildcard. `not=` rides the wildcard. Alpha-tree
`range_children` stays empty.

## The gate

1. `where_tree` unit: `(> ?k 10)` pruned at 5, proven at 15.
2. rete lib. Node-share census still reuse=200 / evals=0.
3. clippy `-D warnings` (`--lib`).
4. FIRE printed on node-share / accum — **not** wall-gated.

## Predicted win

Node-share wash (no ranges). Accum wash. The mechanism is
the gate; `where-numeric` is the named later cell, not a
FIRE cut on `[200 200]`.

## Blast radius

`where_tree.rs`. `matcher.rs` `CmpKind: Hash` if needed.
`kernel.rs` `is_pure_cmp` name. No `.wat`. No crate.
No `unsafe`. Token stays two spans.

## Out of scope = REJECTED

- Alpha-tree range edges. `not=` as a range. Session-stored.
- Two constraints on one dim (wildcard).
- Intern `names`. Facts in `bind_pool`. 2e / 2o. 297.
- Fact insertion. Persist gather to dodge the fold.

## Sequencing

1. Analyze ranges. Populate `range_children`. Walk them.
2. `pure_cmp` includes ranges. Skip `exec_where` when proven.
3. Unit + rete lib. Weigh. Stop.
