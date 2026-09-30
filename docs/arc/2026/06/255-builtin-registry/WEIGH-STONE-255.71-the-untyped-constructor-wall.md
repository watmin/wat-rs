# WEIGH — STONE 255.71: the untyped-constructor wall — STOP ACCEPTED, held local (floor red)

**Executor: a Sonnet subagent (resumed after the rate-limit stop), commits `2fb4578a9` and `332dd3f3e`, not pushed.**
Weighed by the orchestrator on 2026-09-30.

## What is built

- **The wall** (`src/check.rs`): `List`/`PersistentMap`/`PersistentVector`/`Tuple` now require their `:- [T…]` bracket,
  as `Vector`/`HashMap`/`HashSet` already did (arc 109).
- **The committed type table** `TABLE-STONE-255.71-typed-constructors.edn` (180 rows: 168 typed, 2 genuine STOP-1,
  6 `.wat.bad` legacy/positive controls, 1 stale), applied by the recorded codemod. The resumed 158 rows were verified.
  The wall's own floor found **two wrong rows**: three template sites marked STOP-1 whose type variable was in scope,
  and one `:O` that had to be the template's `~ret-ty`. The wall is doing its job.

## The floor: red, and why (orchestrator-read, `.floor/2026-09-30T23-01-26Z`, 62 failed / 6235)

The 177-site census covered the `.wat` **corpus**. The wall also reaches two populations nobody censused:

- **`wat-scripts/fixes/**`: 84 recorded codemods, 674 untyped constructor sites.** They are live wat programs (the loader
  gate type-checks them), so they now refuse to load, and every recorded migration's replay fails with them
  (`every_recorded_migration_replays` 34, `wat_scripts_fixes_load`).
- **Untyped constructors inside Rust string literals** (inline wat in `src/` unit tests and `tests/`): the `rete` tests
  (40), `rete_compile_gate` (32), and others. The agent converted 3 and listed about 40 files as a first pass, not a
  census.

## ⚠ One claim in the report is false

The agent called `probe_arc170_wrong_service_compile_error` *"pre-existing … almost certainly failed before 255.71
touched anything."* **It passed at 255.70** (`.floor/2026-09-28T22-13-36Z`: both rows `PASS`, 6225/6225). This stone
caused it. It belongs in the list of reds to cure, not to excuse. "Pre-existing" is not a disposition; the prior floor
is the measurement.

## For the builder

How the recorded codemods meet the wall (they are programs; a path exemption would be a hole in the wall). Then one stone
converts them and the Rust-literal fixtures, and fixes this stone's own reds, so the wall lands green.
