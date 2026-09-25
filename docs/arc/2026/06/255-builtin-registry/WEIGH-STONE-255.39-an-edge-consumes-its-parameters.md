# WEIGH — STONE 255.39: a surface parameter is consumed by the edge that binds it — ACCEPTED

**Executor: grok via pulsare, commit `1b16794a6`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured

| row | measured | result |
|---|---|---|
| floor | `.floor/2026-09-25T20-57-23Z` | `6127 tests run: 6127 passed (10 slow), 22 skipped` |
| `owner-end?` in `wat/`, `src/` | grep | **0** |
| `Spawned` | `wat/spawn.wat:258-259` | `(defsurface :wat::spawn::Spawned :- [S R] :nature :wat::core::Struct :features [])` |
| a featureless surface with an edge / with no edge / a struct with an unused parameter | `--check tests/types/probe_arc255_39_*` | rc **0** / **1** / **1** (pre-stone: 1 / 1 / 1) |
| **one rule, not two** | `flush_surface_param_debt` (`src/types.rs:5008`) | it calls **the same** `check_type_params_consumed(&def, &span, Some(env))`; the flush only supplies the edges, and does not define consumption |

Taken from the SCORE: clippy 0; census `no STOP-8` with 0 rc flips (including a checker-only census taken
before `spawn.wat` changed); delta NEW 2 / RECOVERY 0; ledger 198; the first floor's red (loose asserts in
the new row) was fixed and the whole floor re-run.

## What landed

- **H5 is structural.** A surface parameter that no feature uses is remembered at declaration and judged
  once every edge is registered (end of `register_types_impl`, one walk for stdlib and user). An edge
  consumes it when its target head is that surface and it binds the slot. A surface already settled by its
  features is judged at declaration, as before. Records, structs and enums are unchanged.
- **`Spawned` is honest:** a featureless surface whose doc says what it is, and whose `Thread`/`Process`
  binder edges write what `S`/`R` are. **The phantom feature is gone.**
- The brief omitted `:features`, which the declarator requires; `:features []` is honest, and **the code
  won**.

## Next, as ruled

The `recv` vocabulary split (`PeerRecvOutcome` / `OwnerRecvOutcome`, named in
`FINDING-INTUERI-naming-the-two-recv-vantages.md`). C1 is now resolved: an owner and a peer are two types.
Then `ServiceEvent` (vocabulary 5 of 5), including `Rejected` → `Oversized`.
