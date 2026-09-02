//! Special-form doc entry for `:wat::core::structtype` — arc 255 Stone 1a-β-i, one of the
//! type-declaration family that shares `defsurface`'s regime (`SpecialFormRole::Declare`,
//! `@Purity Unevaluated`, arc 255 Stone 1a-β-0/0b).
//!
//! ⚠ `structtype` is the Rust-registered primitive `:wat::core::defstruct` (the stdlib
//! `defmacro` in `wat/core.wat:2030`) lowers into — measured while working this stone: a
//! malformed `(:wat::core::defstruct :probe::Bad)` at `--check` raises a `MalformedDecl`
//! whose `:head` is `"structtype"` and whose text is `parse_structtype`/`parse_aggregate`'s
//! own arity message, not `parse_defstruct`'s (`src/types/defstruct.rs:520`). `structtype` is
//! therefore the form that ACTUALLY registers whatever a user spells `defstruct`; see
//! `defstruct`'s own STOP-5 finding in the stone's report for why `defstruct` is not annotated
//! this stone.

use wat_macros::wat_special_form;

/// Declare a named struct type: `:Name`, an optional binder (`:- [T…]`), an optional
/// metadata-map (`:restricted-to`/`:field-metadata`), and a mandatory field-vector
/// (`[field <- :T ...]`). `structtype` is the low-level primitive: `parse_structtype`
/// (`src/types.rs`) injects `:wat::core::Struct` as the parent and delegates to
/// `parse_aggregate`, which registers a `TypeDef::Aggregate` with `Nature::Struct` and mints
/// the struct's constructor + field accessors.
///
/// Processed entirely at FREEZE time, before evaluation exists: `parse_structtype` parses the
/// form into a `TypeDef`, which `register_types_impl` registers into the `TypeEnv` and — via
/// `register_aggregate_methods` (freeze/env.rs) — mints the ctor and accessor fns. The form
/// itself is consumed whole by that pass; it is never spliced into the remaining (evaluated)
/// form stream, so it has no calling convention and no runtime call site to point at —
/// `role = declare` is a REFLECTION fact, "this is the code that processes this form," not a
/// dispatch door, exactly as `defsurface`'s row argues
/// (`DESIGN-STONE-1a-beta-0-the-third-regime-gets-its-name.md`).
///
/// **Category ground —** same as `defsurface`'s: `structtype` registers `:Name` into the
/// type registry — visible to every form after it in the file, not scoped to a body —
/// exactly `Declaration`'s own variant prose ("registers a program-level entity … visible
/// to everything after it"). `Declaration`.
///
/// **Purity ground —** measured directly: `:wat::core::structtype` appears in
/// `src/runtime.rs` exactly ONCE, inside `is_mutation_head` — a hand-list, not a dispatch
/// arm — and nowhere in `dispatch_keyword_head_value`, `eval_tail`, or `step_list`. No
/// `handler`, no eval arm, no tail arm. Same reasoning as `defsurface`'s row
/// (`DESIGN-STONE-1a-beta-0b-a-form-that-never-evaluates.md`): all four consumers of
/// `@Purity` ask a RUNTIME question, and `structtype` has no runtime to ask it about —
/// `Pure` would demand a runnable `@example` of a verb that cannot be run, `Effectful` would
/// claim an effect there is no call to have, `Preserving` would claim sub-forms that are
/// never evaluated (the field-vector is a static list, parsed once). `Unevaluated`.
///
/// **Determinism ground —** the same `structtype` form, parsed against the same preceding
/// declarations, always registers the identical `TypeDef` — no clock, no entropy, no gensym
/// anywhere on `parse_structtype`/`parse_aggregate`'s path. `Deterministic`.
///
/// **Totality ground —** `parse_aggregate` (which `parse_structtype` delegates to, after
/// injecting the `:wat::core::Struct` parent) is measured NOT defined on every input: a
/// missing name, a non-keyword parent, a parent that is not a nature-root, or a malformed
/// field-vector all raise `TypeError::MalformedDecl` instead of returning a `TypeDef` — a
/// raise the freeze pipeline propagates as a hard failure, never a value a caller matches
/// on. Same reasoning `:wat::i64::/`'s own `@Totality Partial` was ruled on
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`). `Partial`.
///
/// **Expand-time ground —** `structtype` has no runtime call site at all (`role = declare`
/// emits no shim) — the form is consumed whole at freeze and never reaches evaluation.
/// `parse_type_decl` (its router) is invoked only from `register_types_impl`, which the
/// freeze pipeline's own phase order runs strictly AFTER `expand_all` completes
/// (`src/freeze.rs`'s pipeline doc, step 4 → step 5) — state that categorically does not
/// exist while a `defmacro` body is being expanded. Same fact `defsurface`'s row measured for
/// `synthesize_surface_protocol`'s `env` dependency; `parse_structtype`/`parse_aggregate`
/// reach the identical pipeline position and also take `env` directly (to validate the
/// parent keyword resolves to a registered nature-root). `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Declaration
/// @Purity Unevaluated
/// @Determinism Deterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::core::structtype :Name [field <- :T ...])
/// @ret :wat::core::nil no runtime value — the form is consumed entirely at freeze time and never reaches evaluation; its effect is the registration it leaves in the type registry
/// @example-norun (:wat::core::structtype :geo::Pt2 [x <- :wat::core::i64]) #=> registers :geo::Pt2 into the type registry; no runtime value
#[wat_special_form(":wat::core::structtype")]
pub(crate) struct Structtype;
