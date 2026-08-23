# DESIGN-STONE — catch-up takes parent beta, does not clone Vec

> **Origin (2026-08-23).** Catch-up occupancy Arc LANDED
> (`bdf11d21`). Right side bumps `alpha[A]`. Left side still
> `wm.beta.get(node_id).cloned()` — a second Vec of the
> same tokens. HashMap split-borrow needs the parent out
> of the map while `entry(child)` mutates. It does not
> need a copy of the bag.

## The enemy

```
all_left: Vec<Token> = beta[P].cloned()
for tok in all_left: probe, index
beta.entry(J).push(new)   // different key, same map
```

Token is Copy. The walk is WHAT. The clone is occupancy
theater of the left: reconstructing parent tokens the
map already holds. Occupancy Arc intern could bump;
beta is still `HashMap<i64, Vec<Token>>`. Do not
Arc-wrap all beta. Take the Vec, walk, put it back.

## The algorithm

```
(all_left, restore) = match beta.remove(P)
    Some(v) => (v, true)
    None    => (empty, false)
walk &all_left for probe and left_idx
if restore: beta.insert(P, all_left)
then entry(J) emit
```

Same tokens, same order. Dual-impl WHAT unchanged.
Do not skip the walk. Do not Session-Vec. Do not
Arc-wrap BetaMemory.

## ★ THE ONE CONTRACT DECISION

**Catch-up holds parent beta by move, not by memcpy.**
Empty vs missing is restored exactly. Child emit
runs after the parent is back.

## The gate

1. Leftover Instant: fanout without-query FIRE does
   not regress. Any drop counts.
2. 7strat 3/3 including three-stratum.
3. rete lib. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): fanout catch-up
drops the left-token memcpy. Probe stays. Wrap W
stays physics. Sibling of occupancy Arc intern.

## Blast radius

`fire/delta.rs` first-keying `all_left`. No `.wat`.
No Session field. No `BetaMemory` type change.

## Out of scope = REJECTED

- Session-Vec. Skip freeze. intern `names`. Array1. 297.
- Arc-wrap `wm.beta`. Occupancy as `Vec<u32>`.

## Sequencing

1. Take / restore. Weigh. Stop.
2. Revert if 7strat red or leftover Instant regresses
   without a named drop.

## Weigh (2026-08-23) — LANDED

7strat 3/3 including three-stratum. Clippy `--lib -D warnings` silent.

Same-session leftover (mean of 3), before intern then after:

| | before | after |
|---|---:|---:|
| fanout without-query FIRE | 24.85 | **24.68** |
| fanout with-query FIRE | 29.10 | 29.42 |
| honest FIRE (census) | 26.06 | **25.58** |

Without-query FIRE **−0.17**. Honest FIRE **−0.48**. Parent beta moved, not memcpy. Child emit after restore. Dual-impl WHAT unchanged. Do not Arc-wrap BetaMemory. Do not Session-Vec.
