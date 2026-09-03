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
//! A second consequence of the same collision: this row's `OpMeta` — sourced by the BRIEF from
//! `rete::vocabulary::RETE_OPS`'s `:wat::rete::i64::>` row (`meta: OpMeta { pure, deterministic,
//! total }`, all `true`) — describes the FALLBACK-WRAPPED 4-arg form, which never raises only
//! because it catches `IntegerOverflow` and substitutes the caller's fallback value. A bare
//! 2-arg alias has no such catch; it forwards straight to `:wat::i64::>`, which CAN raise
//! `IntegerOverflow` (its own doc, `src/intrinsic/i64.rs`, says so explicitly). Declaring
//! `@Totality Total` here would therefore be the exact lie the totality axis's own doctrine
//! forbids ("a GUESSED `Total` is a lie in a fence that admits code into a `where`") — this row
//! declares `Partial` instead, the honest measurement of what a bare 2-arg redirect actually
//! does, not what the differently-shaped row the BRIEF pointed at does.

use wat_macros::wat_special_form;

/// Alias for `:wat::i64::>` — "this name means that name." Calling `(:wat::rete::i64::> a b)`
/// dispatches through the intrinsic registry's `alias_of` field directly to `:wat::i64::>`,
/// with the same two args and the same call span; no separate implementation exists at this
/// name any more than one exists at any other alias.
///
/// **Purity ground —** an alias adds no effect of its own; it evaluates to whatever its target
/// evaluates to, with the identical args, so its purity IS the target's. `:wat::i64::>` is
/// `Pure` (`src/intrinsic/i64.rs`). `Pure`.
///
/// **Determinism ground —** identical reasoning: the same target, called with the same args,
/// always follows the same redirect to the same deterministic computation. `Deterministic`.
///
/// **Totality ground —** ⚠ NOT copied from the `RETE_OPS` `Fallback`-class row the BRIEF points
/// at (`src/rete/vocabulary.rs`'s `:wat::rete::i64::>` entry, `meta.total: true`) — that row's
/// totality is earned by a 4-arg `:undefined`-catching mechanism this alias does not have. A
/// bare 2-arg redirect to `:wat::i64::>` inherits exactly `:wat::i64::>`'s own real behaviour:
/// pure, deterministic, and documented to raise `RuntimeErrorKind::IntegerOverflow` on overflow
/// (`src/intrinsic/i64.rs`'s own doc, verbatim: "Overflow raises a distinct
/// `RuntimeErrorKind::IntegerOverflow`"). A known raise on some inputs in the declared domain is
/// the textbook case for `Partial`, not `Total` — declaring `Total` here would be an unearned
/// claim admitted into any `where`-fence that trusts this axis. `Partial`.
///
/// **Expand-time ground —** the alias evaluates real sub-forms at its own call site (it forwards
/// unevaluated args straight to the target, which evaluates them) — its own expand-time legality
/// is exactly the target's, the same "depends on what it actually runs" reasoning `and`/`match`
/// use for `Preserving` (`and_form.rs`, `match_form.rs`). `:wat::i64::>`'s own `@ExpandTime` is
/// `Legal` (integer arithmetic, safe during macro expansion, `src/intrinsic/i64.rs`) — mirrored
/// directly rather than restated as `Preserving`, since there is no OTHER sub-form here to
/// preserve over: this row's entire body, so to speak, IS the one call it redirects to. `Legal`.
///
/// **Category ground —** an alias performs no doing of its own — it names another verb
/// (`DESIGN-STONE-2a-…md`). Two candidates refused before picking:
///   - `:Transform` refused — Transform's contract is "returns the SAME value in another
///     FORM" (`wat/runtime-meta.wat`'s own prose: `Bytes::to-hex`, `trim`). An alias returns
///     exactly the target's value, UNCHANGED — no re-encoding happens at this row at all, so
///     there is no "another form" to name.
///   - `:Arithmetic` refused — tempting, since the call ultimately computes a sum, but Category
///     asks what THIS row does, not what it forwards to; `:wat::i64::>` already claims
///     `Arithmetic` for the actual computation, and re-claiming it here would assert this row
///     performs a doing it explicitly does not (the whole point of Shape D / the ★★★ contract).
///   - `:ControlFlow` refused — its members (`if`, applying a callable held as a value) all make
///     a runtime-computed DECISION about which code runs. An alias's redirect is fixed at
///     compile time in the doc, not decided from a value at the call site — there is no
///     branching here, only a renamed jump.
///   - `:Reflection` PICKED — its own prose: "the program interrogating ITSELF." What this row's
///     evaluation actually DOES, independent of the arithmetic it redirects to, is consult the
///     registry's own self-knowledge of what a name resolves to and follow it — a fact about the
///     language's OWN naming structure, not about numbers. That is the one axis on this list
///     that names a self-referential act rather than an effect on data, control, or the program's
///     registered entities.
///
/// @added 1.0.0
/// @Purity Pure
/// @Determinism Deterministic
/// @Totality Partial
/// @ExpandTime Legal
/// @Category Reflection
/// @alias :wat::i64::>
/// @arg a :wat::core::i64 the left operand
/// @arg b :wat::core::i64 the right operand
/// @ret :wat::core::bool whether `a` is strictly greater than `b` — the target's answer, unchanged
/// @example (:wat::rete::i64::> 2 1) #=> true
#[wat_special_form(":wat::rete::i64::>")]
pub(crate) struct ReteI64Gt;
