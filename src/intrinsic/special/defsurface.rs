//! Special-form doc entry for `:wat::core::defsurface` — arc 255 Stone 1a-β-0, the witness
//! that gives the freeze/declare-time regime its name (`SpecialFormRole::Declare`).

use wat_macros::wat_special_form;

/// Declare a named structural surface: `:Name`, a mandatory `:nature` bound
/// (`:wat::core::Struct`, `:wat::core::Record`, `:wat::holon::Record`, or
/// `:wat::kernel::Peer`), an optional `:messages` block (peer surfaces only), and a mandatory
/// `:features` member list mixing typed field triples (`name <- :T`) and method lists
/// (`(name [args...] -> :RetType)`). A struct or record satisfies the surface by having every
/// member with a field-type assignable to the member's type — row-polymorphic width
/// subtyping, no `:satisfies`/`:parent`, no declaration at the use site.
///
/// Processed entirely at FREEZE time, before evaluation exists: `register_types_impl`
/// classifies the form, parses it into a `SurfaceDef`, registers it into the `TypeEnv`, and —
/// when the surface carries method members whose request/response sigs are pure —
/// `synthesize_surface_protocol` mints its `<S>::Op` / `<S>::Reply` wire-protocol enums into
/// the SAME registry. The form itself is consumed whole by that pass; it is never spliced into
/// the remaining (evaluated) form stream, so it has no calling convention and no runtime call
/// site to point at — `role = declare` is a REFLECTION fact, "this is the code that processes
/// this form," not a dispatch door (`DESIGN-STONE-1a-beta-0-the-third-regime-gets-its-name.md`).
///
/// **Category ground —** `:Declaration`'s own variant prose (`wat/runtime-meta.wat`) is
/// "registers a program-level entity … visible to everything after it," contrasted there with
/// `:Binding`'s "local, scoped name at runtime." `defsurface` registers `:Name` into the type
/// registry — visible to every form after it in the file, not scoped to a body — exactly that
/// doing, and nothing else in the fifteen-variant set names it. `Declaration`.
///
/// **Purity ground —** all four consumers of `@Purity` ask a RUNTIME question —
/// `rete::purity`'s `pure`/`is_effectful_op` (may this appear in a rule body / does CALLING
/// it have an effect), `purity_mandated_examples` (does it demand a runnable `@example`), and
/// `reflect.rs`'s doc surface — and `defsurface` has no runtime to ask it about: the form is
/// consumed WHOLE at freeze time (`role = declare`, STOP-1) and never reaches evaluation, so
/// it has no calling convention and no runtime call site a claim about "calling it" could even
/// name. All three of the other poles are false statements for exactly this reason: `Pure`
/// would demand a runnable `@example` of a verb that cannot be run (`eval-ast!` has no
/// `handler`/`Eval` impl to fall into) — a false doc claim; `Effectful` says evaluating this
/// has an observable effect, and there is no evaluation to have one; `Preserving` says its
/// purity is its sub-forms', and `:features` is a static member list, never evaluated —
/// nothing to inherit from. `Unevaluated` is the fourth pole built for exactly this row: the
/// axis has no runtime verdict to give, and each of the four consumers computes the correct
/// answer for it with NO edit — `pure` refuses it in a rule body (it is not a runtime
/// expression at all), `is_effectful_op` does not treat it as effectful (there is no call to
/// have one), and `purity_mandated_examples` does not demand a runnable example (the exact
/// trap `Pure` would spring). The registry mutation itself — `register_types_impl`'s `register`
/// closure storing the `SurfaceDef`, `synthesize_surface_protocol` storing the synthesized
/// `::Op`/`::Reply` enums — is real and is what `Category::Declaration` already names
/// ("registers a program-level entity … visible to everything after it"); `@Purity` is a
/// narrower axis asking specifically whether EVALUATING the verb is safe, and that question
/// does not apply to a form with no evaluation
/// (`DESIGN-STONE-1a-beta-0b-a-form-that-never-evaluates.md`). `Unevaluated`.
///
/// **Determinism ground —** the same `defsurface` form, registered against the same preceding
/// declarations (`env`, which `synthesize_surface_protocol`'s own doc notes is consulted "by
/// source order") and the same namespace acronym table, always registers the identical
/// `SurfaceDef` and synthesizes the identical `::Op`/`::Reply` variants — no clock, no entropy,
/// no gensym anywhere on the path. `Deterministic`.
///
/// **Totality ground —** `synthesize_surface_protocol` is measured NOT defined on every
/// `SurfaceDef` in its domain: a serviceable op missing `:max-request-bytes`, a request/
/// response type whose base name violates the `<Op>Request`/`<Op>Response` law, a `Response`
/// enum missing a well-shaped `RequestTooLarge`/`RequestMalformed` variant, or an op that
/// pascal-cases to the reserved `Reply::Failed` all raise a `TypeError` instead of registering
/// anything — a raise the freeze pipeline propagates as a hard failure, never a value a caller
/// matches on. Same reasoning `:wat::i64::/`'s own `@Totality Partial` was ruled on
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`): a raise is not a
/// matchable outcome, so a raising verb is `Partial`.
///
/// **Expand-time ground —** `defsurface` has no runtime call site at all (`role = declare`
/// emits no shim, STOP-1) — the form is consumed whole at freeze and never reaches evaluation
/// — so it is not "legal at expand time versus runtime" in the ordinary two-pole sense either
/// axis pole was built to distinguish. What IS measured: `synthesize_surface_protocol` needs
/// the `TypeEnv` that `register_types` builds incrementally, in source order, over the WHOLE
/// post-expansion form stream (its own doc: "judged against env, which — by source order —
/// already holds the request/response records declared before the surface") — state that
/// categorically does not exist while a `defmacro` body is being expanded, since `expand_all`
/// always finishes before `register_types` ever runs. That is exactly `RuntimeOnly`'s own
/// literal text ("needs state that does not exist yet at expand time"), so it is the honest
/// pole among the two that deny expand-time legality — `Unreviewed` would misrepresent a
/// measured, reasoned absence as one nobody has looked at yet. `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Declaration
/// @Purity Unevaluated
/// @Determinism Deterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::core::defsurface :Name :nature :<nature-root> :features [members])
/// @ret :wat::core::nil no runtime value — the form is consumed entirely at freeze time and never reaches evaluation; its effect is the registration it leaves in the type registry
/// @example-norun (:wat::core::defsurface :geo::Shape :nature :wat::core::Struct :features [(area [self <- :geo::Shape] -> :wat::core::f64)]) #=> registers :geo::Shape into the type registry; no runtime value
#[wat_special_form(":wat::core::defsurface")]
pub(crate) struct Defsurface;
