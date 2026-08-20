# DESIGN-STONE — intern the arm when `compile-all` returns

> **Origin (2026-08-20).** 16: cascade `[50 100]` SETUP is
> **`setup:arm` 12.51**. Remainder 0.01. ARM_BUILDS 1.00/run.
> WAT `compile-all` builds the Session/network and does **not**
> intern the rust `ReteArm`. First `fire-rules` builds it.
> Item 12's second-fire HIT still holds. Item 12's *contract*
> was "compile puts the arm here." This stone completes it.

## The measurement we have

First fire pays 12.51 ms inside SETUP. Compile already
ran. The rust arm is a fire-time leftover of a compile-time
job. Grid `:wat-ns` is fire-only — this 12.5 ms does not
belong there.

## The algorithm

Native `(:wat::rete::arm-session' session) → session`.
Side effect: `rete_arm_get_or_build`. Value unchanged.
`compile-all` wraps the Session constructor with it.
`compile` already calls `compile-all`. No second intern
table. Token stays two spans.

1. **STOP** if `network_identity` is None on a compile-all
   Session — intern cannot HIT; do not ship a no-op.
2. Do not intern on `insert`. Do not intern `names`.
3. Oracle `fire-rules-spec` still ignores the arm.

## ★ THE ONE CONTRACT DECISION

**`compile-all` intern's the `ReteArm` under the network
identity. First `fire-rules` HIT.** Session bytes are
unchanged. The arm table is the same process-lifetime
map item 12 minted.

## The gate

1. `cascade_setup_leftover_split` `[50 100]`: `setup:arm`
   net **< 1 ms**. SETUP printed. ARM_BUILDS still
   **1.00/run** (compile paid the build). Do not
   wall-gate FIRE.
2. `fire_rules_reuses_arm_across_fire_and_insert_overlay`
   still green.
3. rete lib.
4. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **`setup:arm` 12.51 →
~0.** SETUP 12.52 → **~0.01**. Cascade FIRE 30.12 →
**~17.6**. ARM_BUILDS stays 1/run.

## Blast radius

`kernel.rs` one native eval. `runtime.rs` dispatch.
`check.rs` TypeScheme. `purity.rs` completeness ledger.
`wat/rete.wat` `compile-all` wrap. No Session field.
No `.wat` tests beyond compile-all.

## Out of scope = REJECTED

- Intern on insert. Second intern table. Session-`Vec`.
- Intern `names`. 2e / 2o. 297. Fact insertion.
- Per-node timers. Fold accum `setup:seen`.

## Sequencing

1. Native + wrap. Gate arm < 1 ms.
2. Weigh FIRE. Stop.

## Weigh (2026-08-20) — LANDED

`cascade_setup_leftover_split` `[50 100]`, mean of 3.
Gate: rete lib 95, clippy `-D warnings` silent.
`fire_rules_reuses_arm_…` green.

| lump | before | after |
|---|---:|---:|
| setup:arm | 12.51 | **0.00** |
| SETUP | 12.52 | **0.01** |
| ARM_BUILDS / run | 1.00 | **1.00** (compile) |
| cascade FIRE | 30.12 | **17.62** |

Prediction held (17.6). `cell_rank_after_grid` after intern:

| cell | FIRE | top-row |
|---|---:|---|
| fanout `[100 20]` | **26.27** | production 17.39 |
| accum `[200 200]` | 21.11 | alpha 12.98 |
| deep-cascade `[50 100]` | 17.62 | production 4.99 |

Fanout leads native FIRE again. Cascade SETUP is dead.
Next leftover is fanout production (2p ranked that pile)
or cascade ROUND. Do not intern insert. Do not start 297.
