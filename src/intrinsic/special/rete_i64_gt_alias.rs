//! Registry entry for `:wat::rete::i64::>` — arc 255 Stone 2a, the alias witness.
//!
//! DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-2a-the-alias-field-and-why-1b-was-blocked-twice.md`
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-2a-the-alias-field.md`
//!
//! `@alias` — not documentation, the DISPATCH: `dispatch_keyword_head`/`dispatch_keyword_head_value`
//! (`src/runtime.rs`) read `alias_of` directly and re-invoke the target with the same
//! unevaluated args and span, before ever consulting a handler. This struct carries no
//! handler, no `role = eval` implementation, and no delegate — the whole ★★★ contract is that
//! an alias needs none (STOP-2).
//!
//! ⛔ **CORRECTED 2026-09-02 — this paragraph used to assert the opposite, and it was false.**
//! It was written about the DESIGN's ORIGINAL witness, `:wat::rete::i64::+`, where every word of
//! it held; when the witness moved to `:wat::rete::i64::>` the NAME was substituted and the
//! ASSERTIONS were not. Measured on disk: `RETE_OPS`'s `:wat::rete::i64::>` row is
//! `class: OpClass::Alias`, `params: &[ParamType::I64, ParamType::I64]`, `ret: ParamType::Bool`
//! — and `git blame` puts `OpClass::Alias` on that line since 2026-08-02, so this name was
//! NEVER `Fallback`. No corpus call site passes it four arguments; every 4-arg `:undefined`
//! site in the tree belongs to `i64::{+ - * / mod rem quot}` / `f64::*` / `vector::get` / the
//! `*/first` trio / `string::subs`, which really are `Fallback` rows. Registering an alias at
//! this name therefore collides with NOTHING — the registry now answers what `RETE_OPS` already
//! said, which is the whole point of the fold.
//!
//! ★★★ **The warning that paragraph carried is real — it just belongs to a different class, and
//! Phase 1b is the thing that must read it.** For a genuine `OpClass::Fallback` row,
//! `dispatch_keyword_head`'s alias check fires BEFORE `dispatch_keyword_head_value`'s
//! `RETE_PREFIX` gate is ever reached, so registering a 2-arg alias under a `Fallback` name
//! makes the 4-arg `:undefined` form **unreachable**: the call redirects to the arity-2 core
//! verb and raises `ArityMismatch { expected: 2, got: 4 }` instead of substituting the caller's
//! fallback. That is precisely what broke eight live rete tests when the DESIGN named `+`.
//! **`Fallback`'s 20 rows may not be registered as aliases** — that is a mechanism, not a
//! preference, and it is why the SEAM carries the same prohibition.
//!
//! ---
//!
//! **arc 255 Stone 2a-b** — this row used to carry its own `@Purity`/`@Determinism`/
//! `@Totality`/`@ExpandTime`/`@Category` lines, argued from first principles below the fold. It
//! doesn't any more: an alias is a second name for one verb, not a verb with properties of its
//! own, and declaring five axes for it was five opportunities to disagree with the target — one
//! of which it took, within the hour, by the same hand that wrote the ruling
//! (`DESIGN-STONE-2a-b-an-alias-inherits-it-does-not-declare.md`'s measurement:
//! `@Totality Partial` here against `:wat::i64::>`'s own `Total`, `@Category Reflection` here
//! against `:wat::i64::>`'s own `Probe` — one behaviour, reported two ways). The registry now
//! derives all five from the target at fold time (`registry()`'s second pass, `src/intrinsic/
//! mod.rs`); declaring one here is refused at parse time (`wat_doc::DocError::
//! AliasDeclaresAxis`), not silently accepted or silently dropped. `render-doc`/`metadata-of`
//! on this FQDN now report exactly `:wat::i64::>`'s own five axes, always — there is nothing
//! left here that could drift out from under them.

use wat_macros::wat_special_form;

/// Alias for `:wat::i64::>` — "this name means that name." Calling `(:wat::rete::i64::> a b)`
/// dispatches through the intrinsic registry's `alias_of` field directly to `:wat::i64::>`,
/// with the same two args and the same call span; no separate implementation exists at this
/// name any more than one exists at any other alias.
///
/// This row declares none of the five closed-domain axes (`@Purity`/`@Determinism`/
/// `@Totality`/`@ExpandTime`/`@Category`) — arc 255 Stone 2a-b's contract: an alias's axes ARE
/// the target's, resolved by the registry after every submission has folded, not restated here
/// where they could disagree. `(:wat::core::render-doc :wat::rete::i64::>)` and
/// `(:wat::runtime::metadata-of :wat::rete::i64::>)` report `:wat::i64::>`'s own current
/// `Pure` / `Deterministic` / `Partial` / `Legal` / `Probe` — see that verb's own doc
/// (`src/intrinsic/i64.rs`) for the grounds; there is nothing to re-argue at this name.
///
/// @added 1.0.0
/// @alias :wat::i64::>
/// @arg a :wat::core::i64 the left operand
/// @arg b :wat::core::i64 the right operand
/// @ret :wat::core::bool whether `a` is strictly greater than `b` — the target's answer, unchanged
/// @example (:wat::rete::i64::> 2 1) #=> true
#[wat_special_form(":wat::rete::i64::>")]
pub(crate) struct ReteI64Gt;
