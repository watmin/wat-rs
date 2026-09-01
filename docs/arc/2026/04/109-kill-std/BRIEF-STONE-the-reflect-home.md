# BRIEF — STONE: `src/reflect/` — the introspection surface leaves the megafile

Move 33 items (~2,500 lines) out of `runtime.rs` into `src/reflect/`, split by role. DESIGN:
`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-reflect-home.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `./target/release/wat` and `--check` for a fast read. **You may not
spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not
commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at 5114 —
anything that breaks is yours.

## Read in order

1. The DESIGN above — the stale-span hazard, and the one item inside the range that must NOT move.
2. **`src/declare/`** — the stone that shipped immediately before this one, same shape. Read its five
   files and their `use` blocks; that is the standard here.
3. `src/intrinsic/reflect.rs` — **the EDGE this home serves.** It already exists; you are giving its
   implementations a home. Do not put implementations in it.
4. `src/value/environment.rs:148` · `src/value/symbol_table.rs:32` — where `Environment` and
   `SymbolTable` live. Read STOP-1 before writing a single `use`.

## The work

### 1 — create `src/reflect/`, split by ROLE

Declare `pub(crate) mod reflect;` in `src/lib.rs`. Suggested files — **verify the grouping against
the bodies and say so if it does not hold**:

**render.rs** — internal state → AST
`eval_struct_to_form` 8096 · `type_expr_to_ast` 8189 · `binder_head_nodes` 8231 ·
`function_to_signature_ast` 8256 · `function_to_define_ast` 8307 · `type_scheme_to_signature_ast`
8330 · `primitive_to_define_ast` 8361 · `macrodef_to_signature_ast` 8416 · `macrodef_to_define_ast`
8471 · `typedef_to_signature_ast` 8496 · `typedef_to_define_ast` 8529 · `name_from_keyword_or_fn` 8573

**lookup.rs** — find a binding
`enum Binding` 8608 · `lookup_form` 8656 · `eval_lookup_define` 8765

**verbs.rs** — the `*-of` API surface
`eval_signature_of_defn` 8880 · `eval_signature_of_fn` 9002 · `eval_return_type_of` 9068 ·
`eval_body_of` 9162 · `eval_metadata_of` 9281 · `require_ast_children` 9509 ·
`eval_rename_callable_name` 9568 · `eval_extract_arg_names` 9764 · `eval_extract_arg_types` 9870 ·
`eval_field_names_of` 9979 · `eval_field_types_of` 10037 · `resolve_type_keyword_arg` 10078 ·
`resolve_aggregate_def_for_reflection` 10130

**match.rs** — form matching
`eval_form_matches` 10203 · `walk_match_clause` 10306 · `eval_forms` 10493

**expand.rs** — `eval_macroexpand_1` 10508 · `eval_macroexpand` 10561

`mod.rs` holds the module doc and the `mod` declarations. **It is not a bucket** for items that
resisted sorting — place by measuring callers and say which callers decided it.

### 2 — the call sites

Only **2** are external (`src/intrinsic/reflect.rs`, one test). Everything else is `runtime.rs`'s own
dispatch and in-file callers. **The compiler is the census** — fix what it names; do not hunt by hand.

Leave a short retirement comment at each cut, in the shape `src/declare/`'s stone used.

## Blast radius

`src/reflect/` (new) · `src/lib.rs` (one `mod`) · `src/runtime.rs` (33 items out) · the 2 external
sites. No `.wat` corpus change. No registrations added or removed. **No verb's behaviour changes.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — `require_bundle` (line 9486) MUST NOT MOVE.** It sits between `eval_metadata_of` and
`require_ast_children` and is NOT in §1's list. Both its callers are `src/intrinsic/holon/atom.rs` —
it is a **holon** helper living in this range by proximity. `grep -c "fn require_bundle"
src/runtime.rs` must still be **1** when you finish. Giving it a home is `src/holon/`'s business.

**⛔ STOP-2 — IMPORT FROM THE CANONICAL HOME, NEVER THROUGH `runtime`'s FACADE.**
`src/runtime.rs:759-784` re-exports 22 `crate::value` names, so `use crate::runtime::SymbolTable`
**compiles and is a lie**. Import from `crate::value::`, `crate::ast`, `crate::span`. A type
genuinely unreachable otherwise is a **finding to report**, not a licence to use the facade. Your
report must include each new file's `use` block.

**STOP-3 — move by the FUNCTION LIST, never by line span.** The DESIGN explains why this is no
longer merely prudent: partire's original ranges are stale by ~3,506 lines after the declare stone,
and the numbers in §1 were re-derived from the current file. If a function you are about to move is
not in §1's list, STOP.

**STOP-4 — `src/reflect/` must not reference `crate::intrinsic`.** The impl does not know its edge.

**STOP-5 — verbatim means verbatim.** No signature tidying. A visibility bump forced by the new
module boundary is expected — report each one. If a body cannot move unchanged, STOP and report what
forced it.

**STOP-6 — watch the doc blocks.** The declare rider mis-copied one function's start line as its
`pub fn` line and orphaned a 42-line `///` block in `runtime.rs`. It caught this with a systematic
post-edit scan for `///` blocks left sitting above a retirement comment. **Run that scan.**

## Report

Per-file diff summary; the files you created and what went in each; **each new file's `use` block
verbatim**; confirmation `require_bundle` is still in `runtime.rs`; whether the role grouping held
against the bodies or had to change; before/after `wc -l src/runtime.rs`; the result of the orphaned-
doc-block scan; and what surprised you.
