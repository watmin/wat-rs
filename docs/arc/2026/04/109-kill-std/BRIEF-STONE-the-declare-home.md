# BRIEF — STONE: `src/declare/` — the load-time declaration pass leaves the megafile

Move 44 functions (3,707 lines) out of `runtime.rs` into `src/declare/`, split by PHASE. DESIGN:
`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-declare-home.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `./target/release/wat` and `--check` for a fast read. **You may not
spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not
commit, push, stash, revert, or `git checkout --` anything. Tree is clean, floor green at 5114 —
anything that breaks is yours.

## Read in order

1. The DESIGN above — the phase split, and the span trap named three times.
2. **`src/numeric/`** — the stone that shipped immediately before this one, same shape, same
   discipline. Read its five files and especially their `use` blocks: that is the STOP-1 standard
   you are held to.
3. `src/collection/` — the older precedent (`mod.rs` + concern files, declared `pub(crate) mod
   collection;` in `src/lib.rs`).
4. `src/value/environment.rs:148` · `src/value/symbol_table.rs:32` — where `Environment` and
   `SymbolTable` **actually live**. Read STOP-1 before writing a single `use`.

## The work

### 1 — create `src/declare/`, split by PHASE

Declare `pub(crate) mod declare;` in `src/lib.rs` beside its siblings.

**register.rs** — 13 fns, 1,916 lines
`register_defclause` 842-906 · `register_defines` 907-1112 · `register_extend_type_surface_impls`
1113-1269 · `register_stdlib_runtime_defs` 1270-1379 · `register_stdlib_defines` 1478-1615 ·
`register_struct_methods` 1648-1706 · `register_aggregate_methods` 1707-2078 ·
`register_enum_methods` 2079-2213 · `register_newtype_methods` 2214-2335 ·
`register_type_predicates` 2336-2440 · `register_runtime_defs` 2609-2699 ·
`register_runtime_defs_form` 2737-2994 · `register_defalias` 3038-3148

**parse.rs** — 15 fns, 918 lines
`parse_declare_acronyms_form` 2441-2530 · `is_runtime_declaration_head` 2700-2708 ·
`is_declaration_head` 2709-2716 · `is_declaration_form` 2717-2736 · `parse_defalias_form` 2995-3037 ·
`is_struct_form` 3177-3190 · `is_enum_form` 3191-3219 · `try_parse_metadata_map` 3497-3531 ·
`try_parse_fn_shape_def` 3532-3715 · `try_parse_variadic_def_fn_form` 3716-3833 ·
`try_parse_user_variadic_def_fn_form` 3834-3995 · `parse_type_keyword` 4131-4183 ·
`parse_type_slot` 4210-4333 · `is_type_arg_shaped` 4363-4372 · `is_type_var_path` 4373-4406

**preregister.rs** — 6 fns, 526 lines
`preregister_stdlib_defclause_stub` 1380-1421 · `preregister_acronyms` 2531-2608 ·
`preregister_struct_accessors_from_form` 3220-3374 · `preregister_enum_constructors_from_form`
3375-3496 · `preregister_fn_defs_in_do` 3996-4069 · `preregister_fn_defs_in_let` 4070-4130

**typevar.rs** — 4 fns, 97 lines
`angle_type_head_in_name` 4334-4344 · `collect_free_type_vars` 4407-4426 ·
`collect_free_type_vars_in` 4427-4438 · `walk_free_type_vars` 4439-4496

**seven helpers whose phase you must MEASURE, not inherit** — `meta_has_doc_axis_key` 1422-1443 ·
`record_binding_metadata` 1444-1477 · `parametric_decl_type` 1616-1636 ·
`restrictions_to_binding_metadata_ast` 1637-1647 · `build_delegate_body` 3149-3176 ·
`resolve_type_slot_args` 4184-4209 · `angle_minted_name_reason` 4345-4362

My reading — **verify it, do not trust it**: the first four serve registration, `resolve_type_slot_args`
serves parsing, `angle_minted_name_reason` serves typevar, `build_delegate_body` I could not place.
**Place each by its CALLERS.** `mod.rs` holds the module doc and the `mod` declarations — it is not
a bucket for functions you could not sort.

### 2 — let the compiler find the call sites

Moving these breaks every caller. **That is the census, and it is free.** 51 external sites across
12 files were measured (heaviest: `src/freeze/env.rs` 17 · `src/check.rs` 13 · `src/edn/render.rs` 5),
plus in-file callers. Fix what the compiler names; do not hunt by hand.

Leave a short retirement comment at each cut in the shape arc 255's stones use.

## Blast radius

`src/declare/` (new, five files) · `src/lib.rs` (one `mod`) · `src/runtime.rs` (44 fns out) · the ~12
files the compiler names. No `.wat` corpus change. No registrations added or removed. **No
declaration form's behaviour changes.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — IMPORT FROM THE CANONICAL HOME, NEVER THROUGH `runtime`'s FACADE.**
`src/runtime.rs:759-784` re-exports 22 names from `crate::value` — `Environment`, `SymbolTable`,
`Value`, `EvalBreak`, `Function`, `TrackedValue`, … So `use crate::runtime::SymbolTable` **compiles
and is a lie**; the type lives in `src/value/symbol_table.rs`. `src/check.rs:56` made exactly that
mistake and it is a measured cause of that home's dependency cycle. Import from `crate::value::`,
`crate::ast`, `crate::span`. A type genuinely unreachable except through `crate::runtime::` is a
**finding to report** — name it — not a licence to use the facade.

**⛔ STOP-2 — MOVE BY THE FUNCTION LIST, NEVER BY LINE SPAN.** Three casts in this campaign have now
put something inside a range that did not belong to the concern — most recently `eval_tail` (line
4497), the evaluator's own tail-call spine, which sits ONE LINE past this range's end. **`eval_tail`
must remain in `runtime.rs`.** If a function you are about to move is not in §1's list, STOP.

**STOP-3 — no per-FORM files.** `src/declare/defn.rs`, `defstruct.rs`, `defenum.rs` would grow one
file per declaration form in a substrate that mints them regularly. The PHASE split is the
deliverable; the relocation is not.

**STOP-4 — `mod.rs` is not a junk drawer.** Every one of the seven helpers gets placed by measuring
its callers. If one is genuinely called from two phases, say so and place it where its *reason to
change* lives — then report the call sites so the choice can be checked.

**STOP-5 — `src/declare/` must not reference `crate::intrinsic`.** The impl does not know its edge.

**STOP-6 — verbatim means verbatim.** No signature tidying. If a body cannot move unchanged, STOP
and report what forced it. A visibility bump on an unmoved sibling is expected and is not a body
change — report each one.

## Report

Per-file diff summary; the five files and what went in each; **each new file's `use` block verbatim**
(STOP-1's evidence — a diffstat cannot show it); where you placed each of the seven helpers **and the
callers that decided it**; confirmation `eval_tail` is still in `runtime.rs`; before/after
`wc -l src/runtime.rs`; and what surprised you — a helper used from two phases, a caller the count
missed, or a function whose phase was not what its name suggested.
