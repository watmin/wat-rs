//! Special-form doc entry for `:wat::core::quasiquote` — arc 255 Stone 1a-gamma-i, the six
//! homoiconic verbs that really evaluate. Unlike `quote` (`quote.rs`), which evaluates NOTHING,
//! `quasiquote` walks a template and evaluates exactly its `:wat::core::unquote`/
//! `unquote-splicing` sub-forms, substituting the resulting VALUES as literal AST nodes.
//!
//! The `role = eval` pointer is `eval_quasiquote` itself (`src/runtime.rs`), annotated in
//! place — its signature already fits the canonical `NativeHandler` shape, unlike `quote`'s or
//! `forms`' (see those rows' own docs for why THEY need a thin delegate here instead).

use wat_macros::{wat_special_form, wat_special_form_impl};

use crate::ast::WatAST;
use crate::check::{CheckEnv, CheckResult, InferCtx, Subst};
use crate::span::Span;
use crate::types::TypeExpr;
use std::collections::HashMap;

/// Walk `<template>`, substituting the VALUE of every unquoted (`(:wat::core::unquote E)`,
/// `(:wat::core::unquote-splicing E)`) sub-form as a literal AST node, and leaving every other
/// node exactly as written (data, never evaluated) — the mixed data-plus-code shape that makes
/// quasiquote the ergonomic way to build a `:wat::WatAST` template with a few computed holes,
/// rather than `forms`' fully-manual `(vec ...)` assembly. Nested `quasiquote` bumps depth +1 and
/// preserves the wrapper; an `unquote`/`unquote-splicing` fires only at depth 1.
///
/// **Category ground —** identical to `quote`'s row: the program building its own syntax as a
/// first-class value is self-reference at the syntax level — `:Reflection`'s own prose.
/// `Reflection`.
///
/// **Purity ground — measured, `Preserving`, NOT `Pure`:** unlike `struct->form`/`macroexpand`
/// (which evaluate their ONE argument, in full, unconditionally — the ordinary call-by-value
/// shape `write-forms` already covers), `quasiquote`'s template has literal parts that are
/// **never evaluated at all** and unquote sites that **always fire for real** — the SAME
/// selective, structure-dependent evaluation shape `if`'s untaken branch and `and`'s
/// short-circuited tail operands have, and `and_form.rs`'s own ground states the general
/// principle this row inherits: "an ordinary function's arguments are all evaluated before the
/// call ever happens" — `quasiquote` is not that; it is a special form BECAUSE some of what it
/// is handed never runs. An unquote site can embed and fire ARBITRARY effectful code (e.g.
/// `(:wat::core::unquote (:wat::kernel::println "hi"))` really prints), so `quasiquote` adds no
/// effect of its own but is pure exactly when the unquote sites it actually reaches are — the
/// same sentence `Purity::Preserving` was minted with for `if` (`control_flow.rs`). `Preserving`.
///
/// **Determinism ground —** by the identical reasoning: the same template, walked in the same
/// order, always substitutes the same unquote results IF those results are themselves
/// deterministic; `quasiquote` introduces no independent source of variation of its own —
/// measured directly: `walk_quasiquote` (`src/runtime.rs`) never calls `fresh_scope`/`add_scope`
/// (that hygiene-tagging mechanism lives ONLY in the MACRO-EXPANSION walker,
/// `src/macros/expand.rs`'s `walk_template`, a different fn entirely — this is why
/// `macroexpand`/`macroexpand-1`, which DO call `walk_template`, are ruled `Nondeterministic`
/// while this row, which never does, is not). `Preserving`.
///
/// **Totality ground —** `walk_quasiquote`'s own doc names a real, template-structure-dependent
/// failure mode independent of any unquote's own totality: an unquoted value with no canonical
/// AST representation (Struct/Enum/Vec/HashMap/HolonAST) errors at the unquote site. Combined
/// with whatever partiality the unquoted sub-expressions themselves carry, `quasiquote` is total
/// exactly when everything it actually evaluates is — the same sentence `if`'s own
/// `Totality::Preserving` was minted with. `Preserving`.
///
/// **Expand-time ground —** `src/macros/eval.rs`'s `validate_pure_total` special-cases
/// `quasiquote`'s head (never refused at the head), but recurses into its unquote sub-forms via
/// `validate_quasiquote_template` — so a quasiquote form's overall expand-time legality
/// genuinely depends on those sub-forms', the identical conditional-acceptance shape `and`'s own
/// `ExpandTime::Preserving` ruling argues (contrast `quote`, which is refused nothing and
/// recursed into nothing — unconditional `Legal`). `Preserving`.
///
/// @added 1.0.0
/// @Category Reflection
/// @Purity Preserving
/// @Determinism Preserving
/// @Totality Preserving
/// @ExpandTime Preserving
/// @syntax (:wat::core::quasiquote <template>)
/// @ret :wat::WatAST `<template>` with every unquoted sub-form's VALUE substituted as a literal AST node; all other nodes unevaluated
/// @example (:wat::core::write-forms (:wat::core::quasiquote (:foo (:wat::core::unquote (:wat::i64::+ 1 2))))) #=> "(:foo 3)"
#[wat_special_form(":wat::core::quasiquote")]
pub(crate) struct Quasiquote;

/// Arc 255 Stone 1a-gamma-i — the `role = check` pointer for `:wat::core::quasiquote`. The
/// inline arm at `check.rs:4889` stays untouched in its OWN LOGIC (STOP-3: extracted verbatim),
/// moved wholesale to this named fn so the registry's `role = check` annotation names real,
/// reachable code rather than a fn nothing calls — the same wiring `:wat::core::use!` got last
/// stone.
#[wat_special_form_impl(":wat::core::quasiquote", role = check)]
pub(crate) fn infer_quasiquote(
    _head: &str,
    _args: &[WatAST],
    _head_span: &Span, // rune:lint(unused-span) — infallible — no error path: `quasiquote` returns the fixed `:wat::WatAST` type for any template; the body is not type-checked here, so there is no `CheckResult::errs` for a span to locate.
    _env: &CheckEnv,
    _locals: &HashMap<String, TypeExpr>,
    _fresh: &mut InferCtx,
    _subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    // Arc 091 slice 8 — runtime quasiquote returns
    // :wat::WatAST. Body isn't fully type-checked (it's
    // a template); unquoted expressions infer into
    // local context but their types don't constrain the
    // outer result.
    CheckResult::ok(TypeExpr::Path(":wat::WatAST".into()))
}
