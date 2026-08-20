# DESIGN-STONE — split cascade SETUP without per-node timers

> **Origin (2026-08-20).** 15: deep-cascade `[50 100]` leads
> native FIRE **30.12**. `a0` at the same cell: SETUP
> **12.70** (seen 0.004) vs depth-10 SETUP 0.77. ROUND
> 17.36. We do not know whether 12.7 is `rete_arm_get_or_build`
> (intern miss / first-build) or the rest of SETUP (clears,
> index maps, scratch). Guessing is how this arc interned
> the wrong row. This stone prints the split. It does not
> intern off an unranked lump.

## The measurement we do not have

SETUP is one mark around: wm clears, `setup:seen`,
`rete_arm_get_or_build`, join-index HashMaps, match scratch.
Compile-all already interns the arm (`export.rs`). Fire
should HIT. If it MISSES, arm is the 12 ms. If it HITS,
remainder is the 12 ms. `ARM_BUILDS` already counts builds.

## The algorithm

In-fire, **one** extra pair (not per node):

```
setup:arm   — rete_arm_get_or_build
```

Outer SETUP stays. `setup:seen` stays. Remainder =
SETUP − seen − arm. Tax: 1 × cal.

Mean of 3 at `[50 100]`. Print ARM_BUILDS delta per run.

1. **STOP intern** if neither arm nor remainder is ≥ 1 ms.
2. If arm ≥ 1: say whether ARM_BUILDS moved (miss) or
   not (hit, lookup is the row). Do not intern a second
   intern table this stone.
3. If remainder ≥ 1: say so; do not split remainder
   further this stone.
4. Do not intern accum `setup:seen`. Token stays two spans.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the
engine** beyond the empty-in-release mark. Do not restore
per-node timers. Do not intern off this rank until a named
leftover is ≥ 1 ms.

## The gate

1. `cascade_setup_leftover_split` prints SETUP / seen /
   arm / remainder / ARM_BUILDS. SETUP > 0. Do not
   wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **`setup:arm` owns
≥ 10 ms** of 12.70 (50-rule first-build or intern miss).
ARM_BUILDS moves once per census world. If remainder
wins, leftover is clears + HashMap init; say so.

## Blast radius

`kernel.rs` one coarse mark + one test. No `.wat`. No
crate. Token stays two spans.

## Out of scope = REJECTED

- Per-node SETUP timers. Intern this stone off an
  unranked lump. Intern `names`. 2e / 2o. 297.
- Fact insertion. Session-`Vec`. Fold accum `setup:seen`.
- Persist gather. Second arm intern table.

## Sequencing

1. Mark. Test. Print. Rank.
2. Neither ≥ 1 → stop.
3. Else name the intern. Do not intern this stone.

## Weigh (2026-08-20) — LANDED, no intern

`cascade_setup_leftover_split` `[50 100]`, mean of 3.
Gate: rete lib 95, clippy `-D warnings` silent.

| lump | ms |
|---|---:|
| SETUP | **12.52** |
| setup:seen | 0.00 |
| **setup:arm** | **12.51** |
| remainder | 0.01 |
| ARM_BUILDS | **1.00 / run** |

Prediction held. Remainder is dead. The 12.5 ms is
`rete_arm_get_or_build` **MISS**: WAT `compile-all`
builds the Session/network and does **not** intern
the rust `ReteArm`. Fire builds it. Item 12's second
fire HIT still holds (`fire_rules_reuses_arm_…`).

Named intern: intern the arm when `compile-all`
returns the Session, so first `fire-rules` HIT.
Predicted: cascade FIRE 30.12 → **~17.6**. Grid
`:wat-ns` should drop the same if it currently
includes first-fire arm build. Do not intern insert.
Do not intern a second intern table. Do not start 297.
