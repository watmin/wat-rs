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
//! ⚠ **A finding this file's own doc block must carry, not bury** — see the rider report for
//! the full argument: `:wat::rete::i64::>` is ALREADY a live FQDN in `src/rete/vocabulary.rs`
//! (`RETE_OPS`), registered there as `OpClass::Fallback` (NOT `OpClass::Alias`, contra this
//! stone's own DESIGN prose), taking 4 positional args
//! `(a b :undefined fallback)` and catching `IntegerOverflow` to substitute `fallback`. That row
//! is untouched (STOP-3) and still present. But because `dispatch_keyword_head`'s new alias
//! check (STOP-1's own proof requirement) must fire BEFORE `dispatch_keyword_head_value`'s
//! `RETE_PREFIX` gate is ever reached, registering an alias under this SAME name makes the OLD
//! 4-arg fallback-carrying form of `:wat::rete::i64::>` **unreachable** — a 4-arg call now
//! redirects straight to `:wat::i64::>` (arity 2) and raises `ArityMismatch{expected:2,got:4}`
//! instead of computing the sum with overflow fallback. Several `tests/rete/` fixtures call the
//! 4-arg form today (`probe_arc278_8custom_native_differential.rs`,
//! `probe_arc278_55_slice_one_vocabulary.wat`, `probe_arc278_then_user_forms*.{rs,wat}`,
//! `rete/clause.rs`, `rete/compiled_rhs.rs`) and are expected to break. This is a consequence of
//! the exact FQDN the BRIEF names (repeated in its work section, STOP-1, and Sabotage-1) — not a
//! defect in how this struct is wired.
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
