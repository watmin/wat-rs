# Phase B — Superseded by pass-10 architectural pivot

**Status:** archived 2026-05-10. Preserved as historical
record + reference for revised slice 3.

## What this is

The arc 170 slice 3 sweep ran in two phases:

- **Phase A** (committed as foundation): retired `wat/std/sandbox.wat`
  + `wat/std/hermetic.wat`; updated `src/check.rs` /
  `src/runtime.rs` / `src/stdlib.rs` / `src/spawn_process.rs`
  references. **Load-bearing.**
- **Phase B** (this archive): mass-edited ~50 test files +
  `wat-tests/*` + `wat/test.wat` to migrate
  `:user::main` from 3-arg to 4-arg + `:wat::kernel::ExitCode`
  return shape per slice 2's substrate; verb renames
  `fork-program*` → `spawn-process(fn)`. **Invalidated by
  pass 10.**

REALIZATIONS pass 10 reversed the 4-arg + ExitCode shape:
`:user::main` becomes `[] -> :wat::core::nil` (nil IS the
exit code). Phase B's signature edits no longer apply; the
work is superseded.

The diff is preserved in [`PHASE-B-SUPERSEDED.diff`](./PHASE-B-SUPERSEDED.diff)
— ~9400 lines covering 56 files. Applies via:

```bash
git apply docs/arc/2026/05/170-program-entry-points/PHASE-B-SUPERSEDED.diff
```

## What was proved (the value worth preserving)

### 1. Mass-mechanical sweep approach scales to ~50 files

Phase B demonstrated that a substrate-as-teacher walker-driven
mechanical sweep CAN migrate ~50 test files in coordinated
fashion. The pattern: walker fires on legacy shape;
substrate-as-teacher diagnostic names the new shape; sweep
applies. Revised slice 3 inherits this technique against the
new `[] -> :wat::core::nil` shape.

### 2. fork-program* → spawn-process verb migration patterns

Across `tests/wat_arc103_spawn_program.rs`,
`tests/wat_arc104_fork_program.rs`, `tests/wat_fork.rs`,
`tests/wat_run_sandboxed*.rs`, the verb migration shape worked.
**The verb renames are still valid post-pivot** — fork-program*
retires; spawn-process(fn) is the canonical shape per arc 170
DESIGN. Revised slice 3 reuses the migration pattern; only the
:user::main signature shape changes.

### 3. run-sandboxed* retirement is feasible

`tests/wat_run_sandboxed.rs` (-X heavy deletions) +
`tests/wat_run_sandboxed_ast.rs` (157 lines changed)
demonstrated that the run-sandboxed family can be retired in
favor of spawn-process. The retirement pattern transfers to
revised slice 3.

### 4. Walker assertions catch the legacy shape

Slice 2's walker variants (`BareLegacyMainSignature`,
`BareLegacyForkProgram`, `BareLegacySpawnProgram`) fired
correctly during phase B. Their walker firing tests proved
the substrate-as-teacher mechanism's design works.

### 5. Testing-lib reshape attempt (`wat/test.wat`, `wat-tests/test.wat`)

373 lines of `wat/test.wat` changes attempted the testing-lib
rebuild around `run-hermetic` / `run-hermetic-with-io` /
spawn-process layers. Pass 10 + 12 + 13 changed the
canonical shape (println/readln; nil-IS-exit-code; signal
model); the testing-lib rebuild needs to re-target the new
shape. **The 3-layer architecture from BUILD-PLAN.md slice 3
is informed by this attempt**: revised slice 3 starts from
DESIGN.md slice 3's three-layer spec, not from this diff.

## Why preserved instead of discarded

Per user direction 2026-05-10:

> *"we tend to just archive any proofs we needed to our arc
> dir.... i do not want to forget things we proved work"*

The proofs above ARE work-product. Discarding them would lose
verb-migration patterns + walker-firing-test designs that
revised slice 3 directly inherits. The diff lives in the arc
dir as a forward-reference artifact.

## How revised slice 3 references this

When slice 3's BRIEF lands (per BUILD-PLAN.md), the BRIEF
references this archive in two ways:

1. **Verb migration patterns** — slice 3 sweeps `fork-program*`
   → `spawn-process(fn)` callsites. The patterns in
   `PHASE-B-SUPERSEDED.diff` for `tests/wat_arc103_*.rs`,
   `tests/wat_arc104_*.rs`, `tests/wat_fork.rs`,
   `tests/wat_run_sandboxed*.rs` show the shape; revised
   slice 3 applies the same shape against the new
   `[] -> :wat::core::nil` signature.

2. **Testing-lib three-layer rebuild** — slice 3's testing-lib
   work re-targets the new shape. The phase B attempt at
   `wat/test.wat` reshape can be referenced for the SHAPE of
   the rebuild even though the SIGNATURE details have changed.

Revised slice 3 does NOT `git apply` this diff. It reads it for
pattern reference and writes fresh edits against the locked-in
post-pass-13 architecture.

## Cross-references

- [`BUILD-PLAN.md`](./BUILD-PLAN.md) §1 — dirty-tree audit
  named phase B BACK OUT decision (D1)
- [`BUILD-PLAN.md`](./BUILD-PLAN.md) §3 — slice 3 (revised)
  scope; informed by patterns archived here
- [`DESIGN.md`](./DESIGN.md) — slice 3 spec for the new
  testing-lib three-layer shape
- [`SCORE-SLICE-2.md`](./SCORE-SLICE-2.md) — what slice 2
  shipped (the substrate phase B was sweeping over)

## What's NOT in this archive

- **Phase A retirement** (sandbox/hermetic deletions, src/*
  reference cleanups) — load-bearing for foundation; lives in
  the foundation commit, not here
- **Slice 1d closure-extraction walker fix** (src/closure_extract.rs
  + tests/wat_arc170_closure_extraction.rs) — load-bearing for
  foundation; lives in the foundation commit, not here
- **Already-shipped slice 1 / 1b / 1c / 2 work** — historical
  per their respective SCORE docs; not part of dirty tree

This archive captures ONLY phase B (the test sweep that pass-10
invalidated).
