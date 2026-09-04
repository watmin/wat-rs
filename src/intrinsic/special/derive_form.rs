//! Special-form doc entry for `:wat::core::derive` — arc 255 Stone 1c-d, one of "the
//! declaration three": the edge-only half of `extend-type`, 47 corpus call sites.

use wat_macros::wat_special_form;

/// Register the subtype edge `:Child -> :Parent`: `(:wat::core::derive :Child :Parent)`.
/// `env.register_subtype(&child, &parent, ...)` (`src/types.rs:3886`) records the edge — no
/// method-impl loop, unlike `extend-type`'s general form.
///
/// **Category ground — ★★★ argued, not assumed, per Stone 1a-δ's own lesson, and NOT a copy of
/// `extend-type`'s argument despite the shared shape.** `:Declaration`'s prose fits: the edge
/// `env.register_subtype` writes is consulted by `is_subtype`'s exact-string lookup and
/// `transport_edge_keys`/`transport_satisfier_heads` for the rest of the program — a
/// program-level entity, visible to everything after it.
///
/// The contrast that rules out the other three, measured directly:
/// - NOT `:Splice` — `splice_type_decls`'s `:wat::core::derive` arm (`src/types.rs:3886`) ends
///   `Ok(WatAST::List(items, span))`, keeping the form in the stream (downstream `check.rs`'s
///   `infer_list` arm still consults it) — the opposite of a loader, which replaces itself with
///   N forms and does not survive.
/// - NOT `:CheckGate` — the form's entire purpose is the positive registration of the edge, not
///   a refusal; `:CheckGate`'s own prose ("constrains which programs compile," runtime body
///   "identity or otherwise incidental") does not describe a form whose only job is a write.
/// - NOT `:Ambient` — the edge is keyed by the two NAMED types (`:Child`, `:Parent`), not a
///   bare process-global flag no value addresses.
///
/// ⚠ The one honest asymmetry against `extend-type`'s row, reported rather than smoothed over:
/// `extend-type`'s `parse_extend_type_form` IS the function `declare/register.rs` calls to
/// obtain what it registers — a genuine shared declare/check recognizer. `parse_derive_form`
/// has no such declare-time caller anywhere in the tree (measured — its only call site is
/// `check.rs`'s own `:wat::core::derive` arm); the actual `env.register_subtype` mutation for
/// `derive` runs from a SEPARATE, hand-rolled match arm inside `splice_type_decls`
/// (`src/types.rs:3886`) that re-extracts `:Child`/`:Parent` itself rather than calling this
/// parser. The CATEGORY verdict is unaffected — the registration still happens, just not
/// through this fn — but `role = declare`'s pointer here names the one genuinely-existing
/// `derive`-specific function (the recognizer), not the mutating call site, the same
/// "recognizer carries the annotation" shape `parse_defalias_form`'s own doc records, taken one
/// step further since here the recognizer and the mutator are not even in a caller relationship.
/// `Declaration`.
///
/// **Purity ground —** measured directly, same method as `extend-type`'s row:
/// `:wat::core::derive` has NO entry in `dispatch_keyword_head`/`dispatch_keyword_head_value`'s
/// match and no `NativeHandler` — the DESIGN's own table confirms "NONE" for its dispatch arm.
/// All four consumers of `@Purity` ask a RUNTIME question this form has no runtime to answer:
/// `Pure` would demand a runnable `@example` of a verb that cannot be run; `Effectful` would
/// claim an effect there is no call to have; `Preserving` would claim sub-forms this form
/// itself evaluates — `:Child`/`:Parent` are keywords, read once, never evaluated.
/// `Unevaluated`.
///
/// **Determinism ground —** the same `derive` form, parsed against the same preceding type
/// declarations, always produces the identical `(child, parent)` pair — no clock, no entropy,
/// no gensym anywhere on `parse_derive_form`'s path. `Deterministic`.
///
/// **Totality ground —** `parse_derive_form` is measured NOT defined on every input: an item
/// count other than 3, or a `:Child`/`:Parent` slot that is not a keyword, raises
/// `RuntimeErrorKind::MalformedForm` instead of returning a pair; downstream,
/// `env.register_subtype` can also raise `CyclicSubtype` (`types.rs`'s own comment on this
/// arm) — a raise the freeze pipeline propagates as a hard failure, never a value a caller
/// matches on. Same reasoning `:wat::i64::/`'s own `@Totality Partial` was ruled on
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`). `Partial`.
///
/// **Expand-time ground —** `derive` has no runtime call site at all (`role = declare` emits no
/// shim); its registration runs from `register_stdlib_types`/`register_types`
/// (`src/types.rs`, called from `src/freeze/env.rs`'s `build_env`, step 5) — strictly AFTER
/// `expand_all` (step 4) has produced the forms it walks. Also absent from `macros/eval.rs`'s
/// expand-time pure-total allow-list (measured — no `:wat::core::derive` arm there), so a
/// `derive` nested inside a macro body cannot be eagerly evaluated during that expansion — the
/// identical fact `extend-type`'s sibling row measures for its own declare-time state.
/// `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Declaration
/// @Purity Unevaluated
/// @Determinism Deterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::core::derive :Child :Parent)
/// @ret :wat::core::nil no runtime value — the form is consumed entirely at registration (the type-lattice edge) and never reaches evaluation; encountered in expression position it raises `DeclarationInExpressionPosition` instead of producing one
/// @example-norun (:wat::core::derive :probe::Puppy :probe::Dog) #=> registers the (Puppy, Dog) subtype edge; no runtime value
#[wat_special_form(":wat::core::derive")]
pub(crate) struct Derive;
