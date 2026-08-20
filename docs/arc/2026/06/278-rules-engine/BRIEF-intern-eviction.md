# BRIEF — intern eviction on last lease

## The work

`arm-session` takes a lease. `release-session` drops one.
At zero the intern entry is removed. Session stays 8
fields. Fire HIT does not lease.

## Read in order

1. `DESIGN-STONE-intern-zero-mutex.md` (must land first).
2. `DESIGN-STONE-intern-eviction.md`.
3. `src/rete/kernel/arm.rs` intern + `eval_arm_session`.
4. Item 12 in CURRENT-STATE — Weak died; do not revive it.

## Sketch

```
entry { arm, leases }
arm-session     leases += 1 (MISS: intern 1)
fire HIT        unchanged
release-session leases -= 1; 0 → remove
```

## STOP

1. **STOP-1** — Weak table. 9th Session field. Drop intern
   when fire returns.
2. **STOP-2** — intern `names` / facts / Session-`Vec` /
   2e / 2o.
3. **STOP-3** — 297. Service-ify. Content hash. Recast
   vigilia. Stamp `vigilatum`.

## Done

- Release then rebuild: ARM_BUILDS += 1.
- Two distinct compile-alls: release one, the other HIT.
- Overlay is not a second lease. Overlay reuse still green.
- Public `release-session` mouth drops the compile lease.
- rete lib 104. clippy `-D warnings` silent.

Leave dirty.
