# BRIEF — intern cond slot keys once per fire

## The work

Intern each compiled cond's `slot_keys` once at fire SETUP.
`materialize_into` copies `u32` ids. Tests may still scan.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 21, intern_key 1.18.
2. `DESIGN-STONE-bind-key-intern.md` (fire-scoped keys).
3. `compiled_cond.rs` `intern_key` / `materialize_into`.
4. `DESIGN-STONE-cond-key-ids.md`.

## Sketch

```
cond_key_ids[aid] = intern_cond_keys(cond, bind_keys)  // SETUP
materialize_into(..., Some(&ids))
```

## STOP

1. **STOP-1** — process-lifetime `?var` intern / intern `names`.
2. **STOP-2** — 2e / 2o / 297 / insertion.
3. **STOP-3** — change isolated K so K−C goes quiet.

## Done

- Fire SETUP intern once. Isolated K−C still prints.
- rete lib green. clippy silent.

Leave dirty.
