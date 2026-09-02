//! Special-form doc entry for `:wat::core::def` — arc 255 Stone 1a-β-ii, the last of the
//! three names still missing from `freeze::is_liftable_declaration_head`'s registry mirror,
//! and the first row in the family with a check impl of its own
//! (`DESIGN-STONE-1a-beta-ii-the-last-three-and-the-first-hand-list-dies.md`).

use wat_macros::wat_special_form;

/// Bind `:name` to the value `expr` evaluates to — or, with a `{...}` metadata-map second
/// arg, `(:wat::core::def :name {meta} expr)` — registering the binding into the enclosing
/// `SymbolTable`/`CheckEnv`, visible to every form after it in the file. `def` is the
/// primitive every fn-shape declaration lowers into: `(:wat::core::defn :name [args] ->
/// :Ret body)` expands to `(:wat::core::def :name (:wat::core::fn [args] -> :Ret body))`
/// before this form is ever reached.
///
/// **Category ground —** same as `defsurface`'s: `def` registers `:name` into
/// `sym.functions`/`env.defined_values` — visible to every form after it in the file, not
/// scoped to a body — exactly `Declaration`'s own variant prose ("registers a program-level
/// entity … visible to everything after it"). `Declaration`.
///
/// **Purity ground — ★★★ the one contract decision this stone turns on.** Unlike
/// `defmacro`/`defalias`/`typealias`/`structtype`, `def` DOES have an arm in `runtime.rs`
/// (`:2132`) — but it is a REFUSAL, not an implementation:
/// `":wat::core::def" => Err(RuntimeError::new(…, RuntimeErrorKind::
/// DeclarationInExpressionPosition(…)))`. It answers "this form cannot be evaluated here,"
/// never "here is `def`'s value." `role = eval` means "here is the code that evaluates this
/// form" — annotating the refusal would make `show-source :wat::core::def` present an
/// error-raiser as the form's evaluator, a lie about the substrate
/// (`DESIGN-STONE-1a-beta-ii-the-last-three-and-the-first-hand-list-dies.md`'s ★★★). With no
/// `Eval` role and no handler, all four consumers of `@Purity` ask a RUNTIME question `def`
/// has no runtime to answer: `Pure` would demand a runnable `@example` of a verb that cannot
/// be run; `Effectful` would claim an effect there is no call to have — and would also
/// re-open `effectful_by_prefix`'s census, since `:wat::core::` is not one of its eight
/// prefixes, the exact red `Purity::Unevaluated` was minted to close; `Preserving` would
/// claim sub-forms that are never evaluated BY `def` ITSELF — the bound expression is
/// registered by `register_defines`/`infer_def`, not run, at this form's own processing.
/// The refusal arm is the ENFORCEMENT of `Unevaluated`, not a counterexample to it.
/// `Unevaluated`.
///
/// **Determinism ground —** the same `def` form, registered against the same preceding
/// declarations, always binds the identical name to the identical (unevaluated, at this
/// point) expression — no clock, no entropy, no gensym anywhere on
/// `register_defines`/`try_parse_fn_shape_def`'s path. `Deterministic`.
///
/// **Totality ground —** `infer_def` (`def`'s own check impl, `check.rs:7978`) is measured
/// NOT defined on every input: a malformed arg count raises `CheckErrorKind::MalformedForm`,
/// and a redefinition collision raises `CheckErrorKind::DefRedefForbidden` — both propagated
/// as hard diagnostics, never a value a caller matches on. Same reasoning `:wat::i64::/`'s
/// own `@Totality Partial` was ruled on
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`). `Partial`.
///
/// **Expand-time ground —** `def` has no runtime call site at all (`role = declare` emits no
/// shim; the refusal arm at eval time is not a call site either — see the Purity ground
/// above) — a well-formed form is consumed at registration, strictly AFTER `expand_all`
/// completes (`src/freeze.rs`'s pipeline doc), and `is_liftable_declaration_head`'s own doc
/// records why `defn` never reaches this predicate: it macro-expands to `def` BEFORE
/// `extract_closure` runs. `def` is absent from `macros/eval.rs`'s expand-time pure-total
/// allow-list (measured — no `:wat::core::def` arm there), so a `def` nested inside a macro
/// body cannot be eagerly evaluated during expansion — the identical fact `defsurface`'s row
/// measured for `synthesize_surface_protocol`'s `env` dependency. `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Declaration
/// @Purity Unevaluated
/// @Determinism Deterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::core::def :name expr)
/// @ret :wat::core::nil no runtime value in declaration position — the form is consumed at registration and never reaches evaluation there; encountered in expression position it raises `DeclarationInExpressionPosition` instead of producing one
/// @example-norun (:wat::core::def :probe::x 5) #=> registers :probe::x into the symbol table; no runtime value
#[wat_special_form(":wat::core::def")]
pub(crate) struct Def;
