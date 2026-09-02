# BRIEF — STONE: the eval door

Give `role = eval` a callable pointer, so the four registered special forms dispatch through the
registry and their four literal arms die. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-every-role-carries-its-pointer.md` — read its
**AMENDED** section first; it records the probe that reordered these stones.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5119, HEAD `f3ea2b992`.

⚠ **This stone changes a proc macro.** You cannot compile it. Write it carefully, mirror the
existing codegen exactly, and report anything you are unsure of rather than guessing — the
orchestrator meets the compiler.

## Read in order

1. The DESIGN's § "AMENDED" and § "The mechanism — the macro already holds what it needs".
2. **`crates/wat-macros/src/wat_intrinsic.rs` around its return-type handling** — it accepts a
   handler returning `Result<Value, EvalBreak>` and wraps it `TrackedValue::new(v,
   Provenance::Unknown)`, or passes a `Result<TrackedValue, EvalBreak>` through un-rewrapped, and
   `compile_error!`s on anything else. **Your codegen must reuse that exact logic, not a copy of it
   if it can be shared.**
3. **`crates/wat-macros/src/wat_special_form_impl.rs`** — the attribute you are extending. It already
   parses `role = check|eval|tail` and emits `(name, role, source)`.
4. `src/intrinsic/mod.rs`'s `SpecialFormImplSubmission` and the fold in `registry()` that builds
   `impls`.

## The work

### 1 — the submission carries a pointer for `role = eval`

`SpecialFormImplSubmission` gains an eval handler slot. The macro, **only for `role = eval`**, emits
the annotated fn's pointer alongside the source string it already emits. `role = check` and
`role = tail` keep emitting source only — the tail door is a later stone and this one must not
half-build it.

### 2 — `registry()` folds it into `IntrinsicEntry.handler`

★ Into the **existing** `handler` field, not a new one. Then `registry().lookup(head)` finds it
unchanged, `dispatch_keyword_head_value`'s existing guard dispatches it, and
`registry_first_door_owns_every_handler_row_no_literal_arm_survives` — the gate shipped last stone —
starts requiring the arms to go. **That gate going red is this stone working**, not a break.

### 3 — the signatures

Measured against `NativeHandler = fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<TrackedValue, EvalBreak>`:

```
eval_let    (args, span, env, sym) -> Result<TrackedValue, EvalBreak>   EXACT — pointer as-is
eval_if     (args, span, env, sym) -> Result<Value, EvalBreak>          macro's existing wrap
eval_match  (args, span, env, sym) -> Result<Value, EvalBreak>          macro's existing wrap
crate::function::eval_fn (args, span, env) -> Result<Value, RuntimeError>   ⛔ needs a delegate
```

`eval_fn` is the one that does not fit: **three** parameters and `RuntimeError` instead of
`EvalBreak`. Write a thin delegate beside the `Fn` doc-struct in `src/intrinsic/special/fn_form.rs`
that takes the full four, ignores `sym`, and does `.map_err(Into::into)` — the idiom
`src/intrinsic/kernel/stdio.rs` already uses. Annotate the delegate, not `eval_fn`.
**Do not change `eval_fn`'s signature.**

### 4 — delete the four arms

`:wat::core::fn` (~2048), `:wat::core::if` (~2056), `:wat::core::let` (~2054), `:wat::core::match`
(~2222), all in `dispatch_keyword_head_value`. Arm lines only; leave surrounding commentary and the
handler fns. One-line retirement note at each cut, in the shape the last stone used.

## Blast radius

`crates/wat-macros/src/wat_special_form_impl.rs` · `src/intrinsic/mod.rs` (submission + fold) ·
`src/intrinsic/special/fn_form.rs` (one delegate) · the four `#[wat_special_form_impl(role = eval)]`
sites · `src/runtime.rs` (four arm lines) · whatever the compiler names. No `.wat` corpus change.
**No verb changes behaviour — the same four fns run, reached through the registry instead of a
literal arm.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — DO NOT TOUCH `eval_tail` OR `step_list`.** They are the next two stones. All four forms
keep their arms in both. `grep -c "intrinsic::registry()"` inside each must still be **0**.
⚠ Their arms are ABOVE any future guard, so a half-built door there is the guard-hoist's measured
"looks finished and did nothing" — worse than no door.

**⛔ STOP-2 — `role = tail` AND `role = check` KEEP EMITTING SOURCE ONLY.** Only `eval` gains a
pointer. A tail pointer with no `eval_tail` guard to call it is dead weight that looks like a
feature.

**⛔ STOP-3 — DO NOT CHANGE `eval_fn`'s SIGNATURE.** It has three callers' worth of history and its
own home. Write a delegate. This is the same shape as the `op: &str` refusal two stones ago, where
reshaping was the wrong answer and a wrapper was right.

**⛔ STOP-4 — REUSE THE MACRO'S RETURN-TYPE LOGIC, DO NOT RE-DERIVE IT.** `wat_intrinsic.rs` already
decides `Value` vs `TrackedValue` vs `compile_error!`. A second copy in
`wat_special_form_impl.rs` is two authorities for one question — the shape this arc exists to delete.
If it cannot be shared, say so and say why.

**⛔ STOP-5 — THE DEAD-ARM GATE GOING RED IS EXPECTED.** Once these rows carry `handler: Some`,
`registry_first_door_owns_every_handler_row_no_literal_arm_survives` demands their arms be gone. That
is § 4. Do not weaken the gate; do not exempt the four.

**STOP-6 — you cannot compile a proc macro.** Mirror the existing codegen precisely. Report every
place you were unsure, and every construct you copied rather than invented. **A guess in codegen is
worse than a gap, because it fails at every call site at once.**

## Report

Per-file diff summary; the macro change verbatim; **whether STOP-4's shared logic was reusable and
what you did**; the `fn` delegate verbatim; the four arm deletions; confirmation `eval_tail` and
`step_list` are untouched with their grep counts. Then: **what surprised you** — a signature that did
not match the DESIGN's table, a codegen construct with no precedent to copy, or a fold in `registry()`
that could not take an eval pointer without a second field.
