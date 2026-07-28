# BRIEF — the defclause metadata-map must reach STDLIB defclauses

**Stone 1 of the `spawn-program` IPC wall (arc 170 closure #13).** Opened 2026-07-28.

## Why — a declared wall that enforces nothing

A prior strike added metadata-map support to `defclause`. It works for **user**
defclauses and is **silently inert** for **stdlib** ones. `spawn-program` is a stdlib
defclause (`wat/spawn.wat:262`), so the lockdown it was built for cannot be written.

**MEASURED, not inferred** (orchestrator, this session): `{:restricted-to [:wat::]}`
placed on the stdlib defclause `:wat::spawn::runner-count`, rebuilt, called from a
`:user::` fn → `wat --check` returned **CLEAN, exit 0**. The identical shape on a USER
defclause is rejected with a located `DefRestrictedCallerNotAllowed`. `wat/spawn.wat`
was restored byte-identical after the probe.

This is arc 278 R55/R57's masking class: a form that reads as guarded and guards
nothing.

## The root — two registration paths, one stores

| forms | path | writes `binding_metadata`? |
|---|---|---|
| USER | `register_defines` (`src/runtime.rs:745`, defclause arm ~`:899`) — called `freeze/env.rs:211` | **YES** (`:921-926`) |
| STDLIB | `register_stdlib_runtime_defs` (`src/runtime.rs:1056`, defclause arm ~`:1070`) — called `freeze/env.rs:275` (step 7.6) | **NO** — writes only `runtime_def_values` |

The enforcement walker itself is fine and must NOT change: `walk_for_restricted_call` /
`extract_prefix_list_from_metadata` (check.rs) read `binding_metadata` via
`CheckEnv::from_symbols`. The Rust-side `#[restricted_to(...)]` inventory drain
(`freeze/env.rs:224-243`) is a THIRD, working path — leave it alone.

## Read in order

1. `src/runtime.rs:880-930` — the USER defclause arm. The `binding_metadata` insert at
   `:921-926` is the shape to mirror.
2. `src/runtime.rs:1060-1080` — the STDLIB defclause arm. It has `cs` in hand (carrying
   `cs.metadata`) and drops it.
3. `src/runtime.rs:1147` — `preregister_stdlib_defclause_stub`. Ground whether it also
   needs the insert or is name-only; do not assume.
4. `src/value/value.rs:441-475` — `ClauseSet.metadata`, whose doc comment currently
   documents the gap. It must be rewritten to the post-fix truth.
5. `src/freeze/env.rs:205-280` — the pipeline, so the ordering is visible.

## The work

1. **Store the metadata on the stdlib path** — mirror the user arm.
2. **Update `ClauseSet.metadata`'s doc comment** to the new truth. Keep the two-paths
   table; delete the "silently inert" warning once it is false. Do not leave a comment
   that overstates coverage — that overstatement is what this stone exists to correct.
3. **Rehome the RED probes.** `wat-scripts/scratch-pad/arc-defclause-meta-probe/`
   currently holds three DELIBERATELY-RED files (`probe1.wat`,
   `probe3-non-keyword-key.wat`, `probe4-unexpected-extra-form.wat`). `wat-scripts/` is
   loader-gated by `tests/lint/wat_scripts_fixes_load.rs` — every `.wat` under it must
   LOAD. Those three are why the floor is currently `4104 passed, 1 failed`. Move their
   substance into committed Rust gates that assert the RED; delete the three `.wat`.
   `probe2.wat` and `probe5-stdlib-loads-clean.wat` are GREEN and may stay.
4. **Gate the mechanism.** Add a test that goes RED if the new `binding_metadata` insert
   is removed.

## The acceptance condition for the gate (this is the load-bearing row)

> **Delete the `binding_metadata` insert you just added. The gate must go RED. Restore it.
> The gate must go GREEN.** Report both observations.

A gate whose pass does not depend on the mechanism proves nothing about it (arc 278 R59
`NISI FRANGAS, NIHIL PROBAS` — a suite read 4105/4105 for weeks over a protocol that had
never once run). Assert on the restriction actually firing for a STDLIB-registered
defclause, not on a proxy.

## Blast radius

`src/runtime.rs`, `src/value/value.rs`, the new/changed test files, and the three deleted
`.wat` probes. No change to `check.rs`'s walker. No change to `wat/spawn.wat` or any
other `wat/*.wat`. No restriction added to `spawn-program` in this stone.

## STOP triggers — these are REJECTION criteria; ship nothing and report

- **STOP-1** — if storing the metadata on the stdlib path turns any EXISTING stdlib
  defclause red, STOP. No stdlib defclause carries a metadata-map today, so this should
  be inert for all of them; if it is not, the shape is wrong and the orchestrator
  re-plans.
- **STOP-2** — if the gate cannot be made to go RED when the insert is deleted, STOP.
  Report what you tried. A gate that cannot fail is the defect this stone is correcting.
- **STOP-3** — if the fix appears to require editing `walk_for_restricted_call` or
  `extract_prefix_list_from_metadata`, STOP. The walker is correct and shared with
  `def`/`defn`; a change there means the diagnosis was wrong.

## Gate

`cargo nextest run --release`. Read the Summary line by hand, ANSI-stripped. Never a
piped exit code (`… | tail` returns `tail`'s exit). Baseline before this stone is
`4105 tests run: 4104 passed, 1 failed` — the one failure is
`every_wat_scripts_file_loads_on_the_current_runtime`, caused by the three RED probes
item 3 removes. After this stone the floor is expected fully green.

## Out of scope — affirmatively cut, not deferred

- The restriction on `spawn-program` itself. It lands in a later stone, after
  `wat/test.wat`'s two macros stop splicing the call (see below).
- **`wat/test.wat`'s `run-thread` / `run-hermetic` macros.** Measured this session: the
  restriction check attributes to the **expansion site**, not the emitting macro (probe:
  a macro in an allowed namespace expanding a restricted call inside `:user::caller` is
  REJECTED, `:enclosing-fn ":user::caller"`). So the two macros that splice
  `spawn-program` (`wat/test.wat:312→322`, `:374→381`) must first route through a
  `:wat::test::` FUNCTION that holds the capability. That is its own stone; it is not
  this one.
