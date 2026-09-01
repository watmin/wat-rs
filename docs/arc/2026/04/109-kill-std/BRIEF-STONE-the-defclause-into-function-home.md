# BRIEF — STONE: `defclause` dispatch joins `src/function/`

Move 12 items (~1,637 lines) out of `runtime.rs` into `src/function/`'s **existing** files, by role.
DESIGN: `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-defclause-into-function-home.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `./target/release/wat` and `--check` for a fast read. **You may not
spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not
commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at 5114 —
anything that breaks is yours.

## Read in order

1. The DESIGN above — why this mints no new home.
2. **`src/function/`** — all five files. `mod.rs`'s opening line says the home was minted for the
   fn-form; `parse.rs` holds `parse_fn_signature*`, `eval.rs` holds `eval_fn`. You are filling this
   out, not starting it.
3. **`src/reflect/`** — the stone that shipped immediately before, same discipline. Read its `use`
   blocks; that is the standard.
4. `src/value/environment.rs:148` · `src/value/symbol_table.rs:32` — where `Environment` and
   `SymbolTable` live. Read STOP-2 before writing a single `use`.

## The work

### 1 — move 12 items into `src/function/`, by role

**into the existing `src/function/parse.rs`**
`parse_defclause_clause` 3627 · `mod arc109_two_iii_defclause_return_slot` 3908 (a `#[cfg(test)]`
mod — it is `parse_defclause_clause`'s own probe and travels with it) · `parse_defclause_form` 3980 ·
`parse_extend_type_form` 4228 · `parse_derive_form` 4566 · `is_defclause_form` 4632

**into the existing `src/function/eval.rs`**
`eval_call_to_defclause` 4645 · `select_defclause_clause` 4675 ·
`eval_call_to_defclause_with_vals` 4888

**into a NEW `src/function/subsume.rs`**
`declared_type_subsumes` 4998 · `value_matches_type_by_name` 5031 · `val_type_path` 5170

⚠ **Verify that third grouping against the bodies.** These are RUNTIME type-matching for clause
selection, not check-time inference — which is why they are not `infer.rs`'s. If reading them says
otherwise, say so; the previous stone's `verbs.rs` shipped 12 of 13 because exactly this kind of
assignment was wrong.

### 2 — the call sites

**9 external, in 2 files**: `src/check.rs` (5) · `src/declare/register.rs` (4). The compiler names
them; fix what it names.

★ `declare/register.rs`'s four are `crate::runtime::{parse_defclause_form, parse_extend_type_form}` —
imports the previous stone had to make because these had no home. After this stone they point at
`crate::function::parse`, and `declare`'s dependency on `runtime` shrinks. **Report the before/after
count of `crate::runtime::` in `src/declare/register.rs`.**

Leave a short retirement comment at each cut, in the shape `src/reflect/`'s stone used.

## Blast radius

`src/function/{parse,eval}.rs` (existing) · `src/function/subsume.rs` (new) · `src/function/mod.rs`
(one `mod`) · `src/runtime.rs` (12 items out) · `src/check.rs` · `src/declare/register.rs`. No `.wat`
corpus change. No registrations. **No defclause call selects a different clause.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — `eval_let` (5264) IS THE BOUNDARY AND MUST NOT MOVE.** Nor `bind_let_binding` (5338)
nor `eval_do` (5550). They sit immediately after `val_type_path` and are the **eval spine**, adjacent
by line only. `grep -c "fn eval_let" src/runtime.rs` must still be **1**. This trap has appeared in
every stone of this campaign; §1's list is the authority.

**⛔ STOP-2 — IMPORT FROM THE CANONICAL HOME, NEVER THROUGH `runtime`'s FACADE.**
`src/runtime.rs:759-784` re-exports 22 `crate::value` names, so `use crate::runtime::SymbolTable`
**compiles and is a lie**. Import from `crate::value::`, `crate::ast`, `crate::span`. Report each
touched file's `use` block.

**STOP-3 — no `defclause.rs`.** The items go into `parse.rs` and `eval.rs` **beside their
same-role neighbours**. A file grouping all twelve by the form they came from re-creates, inside the
home, the split the home exists to prevent.

**STOP-4 — `src/function/` must not reference `crate::intrinsic`.** The impl does not know its edge.
⚠ The previous stone found a function whose body held 17 references to the registry and correctly
refused to move it. If one of these twelve does the same, **STOP and report it** — that is a finding
about the DESIGN's list, not a problem to route around.

**STOP-5 — verbatim.** No signature tidying. Visibility bumps forced by the new boundary are
expected — on the moving side AND on functions that stay (the last stone found one of each). Report
every one.

**STOP-6 — run the orphaned-doc-block scan.** After editing, scan `runtime.rs` for any `///` block
left sitting above a retirement comment. Two stones ago that scan caught a 42-line doc block stranded
by a mis-copied start line. Its result is a required report line.

## Report

Per-file diff summary; what landed in each file; **each touched file's `use` block verbatim**;
confirmation `eval_let` is still in `runtime.rs`; whether the `subsume` grouping held against the
bodies; the before/after `crate::runtime::` count in `src/declare/register.rs`; before/after
`wc -l src/runtime.rs`; the doc-block scan result; and what surprised you.
