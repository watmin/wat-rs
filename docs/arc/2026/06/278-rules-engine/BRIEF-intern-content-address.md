# BRIEF — intern is content-addressed (Athena HIT)

> **RULED 2026-08-20 — REJECTED.** Discrete intern per
> connection. Do not hash rules. Do not hash queries.
> Do not share across compile-all. Overlay HIT (same
> connection) stays. See the DESIGN-STONE banner.

## The work

Intern key is a structural hash of `Session.rules`, not
`PMap::rust_identity`. Two `compile-all` of equal rules
HIT. Overlay HIT stays. TLS cache + tier-3 keeper so
the HIT crosses workers. Lease count from stone 28
covers two connections on one intern.

## Read in order

1. `DESIGN-STONE-intern-zero-mutex.md` (27).
2. `DESIGN-STONE-intern-eviction.md` (28).
3. `DESIGN-STONE-intern-content-address.md`.
4. `docs/ZERO-MUTEX.md` tier 3 (keeper, not Mutex).
5. `src/value/pmap.rs` `rust_identity` vs `Hash`.

## Sketch

```
key = hash(session.rules)
TLS HIT / else keeper Lookup
MISS: build, Intern, lease=1
HIT:  lease++ (arm-session only)
```

## STOP

1. **STOP-1** — Mutex / `RwLock` / `AtomicPtr`. Hash of
   facts / `rust_identity` / EDN text. Intern `names`.
2. **STOP-2** — Session-`Vec` / 2e / 2o / query-encode
   intern / scratch intern.
3. **STOP-3** — 297. Service-ify. Stamp `vigilatum`.
   Call TLS-only "Athena done."

## Done when

- Two compile-alls, equal rules, one thread: ARM_BUILDS 1.
- Two threads, equal rules: ARM_BUILDS 1 (keeper).
- Unequal rules: MISS.
- Both released: next compile rebuilds.
- Overlay reuse green. `rg Mutex src/rete` empty.
- rete lib. clippy `-D warnings` silent.

Leave dirty.
