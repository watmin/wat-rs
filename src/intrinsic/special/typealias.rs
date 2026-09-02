//! Special-form doc entry for `:wat::core::typealias` — arc 255 Stone 1a-β-i, one of the
//! type-declaration family that shares `defsurface`'s regime (`SpecialFormRole::Declare`,
//! `@Purity Unevaluated`, arc 255 Stone 1a-β-0/0b).

use wat_macros::wat_special_form;

/// Declare a named alias for a type expression: `:Name`, an optional binder (`:- [T…]`),
/// and a mandatory expression (a keyword, a symbol, or a parametric type form). Unlike
/// `newtype`, an alias is transparent — `is_assignable` treats `:Name` and its aliased
/// expression as interchangeable.
///
/// Processed entirely at FREEZE time, before evaluation exists: `parse_typealias`
/// (`src/types.rs:4422`) parses the form into a `TypeDef::Alias`, which
/// `register_types_impl` registers into the `TypeEnv`. The form itself is consumed whole by
/// that pass; it is never spliced into the remaining (evaluated) form stream, so it has no
/// calling convention and no runtime call site to point at — `role = declare` is a
/// REFLECTION fact, "this is the code that processes this form," not a dispatch door,
/// exactly as `defsurface`'s row argues
/// (`DESIGN-STONE-1a-beta-0-the-third-regime-gets-its-name.md`).
///
/// **Category ground —** same as `defsurface`'s: `typealias` registers `:Name` into the type
/// registry — visible to every form after it in the file, not scoped to a body — exactly
/// `Declaration`'s own variant prose ("registers a program-level entity … visible to
/// everything after it"). `Declaration`.
///
/// **Purity ground —** measured directly: `:wat::core::typealias` appears in
/// `src/runtime.rs` exactly ONCE, inside `is_mutation_head` — a hand-list, not a dispatch
/// arm — and nowhere in `dispatch_keyword_head_value`, `eval_tail`, or `step_list`. No
/// `handler`, no eval arm, no tail arm. Same reasoning as `defsurface`'s row
/// (`DESIGN-STONE-1a-beta-0b-a-form-that-never-evaluates.md`): all four consumers of
/// `@Purity` ask a RUNTIME question, and `typealias` has no runtime to ask it about —
/// `Pure` would demand a runnable `@example` of a verb that cannot be run, `Effectful` would
/// claim an effect there is no call to have, `Preserving` would claim sub-forms that are
/// never evaluated (the expression keyword is parsed once, never evaluated).
/// `Unevaluated`.
///
/// **Determinism ground —** the same `typealias` form, parsed against the same preceding
/// declarations, always registers the identical `TypeDef` — no clock, no entropy, no gensym
/// anywhere on `parse_typealias`'s path. `Deterministic`.
///
/// **Totality ground —** `parse_typealias` is measured NOT defined on every input: a
/// missing name or alias-expression keyword, a trailing extra arg, or an expression that is
/// not a keyword/symbol/list/vector all raise `TypeError::MalformedDecl` instead of
/// returning a `TypeDef` — a raise the freeze pipeline propagates as a hard failure, never a
/// value a caller matches on. Same reasoning `:wat::i64::/`'s own `@Totality Partial` was
/// ruled on (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`). `Partial`.
///
/// **Expand-time ground —** `typealias` has no runtime call site at all (`role = declare`
/// emits no shim) — the form is consumed whole at freeze and never reaches evaluation.
/// `parse_type_decl` (its router) is invoked only from `register_types_impl`, which the
/// freeze pipeline's own phase order runs strictly AFTER `expand_all` completes
/// (`src/freeze.rs`'s pipeline doc, step 4 → step 5) — state that categorically does not
/// exist while a `defmacro` body is being expanded. Same fact `defsurface`'s row measured
/// for `synthesize_surface_protocol`'s `env` dependency; `parse_typealias` reaches the
/// identical pipeline position even though its own signature takes no `env` (the aliased
/// expression is stored as an unresolved `TypeExpr`, validated against `env` later in the
/// pipeline, not at parse time). `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Declaration
/// @Purity Unevaluated
/// @Determinism Deterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::core::typealias :Name :Expr)
/// @ret :wat::core::nil no runtime value — the form is consumed entirely at freeze time and never reaches evaluation; its effect is the registration it leaves in the type registry
/// @example-norun (:wat::core::typealias :probe::Tags (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])) #=> registers :probe::Tags into the type registry; no runtime value
#[wat_special_form(":wat::core::typealias")]
pub(crate) struct Typealias;
