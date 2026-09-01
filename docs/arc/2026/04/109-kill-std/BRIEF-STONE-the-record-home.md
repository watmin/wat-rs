# BRIEF — STONE: `src/record/` — the aggregate family gets its home

Move 17 items (~1,087 lines) out of `runtime.rs` into a new `src/record/`, split by role. DESIGN:
`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-record-home.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `./target/release/wat` and `--check` for a fast read. **You may not
spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not
commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at 5114 —
anything that breaks is yours.

## Read in order

1. The DESIGN above.
2. **`src/intrinsic/record.rs`** — the EDGE this home serves. Its line 12 says *"all seven now
   `pub(crate)` so this module can reach them — no body [moves]"*: the visibility bump was made in
   anticipation of the home you are building. **Implementations do not go in it.**
3. **`src/reflect/` and `src/declare/`** — the two most recent homes of this shape. Read their files
   and `use` blocks; that is the standard.
4. `src/value/environment.rs:148` · `src/value/symbol_table.rs:32` — where `Environment` and
   `SymbolTable` live. Read STOP-2 before writing a single `use`.

## The work

### 1 — create `src/record/`, split by ROLE

Declare `pub(crate) mod record;` in `src/lib.rs`. Suggested grouping — **verify against the bodies
and report if it does not hold**:

**construct.rs** — `eval_struct_new` 7370 · `eval_variant` 7458 · `eval_aggregate_new` 9086 ·
`construct_aggregate` 9131 · `eval_kwargs_construct` 9241
**access.rs** — `eval_struct_field` 7692 · `eval_record_field_at` 9557 · `eval_record_q` 9635 ·
`eval_list_q` 9670
**project.rs** — `project_surface_attrs` 9415 · `parse_projection_args` 9459 · `eval_to_core_record` 9528
**update.rs** — `record_field_map` 9706 · `eval_record_to_map` 9774 · `eval_record_same_data` 9809 ·
`record_assoc_inner` 9851 · `eval_record_assoc` 10029

⚠ **Private helpers land beside the verb they serve, not in a `helpers.rs`.** `construct_aggregate`
with `eval_aggregate_new`; `record_assoc_inner` with `eval_record_assoc`; `parse_projection_args`
with `project_surface_attrs`. A helper's reason to change is its verb's.

### 2 — re-point 20 call sites in 2 files

`src/intrinsic/record.rs` 18 (the edge — every delegate) · `src/intrinsic/holon/atom.rs` 2.
Everything else is `runtime.rs`'s own dispatch and in-file callers. The compiler names them.

Leave a short retirement comment at each cut, in the shape the previous stones used.

## Blast radius

`src/record/` (new) · `src/lib.rs` (one `mod`) · `src/runtime.rs` (17 items out) ·
`src/intrinsic/record.rs` · `src/intrinsic/holon/atom.rs`. No `.wat` corpus change. No registrations.
**No aggregate verb changes behaviour.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — `eval_retag_op` (7584) MUST NOT MOVE.** It sits **between** `eval_variant` (7458) and
`eval_struct_field` (7692), and it reads like a record verb — it retags a variant. Its **sole caller
is `src/intrinsic/kernel/serve.rs`**. `grep -c "fn eval_retag_op" src/runtime.rs` must still be
**1**. This is the eighth intruder found inside a proposed module in this campaign; §1's list is the
authority, not the span.

**⛔ STOP-2 — IMPORT FROM THE CANONICAL HOME, NEVER THROUGH `runtime`'s FACADE.**
`src/runtime.rs:759-784` re-exports 22 `crate::value` names, so `use crate::runtime::SymbolTable`
**compiles and is a lie**. Import from `crate::value::`, `crate::ast`, `crate::span`. A type
genuinely unreachable otherwise is a **finding to report**. Your report must include each new file's
`use` block.

**STOP-3 — no `helpers.rs`, no `util.rs`, no `common.rs`.** If a grouping leaves an item you cannot
place by its verb, STOP and report which and why — that is a finding about the DESIGN's role split,
not a bucket to open.

**STOP-4 — `src/record/` must not reference `crate::intrinsic`.** The impl does not know its edge.
⚠ Two stones ago a rider found a function whose body held 17 registry references and correctly
refused to move it. If one of these seventeen does the same, **STOP and report it** — a finding about
the list, not a problem to route around.

**STOP-5 — verbatim.** No signature tidying. Visibility changes forced by the new boundary are
expected, on the moving side and on functions that stay; report each.

**STOP-6 — run the orphaned-doc-block scan** over the whole of `runtime.rs` after editing: any `///`
block left stranded above a retirement comment. Its result is a required report line. ⚠ This move has
**two clusters** with ~1,300 lines of unrelated code between them, so there are more cut boundaries
than usual for a stone this size.

## Report

Per-file diff summary; what landed in each file; **each new file's `use` block verbatim**; whether
the role grouping held against the bodies or had to change, and which callers decided any
re-assignment; confirmation `eval_retag_op` is still in `runtime.rs`; before/after
`wc -l src/runtime.rs`; the doc-block scan result; and what surprised you.
