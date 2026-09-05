//! Registry entries for the 37 `OpClass::Alias`/`Form`/`Redispatch` rows in `RETE_OPS` whose
//! target is already a registered row — arc 255 Stones 1b-i (29 `Alias` rows) and 1b-ii (6
//! `Form` rows + 2 `Redispatch` rows).
//!
//! DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1b-i-the-alias-surface-and-why-1b-is-not-one-stone.md`
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-1b-i-the-alias-surface.md`
//! DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1b-ii-the-form-and-redispatch-rows-have-no-teacher.md`
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-1b-ii-the-form-and-redispatch-rows.md`
//!
//! Same contract, 37 times over: each struct below
//! declares a name and a target and nothing else. `dispatch_keyword_head`/
//! `dispatch_keyword_head_value` (`src/runtime.rs`) read `alias_of` directly and re-invoke the
//! target with the same unevaluated args and span, before ever consulting a handler — no
//! handler, no `role = eval` implementation, and no delegate exists at any of these names.
//!
//! None of these rows declares any of the five closed-domain axes (`@Purity`/`@Determinism`/
//! `@Totality`/`@ExpandTime`/`@Category`) — arc 255 Stone 2a-b's contract: an alias's axes ARE
//! the target's, resolved by the registry after every submission has folded
//! (`DocError::AliasDeclaresAxis` refuses one declared here at parse time). For the 29
//! `OpClass::Alias` rows (Stone 1b-i), `@arg`/`@ret` types are transcribed verbatim from each
//! row's own `params`/`ret` in `src/rete/vocabulary.rs`'s `RETE_OPS` table — there is nothing to
//! argue and nothing to decide. **That is NOT true of the 8 `Form`/`Redispatch` rows below
//! (Stone 1b-ii)** — `RETE_OPS`'s `params`/`ret` fields are dead for those two classes (`ReteOp`'s
//! own field docs: "Empty for `Form`/`Redispatch`" / "unused for `Form`/`Redispatch`"), so their
//! `@arg`/`@ret` (or `@syntax`/`@ret`, where the target itself has no `@arg`) are copied instead
//! from the TARGET's own registry row, at the `file:line` Stone 1b-ii's BRIEF tabulates.
//!
//! ★★★ `OpClass::Fallback` rows may **never** be aliased. The alias check in
//! `dispatch_keyword_head` fires BEFORE `dispatch_keyword_head_value`'s `RETE_PREFIX` gate is
//! ever reached, so registering a 2-arg alias under a `Fallback` name makes the 4-arg
//! `:undefined` form **unreachable**: the call redirects to the arity-2 core verb and raises
//! `ArityMismatch { expected: 2, got: 4 }` instead of substituting the caller's fallback. That
//! is precisely what broke eight live rete tests when Stone 2a's DESIGN named
//! `:wat::rete::i64::+`. **None of the 37 below is `Fallback`** — every one is
//! `OpClass::Alias`, `OpClass::Form`, or `OpClass::Redispatch` in `RETE_OPS`, verified row by
//! row.
//!
//! ---
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

// ─── i64 ─────────────────────────────────────────────────────────────────────────────────────

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

/// Alias for `:wat::i64::<` — "this name means that name." Calling `(:wat::rete::i64::< a b)`
/// dispatches through the registry's `alias_of` field straight to `:wat::i64::<`; no separate
/// implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::i64::<
/// @arg a :wat::core::i64 the left operand
/// @arg b :wat::core::i64 the right operand
/// @ret :wat::core::bool whether `a` is strictly less than `b` — the target's answer, unchanged
/// @example (:wat::rete::i64::< 1 2) #=> true
#[wat_special_form(":wat::rete::i64::<")]
pub(crate) struct ReteI64Lt;

/// Alias for `:wat::i64::>=` — "this name means that name." Calling `(:wat::rete::i64::>= a b)`
/// dispatches through the registry's `alias_of` field straight to `:wat::i64::>=`; no separate
/// implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::i64::>=
/// @arg a :wat::core::i64 the left operand
/// @arg b :wat::core::i64 the right operand
/// @ret :wat::core::bool whether `a` is greater than or equal to `b` — the target's answer, unchanged
/// @example (:wat::rete::i64::>= 2 1) #=> true
#[wat_special_form(":wat::rete::i64::>=")]
pub(crate) struct ReteI64Ge;

/// Alias for `:wat::i64::<=` — "this name means that name." Calling `(:wat::rete::i64::<= a b)`
/// dispatches through the registry's `alias_of` field straight to `:wat::i64::<=`; no separate
/// implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::i64::<=
/// @arg a :wat::core::i64 the left operand
/// @arg b :wat::core::i64 the right operand
/// @ret :wat::core::bool whether `a` is less than or equal to `b` — the target's answer, unchanged
/// @example (:wat::rete::i64::<= 1 1) #=> true
#[wat_special_form(":wat::rete::i64::<=")]
pub(crate) struct ReteI64Le;

/// Alias for `:wat::i64::=` — "this name means that name." Calling `(:wat::rete::i64::= a b)`
/// dispatches through the registry's `alias_of` field straight to `:wat::i64::=`; no separate
/// implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::i64::=
/// @arg a :wat::core::i64 the left operand
/// @arg b :wat::core::i64 the right operand
/// @ret :wat::core::bool whether `a` equals `b` — the target's answer, unchanged
/// @example (:wat::rete::i64::= 1 1) #=> true
#[wat_special_form(":wat::rete::i64::=")]
pub(crate) struct ReteI64Eq;

/// Alias for `:wat::i64::not=` — "this name means that name." Calling `(:wat::rete::i64::not= a
/// b)` dispatches through the registry's `alias_of` field straight to `:wat::i64::not=`; no
/// separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::i64::not=
/// @arg a :wat::core::i64 the left operand
/// @arg b :wat::core::i64 the right operand
/// @ret :wat::core::bool whether `a` does not equal `b` — the target's answer, unchanged
/// @example (:wat::rete::i64::not= 1 2) #=> true
#[wat_special_form(":wat::rete::i64::not=")]
pub(crate) struct ReteI64NotEq;

/// Alias for `:wat::i64::to-f64` — "this name means that name." Calling
/// `(:wat::rete::i64::to-f64 n)` dispatches through the registry's `alias_of` field straight to
/// `:wat::i64::to-f64`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::i64::to-f64
/// @arg n :wat::core::i64 the value converted
/// @ret :wat::core::f64 `n`, cast to f64 — the target's answer, unchanged
/// @example (:wat::rete::i64::to-f64 3) #=> 3.0
#[wat_special_form(":wat::rete::i64::to-f64")]
pub(crate) struct ReteI64ToF64;

/// Alias for `:wat::i64::to-string` — "this name means that name." Calling
/// `(:wat::rete::i64::to-string n)` dispatches through the registry's `alias_of` field straight
/// to `:wat::i64::to-string`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::i64::to-string
/// @arg n :wat::core::i64 the value rendered
/// @ret :wat::core::String `n`, rendered as a base-10 string — the target's answer, unchanged
/// @example (:wat::rete::i64::to-string 42) #=> "42"
#[wat_special_form(":wat::rete::i64::to-string")]
pub(crate) struct ReteI64ToString;

// ─── f64 ─────────────────────────────────────────────────────────────────────────────────────

/// Alias for `:wat::f64::>` — "this name means that name." Calling `(:wat::rete::f64::> a b)`
/// dispatches through the registry's `alias_of` field straight to `:wat::f64::>`; no separate
/// implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::f64::>
/// @arg a :wat::core::f64 the left operand
/// @arg b :wat::core::f64 the right operand
/// @ret :wat::core::bool whether `a` is strictly greater than `b` — the target's answer, unchanged
/// @example (:wat::rete::f64::> 2.0 1.0) #=> true
#[wat_special_form(":wat::rete::f64::>")]
pub(crate) struct ReteF64Gt;

/// Alias for `:wat::f64::<` — "this name means that name." Calling `(:wat::rete::f64::< a b)`
/// dispatches through the registry's `alias_of` field straight to `:wat::f64::<`; no separate
/// implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::f64::<
/// @arg a :wat::core::f64 the left operand
/// @arg b :wat::core::f64 the right operand
/// @ret :wat::core::bool whether `a` is strictly less than `b` — the target's answer, unchanged
/// @example (:wat::rete::f64::< 1.0 2.0) #=> true
#[wat_special_form(":wat::rete::f64::<")]
pub(crate) struct ReteF64Lt;

/// Alias for `:wat::f64::>=` — "this name means that name." Calling `(:wat::rete::f64::>= a b)`
/// dispatches through the registry's `alias_of` field straight to `:wat::f64::>=`; no separate
/// implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::f64::>=
/// @arg a :wat::core::f64 the left operand
/// @arg b :wat::core::f64 the right operand
/// @ret :wat::core::bool whether `a` is greater than or equal to `b` — the target's answer, unchanged
/// @example (:wat::rete::f64::>= 1.0 1.0) #=> true
#[wat_special_form(":wat::rete::f64::>=")]
pub(crate) struct ReteF64Ge;

/// Alias for `:wat::f64::<=` — "this name means that name." Calling `(:wat::rete::f64::<= a b)`
/// dispatches through the registry's `alias_of` field straight to `:wat::f64::<=`; no separate
/// implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::f64::<=
/// @arg a :wat::core::f64 the left operand
/// @arg b :wat::core::f64 the right operand
/// @ret :wat::core::bool whether `a` is less than or equal to `b` — the target's answer, unchanged
/// @example (:wat::rete::f64::<= 1.0 1.0) #=> true
#[wat_special_form(":wat::rete::f64::<=")]
pub(crate) struct ReteF64Le;

/// Alias for `:wat::f64::=` — "this name means that name." Calling `(:wat::rete::f64::= a b)`
/// dispatches through the registry's `alias_of` field straight to `:wat::f64::=`; no separate
/// implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::f64::=
/// @arg a :wat::core::f64 the left operand
/// @arg b :wat::core::f64 the right operand
/// @ret :wat::core::bool whether `a` equals `b` — the target's answer, unchanged
/// @example (:wat::rete::f64::= 1.0 1.0) #=> true
#[wat_special_form(":wat::rete::f64::=")]
pub(crate) struct ReteF64Eq;

/// Alias for `:wat::f64::not=` — "this name means that name." Calling `(:wat::rete::f64::not= a
/// b)` dispatches through the registry's `alias_of` field straight to `:wat::f64::not=`; no
/// separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::f64::not=
/// @arg a :wat::core::f64 the left operand
/// @arg b :wat::core::f64 the right operand
/// @ret :wat::core::bool whether `a` does not equal `b` — the target's answer, unchanged
/// @example (:wat::rete::f64::not= 1.0 2.0) #=> true
#[wat_special_form(":wat::rete::f64::not=")]
pub(crate) struct ReteF64NotEq;

/// Alias for `:wat::f64::to-string` — "this name means that name." Calling
/// `(:wat::rete::f64::to-string x)` dispatches through the registry's `alias_of` field straight
/// to `:wat::f64::to-string`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::f64::to-string
/// @arg x :wat::core::f64 the value rendered
/// @ret :wat::core::String `x`, rendered as a string — the target's answer, unchanged
/// @example (:wat::rete::f64::to-string 1.5) #=> "1.5"
#[wat_special_form(":wat::rete::f64::to-string")]
pub(crate) struct ReteF64ToString;

// ─── string ──────────────────────────────────────────────────────────────────────────────────

/// Alias for `:wat::string::concat` — "this name means that name." Calling
/// `(:wat::rete::string::concat a b)` dispatches through the registry's `alias_of` field
/// straight to `:wat::string::concat`; no separate implementation exists at this name. The
/// checker constrains this row to exactly two args, matching `RETE_OPS`'s own `params`, even
/// though the core verb's underlying implementation is variadic.
///
/// @added 1.0.0
/// @alias :wat::string::concat
/// @arg a :wat::core::String the first string
/// @arg b :wat::core::String the second string
/// @ret :wat::core::String `a` and `b`, concatenated in order — the target's answer, unchanged
/// @example (:wat::rete::string::concat "a" "b") #=> "ab"
#[wat_special_form(":wat::rete::string::concat")]
pub(crate) struct ReteStringConcat;

/// Alias for `:wat::string::starts-with?` — "this name means that name." Calling
/// `(:wat::rete::string::starts-with? haystack prefix)` dispatches through the registry's
/// `alias_of` field straight to `:wat::string::starts-with?`; no separate implementation exists
/// at this name.
///
/// @added 1.0.0
/// @alias :wat::string::starts-with?
/// @arg haystack :wat::core::String the string examined
/// @arg prefix :wat::core::String the prefix sought
/// @ret :wat::core::bool true iff `haystack` begins with `prefix` — the target's answer, unchanged
/// @example (:wat::rete::string::starts-with? "hello" "he") #=> true
#[wat_special_form(":wat::rete::string::starts-with?")]
pub(crate) struct ReteStringStartsWith;

/// Alias for `:wat::string::ends-with?` — "this name means that name." Calling
/// `(:wat::rete::string::ends-with? haystack suffix)` dispatches through the registry's
/// `alias_of` field straight to `:wat::string::ends-with?`; no separate implementation exists
/// at this name.
///
/// @added 1.0.0
/// @alias :wat::string::ends-with?
/// @arg haystack :wat::core::String the string examined
/// @arg suffix :wat::core::String the suffix sought
/// @ret :wat::core::bool true iff `haystack` ends with `suffix` — the target's answer, unchanged
/// @example (:wat::rete::string::ends-with? "hello" "lo") #=> true
#[wat_special_form(":wat::rete::string::ends-with?")]
pub(crate) struct ReteStringEndsWith;

/// Alias for `:wat::string::contains?` — "this name means that name." Calling
/// `(:wat::rete::string::contains? haystack needle)` dispatches through the registry's
/// `alias_of` field straight to `:wat::string::contains?`; no separate implementation exists at
/// this name.
///
/// @added 1.0.0
/// @alias :wat::string::contains?
/// @arg haystack :wat::core::String the string searched
/// @arg needle :wat::core::String the substring sought
/// @ret :wat::core::bool true iff `needle` occurs anywhere in `haystack` — the target's answer, unchanged
/// @example (:wat::rete::string::contains? "hello" "ell") #=> true
#[wat_special_form(":wat::rete::string::contains?")]
pub(crate) struct ReteStringContains;

/// Alias for `:wat::string::empty?` — "this name means that name." Calling
/// `(:wat::rete::string::empty? s)` dispatches through the registry's `alias_of` field straight
/// to `:wat::string::empty?`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::string::empty?
/// @arg s :wat::core::String the string to test
/// @ret :wat::core::bool true iff `s` has zero characters — the target's answer, unchanged
/// @example (:wat::rete::string::empty? "") #=> true
#[wat_special_form(":wat::rete::string::empty?")]
pub(crate) struct ReteStringEmpty;

/// Alias for `:wat::string::length` — "this name means that name." Calling
/// `(:wat::rete::string::length s)` dispatches through the registry's `alias_of` field straight
/// to `:wat::string::length`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::string::length
/// @arg s :wat::core::String the string to measure
/// @ret :wat::core::i64 the number of Unicode scalar values in `s` — the target's answer, unchanged
/// @example (:wat::rete::string::length "hello") #=> 5
#[wat_special_form(":wat::rete::string::length")]
pub(crate) struct ReteStringLength;

/// Alias for `:wat::string::trim` — "this name means that name." Calling
/// `(:wat::rete::string::trim s)` dispatches through the registry's `alias_of` field straight
/// to `:wat::string::trim`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::string::trim
/// @arg s :wat::core::String the string to trim
/// @ret :wat::core::String the string with leading and trailing whitespace removed — the target's answer, unchanged
/// @example (:wat::rete::string::trim "  hi  ") #=> "hi"
#[wat_special_form(":wat::rete::string::trim")]
pub(crate) struct ReteStringTrim;

/// Alias for `:wat::string::to-lowercase` — "this name means that name." Calling
/// `(:wat::rete::string::to-lowercase s)` dispatches through the registry's `alias_of` field
/// straight to `:wat::string::to-lowercase`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::string::to-lowercase
/// @arg s :wat::core::String the string to lowercase
/// @ret :wat::core::String `s` with every character lowercased — the target's answer, unchanged
/// @example (:wat::rete::string::to-lowercase "HI") #=> "hi"
#[wat_special_form(":wat::rete::string::to-lowercase")]
pub(crate) struct ReteStringToLowercase;

// ─── vector ──────────────────────────────────────────────────────────────────────────────────

/// Alias for `:wat::vector::length` — "this name means that name." Calling
/// `(:wat::rete::vector::length v)` dispatches through the registry's `alias_of` field straight
/// to `:wat::vector::length`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::vector::length
/// @arg v (:wat::core::PersistentVector :- [T]) the vector probed
/// @ret :wat::core::i64 the number of elements in `v` — the target's answer, unchanged
/// @example (:wat::rete::vector::length (:wat::core::PersistentVector 1 2 3)) #=> 3
#[wat_special_form(":wat::rete::vector::length")]
pub(crate) struct ReteVectorLength;

/// Alias for `:wat::vector::contains?` — "this name means that name." Calling
/// `(:wat::rete::vector::contains? v item)` dispatches through the registry's `alias_of` field
/// straight to `:wat::vector::contains?`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::vector::contains?
/// @arg v (:wat::core::PersistentVector :- [T]) the vector probed
/// @arg item :T the candidate element
/// @ret :wat::core::bool true iff `item` occurs in `v` — the target's answer, unchanged
/// @example (:wat::rete::vector::contains? (:wat::core::PersistentVector 1 2 3) 2) #=> true
#[wat_special_form(":wat::rete::vector::contains?")]
pub(crate) struct ReteVectorContains;

// ─── map ─────────────────────────────────────────────────────────────────────────────────────

/// Alias for `:wat::map::contains-key?` — "this name means that name." Calling
/// `(:wat::rete::map::contains-key? m k)` dispatches through the registry's `alias_of` field
/// straight to `:wat::map::contains-key?`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::map::contains-key?
/// @arg m (:wat::core::PersistentMap :- [K V]) the map probed
/// @arg k :K the candidate key
/// @ret :wat::core::bool true iff `k` occurs as a key in `m` — the target's answer, unchanged
/// @example (:wat::rete::map::contains-key? (:wat::map::assoc (:wat::core::PersistentMap) "a" 1) "a") #=> true
#[wat_special_form(":wat::rete::map::contains-key?")]
pub(crate) struct ReteMapContainsKey;

// ─── core ────────────────────────────────────────────────────────────────────────────────────

/// Alias for `:wat::core::not` — "this name means that name." Calling `(:wat::rete::core::not
/// b)` dispatches through the registry's `alias_of` field straight to `:wat::core::not`; no
/// separate implementation exists at this name. `not` is a plain strict fn here, not the `Form`
/// class `and`/`or` need for short-circuiting — it has an ordinary `TypeScheme` and dispatches
/// to `eval_not` like any other alias target.
///
/// @added 1.0.0
/// @alias :wat::core::not
/// @arg b :wat::core::bool the operand
/// @ret :wat::core::bool the logical negation of `b` — the target's answer, unchanged
/// @example (:wat::rete::core::not false) #=> true
#[wat_special_form(":wat::rete::core::not")]
pub(crate) struct ReteCoreNot;

/// Alias for `:wat::core::bool::to-string` — "this name means that name." Calling
/// `(:wat::rete::core::bool::to-string b)` dispatches through the registry's `alias_of` field
/// straight to `:wat::core::bool::to-string`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::core::bool::to-string
/// @arg b :wat::core::bool the value rendered
/// @ret :wat::core::String `b`, rendered as `"true"` or `"false"` — the target's answer, unchanged
/// @example (:wat::rete::core::bool::to-string true) #=> "true"
#[wat_special_form(":wat::rete::core::bool::to-string")]
pub(crate) struct ReteCoreBoolToString;

// ─── holon ───────────────────────────────────────────────────────────────────────────────────

/// Alias for `:wat::holon::presence?` — "this name means that name." Calling
/// `(:wat::rete::holon::presence? target reference)` dispatches through the registry's
/// `alias_of` field straight to `:wat::holon::presence?`; no separate implementation exists at
/// this name.
///
/// @added 1.0.0
/// @alias :wat::holon::presence?
/// @arg target :wat::holon::HolonAST the target and reference operands, in order
/// @arg reference :wat::holon::HolonAST the target and reference operands, in order
/// @ret :wat::core::bool true iff `target` clears the presence floor against `reference` — the target's answer, unchanged
/// @example (:wat::rete::holon::presence? (:wat::holon::leaf "role") (:wat::holon::leaf "role")) #=> (:wat::rete::holon::presence? (:wat::holon::leaf "role") (:wat::holon::leaf "role"))
#[wat_special_form(":wat::rete::holon::presence?")]
pub(crate) struct ReteHolonPresence;

// ─── core (Form — lazy / short-circuiting, mirrored by re-dispatch) ───────────────────────────
//
// arc 255 Stone 1b-ii. These 6 rows are `OpClass::Form` in `RETE_OPS`, not `OpClass::Alias` —
// `dispatch_rete_op`'s `Alias | Form | Redispatch` arm treats all three identically (re-invoke
// `dispatch_keyword_head_value(core_name, …)`), so the alias contract here is unchanged. What
// IS different: `RETE_OPS`'s `params`/`ret` fields are dead for `Form`/`Redispatch` rows
// (`ReteOp`'s own field docs — "Empty for Form/Redispatch" / "unused for Form/Redispatch"), so
// the `@arg`/`@ret` (or `@syntax`/`@ret`, for the three whose target itself has no `@arg`) below
// are copied verbatim from each TARGET's own registry row, never from `RETE_OPS`.

/// Alias for `:wat::core::and` — "this name means that name." Calling `(:wat::rete::core::and
/// exprs...)` dispatches through the registry's `alias_of` field straight to `:wat::core::and`;
/// no separate implementation exists at this name. `and` is a `Form`-class row (lazy,
/// short-circuiting), unlike the plain-fn `Alias` rows above — `dispatch_rete_op` re-invokes the
/// same core form under either class, so laziness carries no risk here.
///
/// @added 1.0.0
/// @alias :wat::core::and
/// @arg exprs… :wat::core::bool the operands, evaluated left to right until the first `:false` (or all of them)
/// @ret :wat::core::bool `:false` at the first `:false` operand, else `:true` (`:true` when there are no operands)
/// @example (:wat::rete::core::and true true) #=> true
#[wat_special_form(":wat::rete::core::and")]
pub(crate) struct ReteCoreAnd;

/// Alias for `:wat::core::or` — "this name means that name." Calling `(:wat::rete::core::or
/// exprs...)` dispatches through the registry's `alias_of` field straight to `:wat::core::or`;
/// no separate implementation exists at this name. `or` is a `Form`-class row (lazy,
/// short-circuiting), unlike the plain-fn `Alias` rows above — `dispatch_rete_op` re-invokes the
/// same core form under either class, so laziness carries no risk here.
///
/// @added 1.0.0
/// @alias :wat::core::or
/// @arg exprs… :wat::core::bool the operands, evaluated left to right until the first `:true` (or all of them)
/// @ret :wat::core::bool `:true` at the first `:true` operand, else `:false` (`:false` when there are no operands)
/// @example (:wat::rete::core::or false true) #=> true
#[wat_special_form(":wat::rete::core::or")]
pub(crate) struct ReteCoreOr;

/// Alias for `:wat::core::if` — "this name means that name." Calling `(:wat::rete::core::if cond
/// then else)` dispatches through the registry's `alias_of` field straight to `:wat::core::if`;
/// no separate implementation exists at this name. The untaken branch is never evaluated,
/// exactly as at the target — `Form`-class laziness carries across the alias unchanged.
///
/// @added 1.0.0
/// @alias :wat::core::if
/// @arg cond :wat::core::Bool the condition to branch on
/// @arg then :T returned when cond is :true (the taken branch)
/// @arg else :T returned when cond is :false (the taken branch)
/// @ret :T the taken branch value; both branches unify to T
/// @example (:wat::rete::core::if true 1 2) #=> 1
#[wat_special_form(":wat::rete::core::if")]
pub(crate) struct ReteCoreIf;

/// Alias for `:wat::core::let` — "this name means that name." Calling `(:wat::rete::core::let
/// [<binder> <expr> ...] <body>+)` dispatches through the registry's `alias_of` field straight
/// to `:wat::core::let`; no separate implementation exists at this name. The target's own row
/// carries no `@arg` — `let`'s shape is structural, not positional — so this row expresses it
/// with `@syntax`, copied from the target, exactly as the target does.
///
/// @added 1.0.0
/// @alias :wat::core::let
/// @syntax (:wat::rete::core::let [<binder> <expr> ...] <body>+)
/// @ret :T the value of the final body form
/// @example (:wat::rete::core::let [x 1 y 2] (:wat::i64::+ x y)) #=> 3
#[wat_special_form(":wat::rete::core::let")]
pub(crate) struct ReteCoreLet;

/// Alias for `:wat::core::match` — "this name means that name." Calling
/// `(:wat::rete::core::match <scrutinee> (<pattern> <body>) ...)` dispatches through the
/// registry's `alias_of` field straight to `:wat::core::match`; no separate implementation
/// exists at this name. The target's own row carries no `@arg` — `match`'s shape is structural,
/// not positional — so this row expresses it with `@syntax`, copied from the target, exactly as
/// the target does.
///
/// @added 1.0.0
/// @alias :wat::core::match
/// @syntax (:wat::rete::core::match <scrutinee> (<pattern> <body>) ...)
/// @ret :T the taken arm's value; every arm unifies to T
/// @example (:wat::rete::core::match (:wat::core::Some 3) ((:wat::core::Some x) x) (:wat::core::None 0)) #=> 3
#[wat_special_form(":wat::rete::core::match")]
pub(crate) struct ReteCoreMatch;

/// Alias for `:wat::core::fn` — "this name means that name." Calling `(:wat::rete::core::fn
/// [<param> <- :T ...] -> :RetType <body>+)` dispatches through the registry's `alias_of` field
/// straight to `:wat::core::fn`; no separate implementation exists at this name. The target's
/// own row carries no `@arg` — `fn`'s shape is structural, not positional — so this row expresses
/// it with `@syntax`, copied from the target, exactly as the target does. `@ret` is a function
/// value, not the dead `ParamType::Bool` `RETE_OPS`'s own row carries for this class.
///
/// @added 1.0.0
/// @alias :wat::core::fn
/// @syntax (:wat::rete::core::fn [<param> <- :T ...] -> :RetType <body>+)
/// @ret :wat::core::Fn the constructed closure, callable with the declared parameter and return types
/// @example ((:wat::rete::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) 7) #=> 7
#[wat_special_form(":wat::rete::core::fn")]
pub(crate) struct ReteCoreFn;

// ─── core · holon (Redispatch) ─────────────────────────────────────────────────────────────────
//
// arc 255 Stone 1b-ii. These 2 rows are `OpClass::Redispatch` in `RETE_OPS` — `dispatch_rete_op`
// treats `Redispatch` identically to `Alias`/`Form` (re-invoke `dispatch_keyword_head_value
// (core_name, …)`). Same dead-`params`/`ret` finding as the `Form` rows above: `@arg`/`@ret`
// below are copied verbatim from each target's own registry row, never from `RETE_OPS`.

/// Alias for `:wat::core::List` — "this name means that name." Calling `(:wat::rete::core::List
/// arg1 arg2 ...)` dispatches through the registry's `alias_of` field straight to
/// `:wat::core::List`; no separate implementation exists at this name. `RETE_OPS`'s own row
/// carries the dead placeholder `ret: ParamType::Bool` for this class — the target's real `@ret`
/// is a `List`, copied here instead.
///
/// @added 1.0.0
/// @alias :wat::core::List
/// @arg vals… :wat::core::Value the elements of the new list, in order
/// @ret :wat::core::List a `List` holding each argument, in order
/// @example (:wat::rete::core::List 1 2 3) #=> (:wat::core::List 1 2 3)
#[wat_special_form(":wat::rete::core::List")]
pub(crate) struct ReteCoreList;

/// Alias for `:wat::holon::coincident?` — "this name means that name." Calling
/// `(:wat::rete::holon::coincident? a b)` dispatches through the registry's `alias_of` field
/// straight to `:wat::holon::coincident?`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::holon::coincident?
/// @arg a :wat::core::Value the two operands compared, in order
/// @arg b :wat::core::Value the two operands compared, in order
/// @ret :wat::core::bool true iff `a` clears the coincident floor against `b`
/// @example (:wat::rete::holon::coincident? (:wat::holon::leaf "role") (:wat::holon::leaf "role")) #=> (:wat::rete::holon::coincident? (:wat::holon::leaf "role") (:wat::holon::leaf "role"))
#[wat_special_form(":wat::rete::holon::coincident?")]
pub(crate) struct ReteHolonCoincident;

// ─── arc 255 Stone the-rete-vocabulary-enters-the-registry, Part 3 — 15 more ───────────────────
//
// The derived worklist was 37 unregistered `RETE_OPS` rows, not 35: this stone's DESIGN measured
// "core_name not registered" as the only reason a row could not be a plain alias today (2 rows:
// `cond`, `reduce`) and called the remaining 35 CLEAR. Measured against this file's OWN header
// warning above (`OpClass::Fallback` rows "may never be aliased" — the alias check in
// `dispatch_keyword_head` fires before `dispatch_keyword_head_value`'s `RETE_PREFIX` gate, so a
// plain alias at a `Fallback` name makes its 4-arg `:undefined`-marker form unreachable), 20 of
// those 35 ARE `OpClass::Fallback` (every `Fallback` row in the whole table, in fact — `i64::{+
// - * / mod rem quot}`, `f64::{+ - * /}`, `holon::{cosine dot}`, `linkedlist::get`,
// `string::subs`, `vec::get`, `vector::get`, `{List,PersistentVector,Vector}/first`). Proven by
// re-adding one (`:wat::rete::i64::+`) as a throwaway probe: it reproduced the exact historical
// regression this file's header already names (`fallback_returns_the_arithmetic_result_when_it_
// does_not_overflow` / `fallback_fires_on_overflow_instead_of_raising`, both FAIL with
// `ArityMismatch { op: ":wat::i64::+", expected: 2, got: 4 }`), then was removed. Those 20 are
// NOT registered here — the design's "35 clear" undercounted this hazard by 20.
//
// `reduce` is ALSO not registered here: this stone's Part 2 (moving `:wat::core::reduce`'s own
// alias from a wat-side `defalias` to a registry row) was found to regress a documented
// check-time guarantee — `probe_arc255_1c_f_reduce_2arity_retired`'s negative witness, which
// proves a malformed 2-arg `reduce` call is refused AT CHECK TIME — passed type-checking
// entirely once `reduce` became a registry alias, only failing later at runtime, renamed to
// `:wat::core::foldl`. Part 2 was reverted; `:wat::core::reduce` remains unregistered, so
// `:wat::rete::core::reduce`'s core_name precondition is still unmet and its row is not minted.
//
// `cond` is ALSO not registered here (nor is Part 1's own `:wat::core::cond` declaration row
// shipped): the brief's prescribed axes for `:wat::core::cond` (`@Purity Preserving`, copied
// verbatim from `:wat::core::if`) make the row FAIL `every_special_form_carries_check_and_eval_
// impls` — a handler-less, non-alias row whose `@Purity` is not `Unevaluated` is required by
// that gate to carry BOTH `role = check` and `role = eval` `#[wat_special_form_impl]`
// annotations, and `cond` (a plain `defmacro`, `wat/core.wat:1455`) has neither: its own FQDN
// never reaches `check_program`'s per-callsite inference nor `dispatch_keyword_head`'s runtime
// dispatch — `expand_all` rewrites it to `if` before either would see it. The only mechanically
// compatible pole is `@Purity Unevaluated` + a `role = declare` impl (the shape every OTHER
// handler-less non-alias row in the registry actually uses — `defclause`/`defsurface`/etc.),
// which contradicts the brief's explicit instruction ("axes, from `if`, unchanged"). Confirmed
// on a clean baseline (this gate passes without `cond`) and reproduced with it added (fails
// naming exactly `:wat::core::cond — missing role: check` / `— missing role: eval`). Reverted;
// reported, not reconciled.
//
// The 15 below are the rows actually clear: targets ALREADY registered, `OpClass` (`Alias`/
// `Form`/`Redispatch`, never `Fallback`) carrying no aliasing hazard.

/// Alias for `:wat::core::=` — "this name means that name." Calling `(:wat::rete::core::bool::=
/// a b)` dispatches through the registry's `alias_of` field straight to `:wat::core::=`; no
/// separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::core::=
/// @arg a :wat::core::bool the left operand
/// @arg b :wat::core::bool the right operand
/// @ret :wat::core::bool true iff `a` structurally equals `b` — the target's answer, unchanged
/// @example (:wat::rete::core::bool::= true true) #=> true
#[wat_special_form(":wat::rete::core::bool::=")]
pub(crate) struct ReteCoreBoolEq;

/// Alias for `:wat::core::not=` — "this name means that name." Calling
/// `(:wat::rete::core::bool::not= a b)` dispatches through the registry's `alias_of` field
/// straight to `:wat::core::not=`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::core::not=
/// @arg a :wat::core::bool the left operand
/// @arg b :wat::core::bool the right operand
/// @ret :wat::core::bool true iff `a` does not structurally equal `b` — the target's answer,
///   unchanged
/// @example (:wat::rete::core::bool::not= true false) #=> true
#[wat_special_form(":wat::rete::core::bool::not=")]
pub(crate) struct ReteCoreBoolNotEq;

/// Alias for `:wat::core::=` — "this name means that name." Calling
/// `(:wat::rete::core::keyword::= a b)` dispatches through the registry's `alias_of` field
/// straight to `:wat::core::=`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::core::=
/// @arg a :wat::core::keyword the left operand
/// @arg b :wat::core::keyword the right operand
/// @ret :wat::core::bool true iff `a` structurally equals `b` — the target's answer, unchanged
/// @example (:wat::rete::core::keyword::= :a :a) #=> true
#[wat_special_form(":wat::rete::core::keyword::=")]
pub(crate) struct ReteCoreKeywordEq;

/// Alias for `:wat::core::not=` — "this name means that name." Calling
/// `(:wat::rete::core::keyword::not= a b)` dispatches through the registry's `alias_of` field
/// straight to `:wat::core::not=`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::core::not=
/// @arg a :wat::core::keyword the left operand
/// @arg b :wat::core::keyword the right operand
/// @ret :wat::core::bool true iff `a` does not structurally equal `b` — the target's answer,
///   unchanged
/// @example (:wat::rete::core::keyword::not= :a :b) #=> true
#[wat_special_form(":wat::rete::core::keyword::not=")]
pub(crate) struct ReteCoreKeywordNotEq;

/// Alias for `:wat::core::=` — "this name means that name." Calling `(:wat::rete::string::= a
/// b)` dispatches through the registry's `alias_of` field straight to `:wat::core::=`; no
/// separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::core::=
/// @arg a :wat::core::String the left operand
/// @arg b :wat::core::String the right operand
/// @ret :wat::core::bool true iff `a` structurally equals `b` — the target's answer, unchanged
/// @example (:wat::rete::string::= "a" "a") #=> true
#[wat_special_form(":wat::rete::string::=")]
pub(crate) struct ReteStringEq;

/// Alias for `:wat::core::not=` — "this name means that name." Calling
/// `(:wat::rete::string::not= a b)` dispatches through the registry's `alias_of` field straight
/// to `:wat::core::not=`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::core::not=
/// @arg a :wat::core::String the left operand
/// @arg b :wat::core::String the right operand
/// @ret :wat::core::bool true iff `a` does not structurally equal `b` — the target's answer,
///   unchanged
/// @example (:wat::rete::string::not= "a" "b") #=> true
#[wat_special_form(":wat::rete::string::not=")]
pub(crate) struct ReteStringNotEq;

/// Alias for `:wat::core::=` — "this name means that name." Calling `(:wat::rete::core::enum::=
/// a b)` dispatches through the registry's `alias_of` field straight to `:wat::core::=`; no
/// separate implementation exists at this name. `RETE_OPS` classes this row `Form` (its checker
/// re-dispatches to `infer_equality` rather than a fixed `TypeScheme`), so `@arg`/`@ret` are
/// copied from the target's own row (`src/runtime.rs`) rather than a `RETE_OPS` `params`/`ret`
/// pair — the same rule this file's header states for the `Form`/`Redispatch` rows above.
///
/// @added 1.0.0
/// @alias :wat::core::=
/// @arg args :wat::core::Value the left operand (position 0) then the right operand (position
///   1); the two must be compatible — their types `unify`, one is a subtype of the other, both
///   are subtypes of `:wat::core::Record`, or both are numeric
/// @ret :wat::core::bool true iff position 0 structurally equals position 1 — the target's
///   answer, unchanged
/// @example (:wat::rete::core::enum::= 1 1) #=> true
#[wat_special_form(":wat::rete::core::enum::=")]
pub(crate) struct ReteCoreEnumEq;

/// Alias for `:wat::core::not=` — "this name means that name." Calling
/// `(:wat::rete::core::enum::not= a b)` dispatches through the registry's `alias_of` field
/// straight to `:wat::core::not=`; no separate implementation exists at this name. `RETE_OPS`
/// classes this row `Form` — same reasoning as `enum::=` above for the `@arg`/`@ret` source.
///
/// @added 1.0.0
/// @alias :wat::core::not=
/// @arg args :wat::core::Value the left operand (position 0) then the right operand (position
///   1); same compatibility rule as `enum::=` above
/// @ret :wat::core::bool true iff position 0 does not structurally equal position 1 — the
///   target's answer, unchanged
/// @example (:wat::rete::core::enum::not= 1 2) #=> true
#[wat_special_form(":wat::rete::core::enum::not=")]
pub(crate) struct ReteCoreEnumNotEq;

/// Alias for `:wat::core::PersistentMap` — "this name means that name." Calling
/// `(:wat::rete::core::PersistentMap :a 1 :b 2)` dispatches through the registry's `alias_of`
/// field straight to `:wat::core::PersistentMap`; no separate implementation exists at this
/// name. `RETE_OPS` classes this row `Redispatch` — `@arg`/`@ret` are copied from the target's
/// own row (`src/runtime.rs`), per this file's header rule for `Form`/`Redispatch` rows.
///
/// @added 1.0.0
/// @alias :wat::core::PersistentMap
/// @arg args… :wat::core::Value alternating key/value pairs, in order — the target's own
///   argument shape, unchanged
/// @ret (:wat::core::PersistentMap :- [K V]) a new persistent map holding each pair — the
///   target's answer, unchanged
/// @example (:wat::rete::core::PersistentMap :a 1 :b 2) #=> (:wat::core::PersistentMap :a 1 :b 2)
#[wat_special_form(":wat::rete::core::PersistentMap")]
pub(crate) struct ReteCorePersistentMap;

/// Alias for `:wat::core::PersistentVector` — "this name means that name." Calling
/// `(:wat::rete::core::PersistentVector 1 2 3)` dispatches through the registry's `alias_of`
/// field straight to `:wat::core::PersistentVector`; no separate implementation exists at this
/// name. `RETE_OPS` classes this row `Redispatch` — same reasoning as `PersistentMap` above.
///
/// @added 1.0.0
/// @alias :wat::core::PersistentVector
/// @arg args… :wat::core::Value the elements, in order (0 or more) — the target's own argument
///   shape, unchanged
/// @ret (:wat::core::PersistentVector :- [T]) a new persistent vector holding each argument, in
///   order — the target's answer, unchanged
/// @example (:wat::rete::core::PersistentVector 1 2 3) #=> (:wat::core::PersistentVector 1 2 3)
#[wat_special_form(":wat::rete::core::PersistentVector")]
pub(crate) struct ReteCorePersistentVector;

/// Alias for `:wat::core::Tuple` — "this name means that name." Calling
/// `(:wat::rete::core::Tuple 1 2)` dispatches through the registry's `alias_of` field straight
/// to `:wat::core::Tuple`; no separate implementation exists at this name. `RETE_OPS` classes
/// this row `Redispatch` — same reasoning as `PersistentMap` above.
///
/// @added 1.0.0
/// @alias :wat::core::Tuple
/// @arg args :T one or more heterogeneous element values, or a `:-`-marked leading type-param
///   bracket — the target's own argument shape, unchanged
/// @ret (:wat::core::Tuple :- [T]) the newly constructed heterogeneous tuple — the target's
///   answer, unchanged
/// @example (:wat::rete::core::Tuple 1 2) #=> (:wat::core::Tuple 1 2)
#[wat_special_form(":wat::rete::core::Tuple")]
pub(crate) struct ReteCoreTuple;

/// Alias for `:wat::core::Vector` — "this name means that name." Calling
/// `(:wat::rete::core::Vector :- [:wat::core::i64] 1 2 3)` dispatches through the registry's
/// `alias_of` field straight to `:wat::core::Vector`; no separate implementation exists at this
/// name. `RETE_OPS` classes this row `Redispatch` — same reasoning as `PersistentMap` above.
///
/// @added 1.0.0
/// @alias :wat::core::Vector
/// @arg args :T a mandatory leading element-type declaration followed by zero or more elements,
///   in order — the target's own argument shape, unchanged
/// @ret (:wat::core::Vector :- [T]) a new vector holding each argument after the leading type
///   declaration, in order — the target's answer, unchanged
/// @example (:wat::rete::core::Vector :- [:wat::core::i64] 1 2 3) #=> [1 2 3]
#[wat_special_form(":wat::rete::core::Vector")]
pub(crate) struct ReteCoreVector;

/// Alias for `:wat::core::filter` — "this name means that name." Calling
/// `(:wat::rete::core::filter pred coll)` dispatches through the registry's `alias_of` field
/// straight to `:wat::core::filter`; no separate implementation exists at this name. `RETE_OPS`
/// classes this row `Redispatch` — same reasoning as `PersistentMap` above.
///
/// @added 1.0.0
/// @alias :wat::core::filter
/// @arg args [:T :-> :wat::core::bool] `pred` (position 0, applied lazily per pulled element)
///   then `coll` (position 1, the receiver) — the target's own argument shape, unchanged
/// @ret (:wat::core::Vector :- [T]) the target's answer, unchanged — NOTE: as with the target,
///   the real runtime return is a lazy `(:wat::stream::Stream :- [T])`
/// @example (:wat::core::stream->vec [] (:wat::rete::core::filter (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::i64::> x 1)) (:wat::core::Vector 1 2 3))) #=> (:wat::core::Vector 2 3)
#[wat_special_form(":wat::rete::core::filter")]
pub(crate) struct ReteCoreFilter;

/// Alias for `:wat::core::foldl` — "this name means that name." Calling
/// `(:wat::rete::core::foldl f init coll)` dispatches through the registry's `alias_of` field
/// straight to `:wat::core::foldl`; no separate implementation exists at this name. `RETE_OPS`
/// classes this row `Redispatch` — same reasoning as `PersistentMap` above.
///
/// @added 1.0.0
/// @alias :wat::core::foldl
/// @arg args [:Acc :T :-> :Acc] `f` (position 0) then `init` (position 1, `:Acc`, the seed
///   accumulator) then `coll` (position 2, the receiver) — the target's own argument shape,
///   unchanged
/// @ret :Acc the final accumulator after folding `f` over every element of `coll` — the
///   target's answer, unchanged
/// @example (:wat::rete::core::foldl (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ acc x)) 0 (:wat::core::Vector 1 2 3)) #=> 6
#[wat_special_form(":wat::rete::core::foldl")]
pub(crate) struct ReteCoreFoldl;

/// Alias for `:wat::core::map` — "this name means that name." Calling `(:wat::rete::core::map f
/// coll)` dispatches through the registry's `alias_of` field straight to `:wat::core::map`; no
/// separate implementation exists at this name. `RETE_OPS` classes this row `Redispatch` — same
/// reasoning as `PersistentMap` above.
///
/// @added 1.0.0
/// @alias :wat::core::map
/// @arg args [:T :-> :U] `f` (position 0, applied lazily per pulled element) then `coll`
///   (position 1, the receiver) — the target's own argument shape, unchanged
/// @ret (:wat::core::Vector :- [U]) the target's answer, unchanged — NOTE: as with the target,
///   the real runtime return is a lazy `(:wat::stream::Stream :- [U])`
/// @example (:wat::core::stream->vec [] (:wat::rete::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ x 1)) (:wat::core::Vector 1 2 3))) #=> (:wat::core::Vector 2 3 4)
#[wat_special_form(":wat::rete::core::map")]
pub(crate) struct ReteCoreMap;
