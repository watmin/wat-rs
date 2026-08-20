# BRIEF — intern the arm when `compile-all` returns

## The work

`compile-all` intern's the rust `ReteArm` so first
`fire-rules` HIT. Session value unchanged.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 16, arm 12.51.
2. `DESIGN-STONE-cascade-setup-split.md` weigh.
3. `kernel.rs` `rete_arm_get_or_build` / `eval_fire_rules_native`.
4. `wat/rete.wat` `compile-all`.
5. `DESIGN-STONE-arm-at-compile.md`.

## Sketch

```
eval_arm_session'  — get_or_build, return session
compile-all        — (arm-session' (Session …))
```

## STOP

1. **STOP-1** — intern on insert / identity None / no-op wrap.
2. **STOP-2** — intern `names` / facts in `bind_pool` / 2e.
3. **STOP-3** — 297. Fact insertion. New intern table.

## Done

- `setup:arm` < 1 ms at `[50 100]`. ARM_BUILDS 1/run.
- rete lib green. clippy `-D warnings` silent.

Leave dirty.
