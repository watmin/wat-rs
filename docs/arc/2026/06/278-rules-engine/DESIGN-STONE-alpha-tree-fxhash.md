# DESIGN-STONE — alpha-tree children are FxHashMap

> **Origin (2026-08-20).** After 23: honest intern cell still
> accum. Isolated I−G tree walk **1.03** (stone 10: 0.08).
> `Node.children` is `std::collections::HashMap<Value, _>` —
> SipHash of the field `Value` 40,200 times. `wm.alpha` already
> interned FxHash (12). Scratch 1.75 STOP. Do not intern names.

## The measurement we have

`accum_alpha_tree_walk_split` today: I−G **1.03**, T−I 1.96
(isolated `candidates()` alloc; fire uses `candidates_into`).
`walk` is `children.get(field)`.

## The algorithm

`Node.children: FxHashMap<Value, Arc<Node>>`. Build still
buckets in a std map, then collects. Over-approx unchanged.
Token stays two spans. Do not populate `range_children`.

## ★ THE ONE CONTRACT DECISION

**Fire-path field equality fan-out hashes with FxHash, not
SipHash.** The tree still over-approximates.

## The gate

1. `accum_alpha_tree_walk_split` prints I−G. FIRE printed
   at leftover split. Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): I−G 1.03 → **~0.3–0.5**.
FIRE 19.04 → **~18.5**. Scratch untouched.

## Blast radius

`alpha_tree.rs` `Node.children` only. No `.wat`. No kernel
fire-path besides the walk.

## Out of scope = REJECTED

- Range edges. Intern `names`. Scratch repr. 2e / 2o. 297.
- Insertion. Session-`Vec`. i64-only children map.

## Sequencing

1. FxHashMap. Weigh I−G / FIRE. Stop.

## Weigh (2026-08-20) — LANDED, wash

`accum_alpha_tree_walk_split`: I−G **0.98** (was 1.03).
Predicted → 0.4 **missed**. Walk leftover is not SipHash.

`accum_leftover_split` `[200 200]`:

| lump | before | after |
|---|---:|---:|
| FIRE | 19.04 | **18.75** |
| honest_FIRE | 18.61 | **18.32** |
| honest_alpha | 16.24 | **15.92** |

Gate: rete lib 98, clippy `-D warnings` silent. Kept:
same hasher family as `wm.alpha` (12). FIRE did not rise.
Scratch STOP. Do not intern names. Do not start 297.

Next leftover: cascade ROUND extra vs shallow **+7.04**
(A0 equal-work). Accum internable ≥ 1 is scratch STOP.
