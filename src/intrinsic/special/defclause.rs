//! Special-form doc entry for `:wat::core::defclause` — arc 255 Stone 1c-d, one of "the
//! declaration three" (`extend-type` / `derive` / `defclause`): a `:wat::core::*` form with a
//! declare-time parser and a checker arm but, until this stone, no registry row at all.
//!
//! ★★★ Registering this row RESTORES a named refusal a prior stone lost, by construction, not
//! by an arm. `SEAM:214` (the hand-rolled-arms-retire stone) recorded: *"`defclause` lost its
//! named refusal this session; it has no registry row. Register it."* — `def` had a row and
//! kept its `DeclarationInExpressionPosition` refusal when its hand-rolled `runtime.rs` arm was
//! deleted; `defclause` had none, so the SAME deletion left it falling through to the generic
//! `UnknownFunction` fallback (`runtime.rs:2250` records the loss at the site). The moment this
//! row exists with `@Purity Unevaluated`, `dispatch_keyword_head`/`dispatch_keyword_head_value`'s
//! registry-first `Unevaluated`-keyed guard (`runtime.rs:1951`/`:2086`) answers
//! `DeclarationInExpressionPosition` for `:wat::core::defclause` again — see this stone's report
//! for the probe that verifies it live.

use wat_macros::wat_special_form;

/// Declare a named, possibly-multi-clause function: `(:wat::core::defclause :name [-> :T]
/// ([args] body) ...)` (Option A, one shared return type) or `(:wat::core::defclause :name
/// ([args] -> :T body) ...)` (Option B, per-clause return types) — the primitive `defn` itself
/// is sugar over for the single-clause case. Each clause may carry `:guard`/`:ensure` metadata.
/// `register_defclause_from_form`/`preregister_defclause_in_env` (`src/check.rs`) collect every
/// `defclause` into `env.defclause_registrations` / the runtime `ClauseSet` table, visible to
/// every call site after it in the file.
///
/// **Category ground — ★★★ the strongest of the three, because it is not an inference from
/// prose, it is a citation.** `:Declaration`'s own defining comment in `wat/runtime-meta.wat`
/// (the SOURCE OF TRUTH the Rust enum is generated from) reads: *"registers a program-level
/// entity (def, defclause, declare-acronyms). Distinct from `:Binding` — a declaration
/// registers into the program, visible to everything after it."* `defclause` is named BY THE
/// ENUM ITSELF as a `:Declaration` exemplar, alongside `def` — no reading-what-it-actually-does
/// exercise is needed here the way `extend-type`/`derive` required one (their own rows argue it
/// from measurement instead, since the enum's prose does not name them). `Declaration`.
///
/// **Purity ground —** measured directly, same method as `def`/`defmacro`/`defalias`'s own
/// rows: `:wat::core::defclause` has NO entry in `dispatch_keyword_head`/
/// `dispatch_keyword_head_value`'s match (the hand-rolled arm the-hand-rolled-arms-retire stone
/// deleted was never replaced — `runtime.rs:2250`'s own retirement comment records this), and no
/// `NativeHandler`. All four consumers of `@Purity` ask a RUNTIME question `defclause` has no
/// runtime to answer: `Pure` would demand a runnable `@example` of a verb that cannot be run;
/// `Effectful` would claim an effect there is no call to have; `Preserving` would claim
/// sub-forms `defclause` itself evaluates — the clause bodies are stored whole into the
/// `ClauseSet`, type-checked by `infer_defclause` but not *evaluated* by this form, only later,
/// once, per call, by the registered `Function`. `Unevaluated`.
///
/// **Determinism ground —** the same `defclause` form, parsed against the same preceding
/// declarations, always produces an identical `ClauseSet` (name, clauses, shared return,
/// metadata) — no clock, no entropy, no gensym anywhere on `parse_defclause_form`'s path.
/// `Deterministic`.
///
/// **Totality ground —** `parse_defclause_form` is measured NOT defined on every input: a
/// non-list form, a missing name keyword, a non-keyword `:name`, and (via the shared
/// `crate::resolve::register` namespacing gate) a reserved/dotted/unnamespaced name all raise
/// `RuntimeErrorKind::MalformedForm`/the gate's own refusals instead of returning a `ClauseSet`
/// — propagated as a hard failure, never a value a caller matches on. Same reasoning
/// `:wat::i64::/`'s own `@Totality Partial` was ruled on
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`). `Partial`.
///
/// **Expand-time ground —** `defclause` has no runtime call site at all (`role = declare` emits
/// no shim); `preregister_defclause_in_env`/`register_defclause_from_form` run from inside
/// `check_program` (`src/check.rs`), step 8 of the startup pipeline — strictly AFTER
/// `expand_all` (step 4, `src/freeze/env.rs`'s `build_env` doc) has already produced the forms
/// they walk. `defclause` is also absent from `macros/eval.rs`'s expand-time pure-total
/// allow-list (measured — no `:wat::core::defclause` arm there), so a `defclause` nested inside
/// a macro body cannot be eagerly evaluated during that macro's own expansion — the identical
/// fact `defsurface`'s/`defmacro`'s rows measured for their own declare-time state.
/// `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Declaration
/// @Purity Unevaluated
/// @Determinism Deterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::core::defclause :name [-> :T] ([args] body) ...)
/// @ret :wat::core::nil no runtime value — the form is consumed entirely at check-time registration and never reaches evaluation; encountered in expression position it raises `DeclarationInExpressionPosition` instead of producing one
/// @example-norun (:wat::core::defclause :probe::id ([n <- :wat::core::i64] -> :wat::core::i64 n)) #=> registers :probe::id into the ClauseSet table; no runtime value
#[wat_special_form(":wat::core::defclause")]
pub(crate) struct Defclause;
