# BRIEF 2a4c — the stdlib door replaces a divergent MACRO, and walks a stdlib file as it walks a user file

> Found by the orchestrator verifying 4b (finding 19). Probed. Batch 1 unaffected; batch 2 (#61–#125)
> unaffected; 6 later steps need it, the first at #155.

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Never use worktrees.

## Why — measured (`bootstrap/era/probe-R/door-stdlib.wat`, HEAD `33f3ebcfb`'s binary)

2a4's stdlib mode replaces a type the file declares divergently — probed: a copy of `wat/grep.wat`
whose `defrecord Node` gains `depth` answers `[ id parent index kind depth ]`, at top level and
wrapped in a `do`. Two things it does not do:

1. **It refuses a divergent MACRO.** `register_stdlib_defmacros` registers against the snapshot's
   clone, and the registry's one gate refuses a divergent re-definition (`src/macros/registry.rs:71-93`;
   equivalence is structural, span-agnostic). The era's `wat/core.wat` through the door:
   `REFUSED … #wat.macro/DuplicateMacro {:message "duplicate macro registration: :wat::core::defn" …}`.
   So in a step that CHANGES a stdlib macro, the door expands that file with the snapshot's OLD
   macro; where the macro mints types (`defservice`, `sift-rules-defsvc`), the codemods answer those
   types stale, visible only as one `UNREGISTERABLE … DuplicateMacro` report line.
   - Batch 1: #31 #38 #40 #43 changed `:wat::gen::record`, which builds an expression, mints no type,
     and has no top-level use — no answer changed (verified).
   - Later: 6 steps change 4 stdlib macros — #155 #157 #159 #226 #377 #379 (`sift-rules-defsvc` ×3,
     `sieve-pred` ×2, `defservice`, `defquery`; `bootstrap/era/replay-plan/future-macro-changes.txt`).
2. **It expands every `defn` body.** The user half keeps only what can declare a type (2a1's one-step
   walk, `collect_type_forms`); the stdlib half runs `expand_all_with` over every form, so a body that
   cannot expand in the door refuses a form that declares nothing:
   `wat/kernel/services/stdio.wat ProgramBodyEvalFailed …:wat::kernel::start-primed-stdio`,
   `wat/service.wat ProgramBodyEvalFailed … line=2443`.

**The baseline** (`bootstrap/era/probe-S/run5.sh` at `33f3ebcfb`, now handing repo-relative paths —
the orchestrator's instrument fix, finding 19): MA 0 losing · PC 2 losing (the 2a2 artifact, and
`wat/service.wat`, which PC's own `skip-path?` now honours) · VS 0 losing, report lines 34 — of which
**11 `DuplicateMacro` and 2 `ProgramBodyEvalFailed` on `wat/` files**; chain vs main 1370.

## The shape

1. **The macro mirror of the type rule.** In `register_declared_stdlib_types` (`src/freeze/env.rs`): a
   `defmacro` the file defines that the cloned registry holds DIVERGENTLY (not
   `macro_structurally_equivalent`) is retracted from the door's copy before `register_stdlib_defmacros`;
   an equivalent one stays a no-op. The copy only.
2. **The stdlib half walks.** It keeps only what can declare a type, through the same one-step walk the
   user half runs (`collect_type_forms`), with Stdlib-privilege expansion so a companion macro minted
   mid-walk still registers (2a4's probe: `defstruct`'s companion). A `defn` body is never expanded.
3. **A stdlib-file refusal is STOP-9.** After 2a4c the stdlib door reads every current `wat/` file as it
   declares itself, so `convert.sh` reporting `UNREGISTERABLE` for a `wat/…` path is **STOP-9** in
   BRIEF-1 § "One step" 3 (the check that fires where this gap hid behind a report line).

## Fixture — it must carry a divergence AGAINST THE SNAPSHOT

A probe macro is not in the real snapshot, so one call through the verb cannot diverge. Chain the Rust
door: call 1 registers file A (macro `:wat::probe2a4c::mk` minting
`(:wat::core::defenum :wat::probe2a4c::E :wat::enum::Pure :V [a <- :wat::core::i64])`, plus a top-level
`(:wat::probe2a4c::mk)`); call 2's snapshot registries are call 1's OUTPUT, and file B changes `mk` to
mint `:V [b <- … c <- …]`. Call 2 answers `E` = `V [b c]`. Under the mutation that removes the macro
retraction: `DuplicateMacro :wat::probe2a4c::mk` (captured). A second fixture: a stdlib-mode file whose
`defn` body cannot expand in the door returns its types, not a refusal.

## The bar

- Both fixtures GREEN; the first RED under its mutation (captured, verbatim).
- The era `wat/core.wat` through the door: Ok, no `DuplicateMacro`.
- `run5.sh`: 0 files losing on MA and VS, PC's 2 unchanged; chain vs main ≥ 1370; the `wat/` refusals:
  `DuplicateMacro` 11 → 0, `ProgramBodyEvalFailed` 2 → 0. Any `wat/` refusal left (MA's
  `MalformedDefmacro` on the era's `core.wat` `format` and `service.wat` `defservice` are expected to be
  era bodies naming verbs today's gates refuse) is named with its full cause and explained — or STOP-1.
- `census.sh` at the new HEAD: `--diff` against `.census/latest` prints no STOP-8.
- Floor + clippy.

## STOP triggers — rejections; report verbatim

- **STOP-1:** a current `wat/` file is still refused by the stdlib door after 2a4c — the file, the form,
  the cause.
- **STOP-2:** a floor is red — paste the whole block, do not re-run.

## Tier

Commit on green. **Do not push.** Yield with `SCORE-2a4c.md`.
