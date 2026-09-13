# BRIEF 2a4 — the door reads a stdlib file as stdlib; a step that changes the stdlib converts in two phases

> Batch 1 is paused at #22 (`SCORE-4-replay-batch-1.md`, STOP-2); this stone unblocks it. The
> orchestrator's probe of the stdlib mode PASSED (below).

## Why — measured

Batch 1 stopped at #22 `8eeff8adc`: grok-rete promotes `wat-scripts/lib/gen.wat` to the stdlib as
`wat/gen.wat` (`:wat::gen::*`). Three failures, one cause:
- the door registers a program with USER privilege (`src/freeze/env.rs:134`,
  `register_types_with_acronyms` → `register_with_span` → `Privilege::User`), so every `:wat::gen::`
  type and the `record` defmacro come back `UNREGISTERABLE ReservedPrefix`, and match-arm and
  positional-ctor leave `wat/gen.wat` unconverted;
- baked unconverted through `include_str!`, it stops the binary starting (`ReturnTypeMismatch`
  `wat/gen.wat:143`), so no codemod can run on that binary;
- its consumers (`gen-selftest.wat`, …) ask `type-of` in HEAD's world, which has no `wat/gen.wat`.

**The class, not the case** (`bootstrap/era/replay-plan/stdlib-touch.tsv`): 34 grok-rete commits from #22
touch a stdlib `wat/*.wat` file (2 add one; 25 also change its consumers); 19 of batch 1's remaining 39.
And for a stdlib file a commit CHANGES, the door today refuses the changed type as a duplicate and the
codemod answers it from HEAD — the file's PREVIOUS version: a new or reordered field converts stale,
silently.

## The shape

1. **The door has a stdlib mode: `build_env`'s STDLIB half on these forms**, as the door today runs its
   user half. Exactly the three stdlib calls `build_env` makes (`src/freeze/env.rs:306-312`, `:340`):
   `register_stdlib_defmacros`, then `crate::macros::expand::expand_all_with(…, Privilege::Stdlib)` (so a
   companion macro `defstruct`/`defrecord` mints during expansion registers — the probe's second run was
   refused `ReservedPrefix: cannot declare macro :wat::gen::Gen` under plain `expand_all`), then
   `register_stdlib_types` (`src/types.rs:3682`, `register_stdlib_with_span` `:809-815`). A type the file
   declares that the snapshot already holds DIVERGENTLY (`register_validated`'s `Existing::Divergent`,
   `:821-835`) is REPLACED by the file's declaration — in the door's private per-call copy only. It returns
   every type the file declares (not only the new ones). Name the verb (a sibling of
   `:wat::runtime::declared-types`, or a mode of it); one row of `@example`, one `@arg`, as 2a1 did.
   - A real clash between two stdlib files is not hidden: the step's `cargo build` + startup still refuse it.
2. **The codemods ask the stdlib mode for a file under `wat/`** (the stdlib source directory,
   `src/load/stdlib.rs`'s `include_str!` home): `wat/fix.wat`'s `enum-fields` chooses the mode by the path.
3. **A step that touches a stdlib file converts in two phases** (`convert.sh` + the recipe):
   (a) convert that step's `wat/*.wat` files (stdlib mode); (b) put them in place and `cargo build
   --release`, so the binary carries their converted text; (c) convert the step's other `.wat` with the
   rebuilt binary — their stdlib questions are now answered by `type-of` in the new world. The C^ set uses
   HEAD's binary as today (HEAD already carries C^'s stdlib).
4. **Fixtures:** a new stdlib file (the #22 shape); a stdlib file that CHANGES an enum's fields (the
   converted constructor uses the NEW fields); a consumer in the same step.

## The probe (orchestrator, before release — FM 2-bis) — PASSED

Temporary test (`bootstrap/era/probe-R/zz_probe_stdlib_mode.rs.snippet`, run on a stashed tree, reverted):
`register_stdlib_defmacros` → `expand_all_with(…, Privilege::Stdlib)` → `register_stdlib_types` on the
staged `wat/gen.wat`: `record` macro registered; `:wat::gen::Gen`, `GenAcc`, `CheckOutcome`, `Pick`,
`GenRev` all registered; `CheckOutcome` = `Checked [points violations]`, `EmptySpace []`. Control on the
same text through today's user door: `ReservedPrefix: cannot declare macro :wat::gen::record`.
Three runs to get here, each a real lesson: raw era text (the rename steps had not run), then plain
`expand_all` (companion macros need Stdlib expansion), then the full stdlib half.

The contract it probed:

Stdlib-privilege registration of #22's `wat/gen.wat` AS THE CHAIN HANDS IT TO STEP 23 (the staged
convert.sh output: renames applied) into a copy of the snapshot registers `:wat::gen::CheckOutcome`
(variants and fields), `Gen`, `GenAcc`, `Pick`, `GenRev`, and the `record` macro — with no ReservedPrefix.
First run (raw `8eeff8adc` text) was the wrong input: `register_stdlib_defmacros` refused `record` because
its body still said `:wat::core::string::concat`, which steps 1–12 rename before the door ever runs.

## The bar

- Resume batch 1 at #22 from the staged overlay: `wat/gen.wat` converts (0 UNREGISTERABLE ReservedPrefix),
  the binary starts, `gen_library_satisfies_its_own_laws` and `rete_fuzzer_finds_no_native_oracle_divergence`
  pass by name at #22.
- `run5.sh` unchanged (0 files losing; the chain vs main ≥ 1370) — the corpus's stdlib files now go through
  the stdlib mode.
- Floor + clippy.

## STOP triggers

- **STOP-1:** a stdlib file's own type still answers stale or refused after the stdlib mode. Report the
  file, the type, both definitions.
- **STOP-2:** the floor is red. Paste the whole block verbatim, and do not re-run.

## Tier

Commit on green. **Do not push.** Yield with `SCORE-2a4.md`; then batch 1 resumes at #22.

## Order of work — your #22 overlay is still staged

The staged #22 overlay bakes the UNCONVERTED `wat/gen.wat` into the stdlib (`src/load/stdlib.rs`
`include_str!`), so every build on that tree fails. So:
1. `git stash push -m "grok #22 overlay"` (keep the index: it holds your `git rm`/add of `gen.wat`);
2. strike 2a4 on the clean tree (floor + clippy green), commit it, yield nothing yet;
3. `git stash pop --index`, then redo #22 under the two-phase recipe with the new door: convert
   `wat/gen.wat` (stdlib mode), `cargo build --release`, convert its consumers, gate, commit
   `REPLAY(grok-rete #22)`;
4. continue batch 1 (#23 → #60) under BRIEF-4, its addendum, and this two-phase rule for every step that
   touches `wat/*.wat`. Yield after #60, or at the first STOP, with `SCORE-2a4.md`,
   `SCORE-4-replay-batch-1.md` and `REPLAY-LOG.md`.
