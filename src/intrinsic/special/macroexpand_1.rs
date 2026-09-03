//! Special-form doc entry for `:wat::core::macroexpand-1` — arc 255 Stone 1a-gamma-i,
//! `:wat::core::macroexpand`'s sibling (`macroexpand.rs`, one expansion step vs. fixpoint). See
//! that file's module doc for the full STOP-1 finding (measured NOT `Effectful` — the fenced
//! `macro_eval` pure-total evaluator structurally denies any leaked effect) and the measured
//! `Nondeterministic` finding (hygiene scope-tagging, the same mechanism `fresh-symbol` already
//! carries). Every ground below is that row's, restated for this FQDN because `render-doc` reads
//! per-entry, not per-argument-shared-fn — the same restatement discipline
//! `config_set_eval_redef.rs` uses for its own sibling row.
//!
//! `role = check` is `macroexpand.rs`'s `infer_macroexpand`, stacked with a second
//! `#[wat_special_form_impl]` attribute there (legal — `role = check` emits source only, no
//! shim). `role = eval` is its OWN separate fn, `src/reflect/expand.rs`'s `eval_macroexpand_1`
//! (annotated in place) — NOT stacked on `eval_macroexpand`'s fn, because `role = eval` codegens
//! a dispatch shim NAMED FROM THE FN IDENTIFIER ALONE
//! (`[[NOTE-role-eval-cannot-stack-and-the-error-does-not-say-so]]`), and the two are already
//! separate fns in `src/reflect/expand.rs` from birth (one expansion step vs. fixpoint are
//! genuinely different bodies, not a shared no-op) — so this landmine never bites here.

use wat_macros::wat_special_form;

/// One macro-expansion step: if `<form>` is a macro call (a list whose head is a registered
/// macro keyword), apply that macro's template once and return the result; otherwise return
/// `<form>` unchanged. Does NOT recurse into children and does NOT fixpoint — matches Common
/// Lisp / Clojure `macroexpand-1`. Arc 030.
///
/// **Category ground —** identical to `:wat::core::macroexpand`'s row: `Reflection`.
///
/// **Purity ground —** identical reasoning to `:wat::core::macroexpand`'s row:
/// `eval_macroexpand_1` (`src/reflect/expand.rs:33`) evaluates its ONE argument via ordinary
/// call-by-value, then calls `expand_once` exactly once — whose only path to the real evaluator
/// is the SAME fenced `macro_eval` pure-total gate `macroexpand`'s row measures. `Pure`.
///
/// **Determinism ground —** identical mechanism, one step instead of a fixpoint loop: a macro
/// whose template introduces a local identifier gets a fresh `ScopeId` from the SAME
/// process-global counter on every call. `Nondeterministic`.
///
/// **Totality ground —** identical: `eval_macroexpand_1`'s own body raises
/// `MacroExpansionFailed` when the single `expand_once` call fails (unknown macro name resolved
/// at runtime, wrong arity for the named macro, template-body errors) — a real, non-arity
/// failure mode a static check cannot rule out. `Partial`.
///
/// **Expand-time ground —** identical: `":wat::core::macroexpand-1"` is not in
/// `is_expand_time_legal`'s allow-list either (measured by grep — zero hits). `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Reflection
/// @Purity Pure
/// @Determinism Nondeterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::core::macroexpand-1 <form>)
/// @ret :wat::WatAST `<form>` if it names a registered macro call, expanded ONE step; `<form>` unchanged otherwise
/// @example-norun (:wat::core::macroexpand-1 (:wat::core::quote (:some::macro-with-a-template-local-binder arg))) #=> the one-step-expanded form, with any template-introduced identifier carrying a freshly-minted `ScopeId` on every call — never guaranteed equal to a prior call's result for the same input
#[wat_special_form(":wat::core::macroexpand-1")]
pub(crate) struct MacroexpandOne;
