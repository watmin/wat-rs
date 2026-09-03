//! Special-form doc entry for `:wat::core::do` — arc 255 Stone 1a-zeta, the last three of the
//! special-form table. Doc-only: `eval_do` (`src/runtime.rs`), `eval_do_tail` (`src/runtime.rs`)
//! and `infer_do` (`src/check.rs`) already fit their canonical `NativeHandler`/`TailHandler`/
//! check shapes exactly, so all three `#[wat_special_form_impl]` annotations live in place
//! (mirrors `control_flow.rs`'s `if`, not `quote.rs`'s thin-delegate shape).

use wat_macros::wat_special_form;

/// Evaluate `<f1> <f2> ... <fN>` in sequence; the value of every non-final form is discarded
/// (kept only for its side effect), and the FINAL form's value is the `do` form's value. Empty
/// arg list is a `MalformedForm`. Unlike `if`/`and`, which may skip a branch or short-circuit,
/// `do` evaluates every one of its sub-forms, always, in textual order — the shape this row's
/// axes follow.
///
/// **Category ground —** `eval_do` (`src/runtime.rs:4160`) sequences the evaluation of N forms
/// into one evaluation event, deciding what runs, in what order, and which one's resulting
/// value survives — the same "directs evaluation" prose `if`'s own `ControlFlow` ruling argues
/// (`control_flow.rs`). The tail door's own retirement comments (`runtime.rs:987`) already name
/// `do` textually alongside `if`/`let`/`match`/`and`/`or` as the same family's sixth member
/// ("`if`/`let`/`match` are the only three registered tail impls as of this stone; every other
/// head (`do`/`and`/`or`/...) has no registry row"). `ControlFlow`.
///
/// **Purity ground —** measured directly: `eval_do` calls `eval_inner` on every element of
/// `args` — the non-final ones for effect (discarded), the final one for its value — never
/// skipping any (unlike `and`'s short-circuit or `if`'s untaken branch). `do` adds no effect of
/// its own; it is pure exactly when every sub-form it runs is, the same sentence
/// `Purity::Preserving` was minted with for `if` (`control_flow.rs`) and `and` (`and_form.rs`).
/// `Preserving`.
///
/// **Determinism ground —** `eval_do` walks `args` in a fixed textual order, calling
/// `eval_inner` sequentially; it consults no clock, no entropy, no hygiene-tagging counter of
/// its own (that mechanism lives only in the macro-expansion walker, a different function
/// entirely — see `quasiquote.rs`'s identical carve-out). The same sequence of sub-forms, run in
/// the same order, always produces the same result if each sub-form does. `Preserving`.
///
/// **Totality ground —** `eval_do`'s only fallible path of its own is the `args.is_empty()`
/// guard (`MalformedForm`) — a fixed-shape check `infer_do` (`src/check.rs`) already refuses
/// earlier with the identical guard, the same "malformed signature is outside totality's
/// domain" carve-out `if`'s and `quote`'s own `Total`/`Preserving` grounds use. Past that guard,
/// `do` is total exactly when every sub-form it evaluates — ALL of them, non-final and final
/// alike — is total. `Preserving`.
///
/// **Expand-time ground —** `src/macros/eval.rs`'s `is_expand_time_legal` residue hand-list
/// names `":wat::core::do"` literally (its "value/control-flow ops with no per-verb home yet"
/// group), so `is_expand_time_legal(":wat::core::do")` currently returns `true` via that list
/// (pre-registration; `lookup_entry` returns `None`). `validate_pure_total` (`macros/eval.rs`)
/// recurses into every argument of any head for which `is_expand_time_legal` is true (skipping
/// only the specially-cased `quote`/`quasiquote` heads) — so `do`'s admission inside a macro
/// body is already, in the running code, conditioned on its sub-forms' own admission, the exact
/// shape `and`'s own `ExpandTime::Preserving` ground argues (`and_form.rs`): `do` evaluates real
/// sub-forms at its own call site (all of them, not merely some), so its own expand-time
/// legality genuinely depends on theirs. `Preserving`.
///
/// @added 1.0.0
/// @Category ControlFlow
/// @Purity Preserving
/// @Determinism Preserving
/// @Totality Preserving
/// @ExpandTime Preserving
/// @syntax (:wat::core::do <form>+)
/// @ret :T the last form's value; every earlier form is evaluated and discarded
/// @example (:wat::core::do 1 2 3) #=> 3
#[wat_special_form(":wat::core::do")]
pub(crate) struct Do;
