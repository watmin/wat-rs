# DESIGN-STONE — catch-up bumps occupancy Arc, does not memcpy Vec

> **Origin (2026-08-23).** Packed seed `d_alpha` range LANDED
> (`b166cd10`). Occupancy is `Arc<Vec<Element>>`, shared
> across alphas. Hash-join catch-up then
> `v.as_ref().clone()` — a second Vec of the same
> occupants. Split-borrow needs a local handle. It does
> not need a copy of the bag.

## The enemy

```
all_right: Vec<Element> = alpha[A].as_ref().clone()
for el in all_right: key, span, index
```

Seed already shared that Arc. Catch-up memcpy is
occupancy theater: asking occupancy who sits in the
leaf by copying the answer. The walk is WHAT.
`element_with_row_span` mutates `bind_pool`, not
alpha. Catch-up `continue`s before `make_mut`.

`all_left` clones `wm.beta` (HashMap split-borrow).
Not this intern. Beta is not Arc occupancy.

## The algorithm

```
all_right = alpha[A].cloned()          // Arc bump
n = all_right.len()
for &el in all_right:                  // Element is Copy
    key, span onto indexed copy
```

Same occupants, same order. Dual-impl WHAT
unchanged. Do not skip the walk. Do not Session-Vec.

## ★ THE ONE CONTRACT DECISION

**Catch-up holds occupancy by Arc, not by memcpy.**
The indexed right still gets BindSpan copies
(`DESIGN-STONE-join-index-span`). Occupancy itself
is not rewritten.

## The gate

1. Leftover Instant: fanout catch-up / FIRE does
   not regress. Any drop counts.
2. 7strat 3/3 including three-stratum.
3. rete lib. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): fanout catch-up
drops the 40k-Element memcpy. Probe stays. Wrap W
stays physics.

## Blast radius

`fire/delta.rs` first-keying catch-up `all_right`.
No `.wat`. No Session field. No `AlphaMemory` type
change. `all_left` stays a Vec clone.

## Out of scope = REJECTED

- Session-Vec. Skip freeze. intern `names`. Array1. 297.
- Arc-wrap `wm.beta` to intern `all_left`.
- Occupancy as `Vec<u32>`. Skip packing Readings.

## Sequencing

1. Arc bump. Weigh. Stop.
2. Revert if 7strat red or leftover Instant regresses
   without a named drop.

## Weigh (2026-08-23) — LANDED

7strat 3/3 including three-stratum. Clippy `--lib -D warnings` silent.

Same-session leftover (mean of 3), before intern then after:

| | before | after |
|---|---:|---:|
| fanout without-query FIRE | 26.29 | **24.78** |
| fanout with-query FIRE | 30.24 | **29.93** |
| harvest:query | 6.01 | 6.16 |
| honest FIRE (census) | 25.43 | 25.90 |
| hash-join | 2.21 | 2.21 |
| probe | 1.71 | 1.70 |

Without-query FIRE **−1.51** (the catch-up cell). Occupancy held by Arc. `all_left` still clones `wm.beta`. Dual-impl WHAT unchanged. Do not Session-Vec. Do not skip the walk.
