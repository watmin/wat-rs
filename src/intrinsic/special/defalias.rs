//! Special-form doc entry for `:wat::core::defalias` — arc 255 Stone 1a-β-ii, the last of the
//! last three names in `freeze::is_liftable_declaration_head`'s domain to register.

use wat_macros::wat_special_form;

/// Declare `:alias-name` as an alias for `:target-name` — a user-defined function or a
/// substrate primitive. `register_defalias` synthesizes a delegating `Function` whose
/// signature copies the target's (params, type params, return type) and whose body calls
/// `(target p0 p1 ... & rest)`, then registers it under `:alias-name` — the same registered
/// shape a hand-written wrapper `defn` would produce, minted instead of typed out.
///
/// **Category ground —** same as `defsurface`'s: `defalias` registers `:alias-name` into
/// `sym.functions` — visible to every form after it in the file, not scoped to a body —
/// exactly `Declaration`'s own variant prose ("registers a program-level entity … visible
/// to everything after it"). `Declaration`.
///
/// **Purity ground —** measured directly: `:wat::core::defalias` appears NOWHERE in
/// `src/runtime.rs` — not in `dispatch_keyword_head_value`, not in `eval_tail`, not in
/// `step_list`, and not even in the `is_mutation_head`/`is_mutation_form` hand-lists its
/// siblings (`defmacro`/`structtype`/`defenum`/…) still carry. No `handler`, no eval arm,
/// no tail arm, no mutation-guard mention at all. Same reasoning as `defsurface`'s row: all
/// four consumers of `@Purity` ask a RUNTIME question, and `defalias` has no runtime to ask
/// it about — `Pure` would demand a runnable `@example` of a verb that cannot be run,
/// `Effectful` would claim an effect there is no call to have, `Preserving` would claim
/// sub-forms that are never evaluated (`:alias-name`/`:target-name` are keywords, read once,
/// never evaluated). `Unevaluated`.
///
/// **Determinism ground —** the same `defalias` form, registered against the same preceding
/// declarations (the target's shape, resolved from `sym`/`CheckEnv::with_builtins_and_types`
/// at the moment of registration), always produces the identical delegating `Function` — no
/// clock, no entropy, no gensym anywhere on `parse_defalias_form`/`register_defalias`'s
/// path (the synthesized param names `_p0`, `_p1`, … are positional, not gensym'd).
/// `Deterministic`.
///
/// **Totality ground —** `parse_defalias_form` alone is a total shape-recognizer (`Option`,
/// never a raise) — but declare-time processing of a MATCHED `defalias` form is not: once
/// matched, `register_defalias` runs `crate::resolve::register`, whose gate can answer
/// `Reserved`/`Duplicate`/`Unnamespaced`/`DottedName` — measured directly:
/// `(:wat::core::defalias :wat::core::my-alias :wat::core::length)` from user privilege
/// raises `RuntimeErrorKind::ReservedPrefix`, a raise the freeze pipeline propagates as a
/// hard failure, never a value a caller matches on. Same reasoning `:wat::i64::/`'s own
/// `@Totality Partial` was ruled on
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`). `Partial`.
///
/// **Expand-time ground —** `defalias` has no runtime call site at all (`role = declare`
/// emits no shim) — a well-formed form is consumed at registration
/// (`register_defines`/`register_defalias`), strictly AFTER `expand_all` completes
/// (`src/freeze.rs`'s pipeline doc; `src/freeze/env.rs`'s `build_env` doc, step 4 → step 6),
/// state that categorically does not exist while a `defmacro` body is being expanded.
/// `defalias` is also absent from `macros/eval.rs`'s expand-time pure-total allow-list
/// (measured — no `:wat::core::defalias` arm there), so a `defalias` nested inside a macro
/// body cannot be eagerly evaluated during expansion — the identical fact `defsurface`'s
/// row measured for `synthesize_surface_protocol`'s `env` dependency. `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Declaration
/// @Purity Unevaluated
/// @Determinism Deterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::core::defalias :alias-name :target-name)
/// @ret :wat::core::nil no runtime value — the form is consumed entirely at registration time and never reaches evaluation; its effect is the delegating Function it leaves in the symbol table
/// @example-norun (:wat::core::defalias :probe::size :wat::core::length) #=> registers :probe::size as a delegating alias for :wat::core::length; no runtime value
#[wat_special_form(":wat::core::defalias")]
pub(crate) struct Defalias;
