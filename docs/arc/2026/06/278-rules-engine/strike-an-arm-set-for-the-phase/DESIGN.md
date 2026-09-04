# DESIGN — six arms measure a branch the engine does not take; the phase has never had an arm set

## Why

Work-list **C12**, opened by C6's strike: *"NO ARM IN THIS FILE MEASURES THE FILTER PHASE AS IT
EXISTS."*

C6 landed the honest half — it deleted a stale constant, read the phase live, and then **refused to
assert** the reconstruction check because it failed at ~7x. That refusal is still standing. C12 is
the other half: **build the arms.**

## Driven at HEAD `deecfac6e`

```
STEP 0 — where-predicate cost decomposition, node-share [50 200], ONE ROUND's worth per arm
  A  env build alone         ( 10000 x)     1.858 ms
  B  env build + walk        ( 10000 x)     9.799 ms
  C  token clone             (    50 x)     0.132 ms
  D  env + walk, VAR-FREE    ( 10000 x)     6.170 ms
  E  hand-written Rust       ( 10000 x)     0.058 ms   <- THE FLOOR
  F  compiled exec_where     ( 10000 x)     2.617 ms
  RECONSTRUCTION  F+C =  2.749 ms  vs a LIVE `filter` of  0.419 ms  ( 656% accounted)
```

**A, B, D, E and F all measure `exec_where` and its env build. The fire calls `exec_where` ZERO
times on this axis** — `dispatch_where_tests` takes the `proven && is_pure_cmp` reuse branch
(`fire/mod.rs:2038`), which the file's own header records at `:63-65`. Only **C** measures work the
engine performs, and C is **0.132 / 0.419 = 31.5%** of the phase.

**~68.5% of the filter phase has no arm.**

## What the phase actually does

`dispatch_where_tests` (`fire/mod.rs:2012`), per token, in the branch the fire takes:

```rust
let binds  = bind_view(...);                                  // 1
let cands  = sink.where_tree.candidates(&binds, &span);       // 2
let proven: HashSet<i64> = cands.proven.into_iter().collect();// 3
let maybe:  HashSet<i64> = cands.maybe.into_iter().collect(); // 4
for &tid in tids { ... covers / contains ... }                // 5
    sink.d_beta.entry(tid).or_default().push(*tok);           // 6
```

Six mechanisms. **None of them has an arm.** The benchmark's ladder measures the `else` branch at
line 56 — the one reached only when the where-tree cannot prove the test.

## The contract decision, pinned

**Build a cumulative arm set over the branch the fire TAKES, and let the reconstruction say what it
covers — measured, not arranged.**

- New arms, cumulative in the existing A→B style so each row is a delta:
  `bind_view` alone → `+candidates` → `+the two HashSet builds` → `+the tid loop` → `+d_beta pushes`.
- **The existing A/B/D/E/F stay**, relabelled to say they measure the `exec_where` branch, which this
  axis does not take. They are the headroom story (`B-E` is what a perfect compile could remove) and
  deleting them would discard a real measurement. **They must stop being summed into a
  reconstruction of a phase they are not in.**
- **The reconstruction check states coverage as a MEASURED fraction with its spread**, and asserts an
  invariant — not a number. If the new arms still do not reconstruct the phase, **that is the
  result**, recorded the way C6 recorded its refusal, with the samples beside it.

⛔ **The failure mode this strike must not commit.** C6 proved the declared check fails at ~7x and
refused to assert it. An arm set that happens to land closer, asserted because it looks better, is
the same claim with better luck behind it. **Coverage is measured and stated; it is never arranged.**

## ⚠ One sample is not a shift

C6 recorded **684 / 693 / 734 / 723 / 686 / 698%** over six runs. This design's drive read **656%** —
outside that band. **One sample cannot establish a change**, and `[[six-samples-or-no-number]]` says
so. The strike re-measures the pre-value at six samples before it reasons from any movement.

## Out of scope = REJECTED

- **Making the axis take the `exec_where` branch.** The sizes are a recorded artifact and the branch
  choice is the engine's. If a workload that exercises `exec_where` is wanted, it is a new axis with
  its own ladder rung, named — not a re-dial of this one.
- **Deleting A/B/D/E/F.** They measure something real about the compiled path's headroom. The defect
  is the reconstruction that sums them into a phase they do not run in.
- **Optimising anything.** This strike measures. A hot-path change on the strength of a
  newly-honest number is the next strike, with its own before/after.
