//! `:wat::edn::*` intrinsics — arc 255 Stone HOME-11, the EDN REGISTRY home.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-HOME-11-edn-gets-a-REGISTRY-home.md`.
//!
//! HOME-5 (`8ddccaaa3`) already carved the FILE home (`src/edn/`) — five loose root files into
//! a named directory. That carve tidied the tree; it did not make any of these 13 names
//! addressable through the intrinsic registry, so `src/resolve/walk.rs:268`'s blanket-accept
//! waved through exactly as many names after HOME-5 as before. This stone is the REGISTRY half:
//! the 13 `:wat::edn::` dispatch arms in `runtime.rs` lift into `#[wat_intrinsic]` handlers here.
//! **Nothing is renamed** — `:wat::edn::` is already the final spelling (pure re-registration,
//! same shape as HOME-8/HOME-10). No codemod, no `RetirementEntry` row, no `.wat` corpus touch.
//!
//! ## The one contract decision — three of these are PRODUCERS
//!
//! `read`, `read-json`, `read-foreign` each mint a `TrackedValue` with
//! `Provenance::RuntimeBuilt { producer, .. }` inside their `src/edn/render.rs` bodies (one
//! `RuntimeBuilt` construction site apiece — measured directly, not guessed from the verb name).
//! Their handlers below return `Result<TrackedValue, EvalBreak>` un-rewrapped, exactly the shape
//! `src/intrinsic/keyword.rs` established and arc 255 Stone G's `sniff_return` requires to keep
//! a registry-routed producer's own stamp alive instead of the shim's default
//! `Provenance::Unknown` rewrap. The other 10 verbs (the four `write*` renderers, `validate`,
//! and the five `ForeignRecord`/`ForeignVariant` accessors) return plain values — no
//! `RuntimeBuilt` site in any of their bodies — so their handlers keep the bare-`Value` shape.
//!
//! ## Two homes, same split HOME-5 established
//!
//! This file is the REGISTRY home — dispatch shim + `///` doc preamble only. The algorithms
//! these handlers call (`crate::edn::render::eval_edn_*`, `crate::edn::render::eval_foreign_*`,
//! `crate::runtime::eval_edn_validate`) are UNTOUCHED by this stone; they already lived in
//! `src/edn/render.rs` (HOME-5) / `src/runtime.rs`, and stay there.
//!
//! `:wat::edn::validate` is the one exception to "moved from `src/edn/render.rs`": its algorithm
//! lives in `src/runtime.rs` (`eval_edn_validate`, widened from a bare `fn` to `pub(crate) fn` so
//! this module can reach it — a visibility-only change, not a behavior change). It is
//! special-cased in `src/check.rs`'s `infer` (arg[1] is a type-position node, not a value to
//! infer) rather than carrying a registered `TypeScheme` — same shape as `:wat::core::conforms?`
//! beside it there, and the same shape `src/intrinsic/kernel/identity.rs`'s
//! `require-wire-address` already established for `@Category CheckGate`: "no registered
//! `TypeScheme` — `check.rs`'s special-cased match arm is the real authority", so
//! `doc_arg_ret_types_match_checker_scheme` (`src/intrinsic/mod.rs`) finds no scheme for this name
//! and skips it rather than firing a false mismatch.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, TrackedValue, Value};

// ─── decode: the 3 producers ────────────────────────────────────────────────

/// `(:wat::edn::read s)` → `:T`. Parses an EDN string into a wat runtime value; the
/// polymorphic-fresh-var return lets the caller's binding context unify with whatever shape the
/// parsed value takes. An unknown tag RAISES (`UnknownTag`) — the strict twin of `read-foreign`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     s :wat::core::String the EDN text parsed
/// @ret     :T the decoded value
/// @example (:wat::edn::read "42") #=> 42
/// @see     :wat::edn::write
/// @see     :wat::edn::read-foreign
#[wat_intrinsic(":wat::edn::read")]
pub(crate) fn eval_edn_read_home(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_edn_read(std::slice::from_ref(s), span, env, sym).map_err(Into::into)
}

/// `(:wat::edn::read-json s)` → `(:wat::edn::ReadJsonOutcome :- [T])`. The JSON-input twin of
/// `read`: parses JSON (not EDN) text into a wat runtime value. TOTAL — never raises — because
/// this verb's input arrives from a remote, untrusted harness over stdio (`wat --mcp`), so a
/// malformed byte must not be able to end the session; parse/decode failure is the matchable
/// `Malformed[cause]` variant instead.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     s :wat::core::String the JSON text parsed
/// @ret     (:wat::edn::ReadJsonOutcome :- [T]) `Value[v]` on success, `Malformed[cause]` otherwise
/// @example (:wat::edn::read-json "42") #=> (:wat::edn::ReadJsonOutcome::Value 42)
/// @see     :wat::edn::read
/// @see     :wat::edn::read-foreign
#[wat_intrinsic(":wat::edn::read-json")]
pub(crate) fn eval_edn_read_json_home(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_edn_read_json(std::slice::from_ref(s), span, env, sym).map_err(Into::into)
}

/// `(:wat::edn::read-foreign s)` → `(:wat::edn::ReadForeignOutcome :- [T])`. The DATA-MODE
/// sibling of `read`: same String→parse→decode path, but an unknown tag reconstructs a
/// self-describing dynamic value (`ForeignRecord` for a map body, `ForeignVariant` for a vector
/// body) instead of raising `UnknownTag` — recursive, so a nested unknown tag decodes all the way
/// down. TOTAL — parse/decode failure is `Malformed[cause]`, never a raise.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     s :wat::core::String the EDN text parsed
/// @ret     (:wat::edn::ReadForeignOutcome :- [T]) `Value[v]` on success, `Malformed[cause]` otherwise
/// @example (:wat::edn::read-foreign "42") #=> (:wat::edn::ReadForeignOutcome::Value 42)
/// @see     :wat::edn::read
/// @see     :wat::edn::ForeignRecord/get
#[wat_intrinsic(":wat::edn::read-foreign")]
pub(crate) fn eval_edn_read_foreign_home(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_edn_read_foreign(std::slice::from_ref(s), span, env, sym).map_err(Into::into)
}

// ─── encode: the 4 writers (plain values, no RuntimeBuilt site) ────────────

/// `(:wat::edn::write v)` → `:wat::core::String`. Compact single-line EDN.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     v :T the value rendered
/// @ret     :wat::core::String the compact single-line EDN text
/// @example (:wat::edn::write 42) #=> "42"
/// @see     :wat::edn::read
#[wat_intrinsic(":wat::edn::write")]
pub(crate) fn eval_edn_write_home(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::edn::render::eval_edn_write(std::slice::from_ref(v), span, env, sym).map_err(Into::into)
}

/// `(:wat::edn::write-pretty v)` → `:wat::core::String`. Multi-line indented EDN.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     v :T the value rendered
/// @ret     :wat::core::String the multi-line indented EDN text
/// @example (:wat::edn::write-pretty 42) #=> "42"
/// @see     :wat::edn::write
#[wat_intrinsic(":wat::edn::write-pretty")]
pub(crate) fn eval_edn_write_pretty_home(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::edn::render::eval_edn_write_pretty(std::slice::from_ref(v), span, env, sym).map_err(Into::into)
}

/// `(:wat::edn::write-json v)` → `:wat::core::String`. JSON via wat-edn's round-trip-safe
/// sentinel-tagged-object convention.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     v :T the value rendered
/// @ret     :wat::core::String the round-trip-safe JSON text
/// @example (:wat::edn::write-json 42) #=> "42"
/// @see     :wat::edn::write-json-natural
#[wat_intrinsic(":wat::edn::write-json")]
pub(crate) fn eval_edn_write_json_home(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::edn::render::eval_edn_write_json(std::slice::from_ref(v), span, env, sym).map_err(Into::into)
}

/// `(:wat::edn::write-json-natural v)` → `:wat::core::String`. Ingestion-tooling-friendly JSON:
/// drops the `#tag`/`body` sentinel wrapping, drops the `:` prefix from keywords, renders
/// Instants as bare ISO-8601 strings. Lossy — round-trip back to wat values is not preserved;
/// use `write-json` for that.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     v :T the value rendered
/// @ret     :wat::core::String the natural (lossy) JSON text
/// @example (:wat::edn::write-json-natural 42) #=> "42"
/// @see     :wat::edn::write-json
#[wat_intrinsic(":wat::edn::write-json-natural")]
pub(crate) fn eval_edn_write_json_natural_home(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::edn::render::eval_edn_write_json_natural(std::slice::from_ref(v), span, env, sym)
        .map_err(Into::into)
}

// ─── validate — a CheckGate, algorithm stays in runtime.rs ─────────────────

/// `(:wat::edn::validate value declared-type)` → `:wat::edn::Validation`. The DEEP shape check
/// `:wat::core::conforms?` structurally cannot do (its Aggregate arm is nominal-only, never
/// recursing into fields). Renders `value` to EDN (the same writer `write` uses) and walks it
/// against `declared-type` field-by-field, yielding the first offending path on a mismatch.
/// Never raises on a bad *value* — a mismatch is the matchable `Invalid[path expected got]`; a
/// bad *type keyword* (unparseable / no registry) is a programmer error and still raises.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      CheckGate
/// @arg     value :T the value checked against `declared_type`'s shape
/// @arg     declared_type :wat::WatAST the type keyword or `(Head :- [args])` type form checked against
/// @ret     :wat::edn::Validation `Valid`, or `Invalid[path expected got]` naming the first mismatch
/// @example (:wat::edn::validate 42 :wat::core::i64) #=> :wat::edn::Validation::Valid
// `@see :wat::core::conforms?` deliberately omitted — `@see` resolves only to REGISTERED
// intrinsics (`all_see_fqdns_resolve_to_registered_intrinsics`), and `conforms?` has not been
// carved into the registry by any stone yet (still a `runtime.rs`/`check.rs` literal, same as
// `validate` was before this one). A dangling cross-reference to it here would be this doc
// lying about what the corpus can actually resolve.
// No registered `TypeScheme` — `src/check.rs`'s special-cased `":wat::edn::validate"` arm in
// `infer` (arg[1] is type-position, not a value to infer, same shape as `conforms?` beside it)
// is the real authority, exactly the precedent `src/intrinsic/kernel/identity.rs`'s
// `require-wire-address` recorded for `@Category CheckGate`.
#[wat_intrinsic(":wat::edn::validate")]
pub(crate) fn eval_edn_validate_home(
    value: &WatAST,
    declared_type: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_edn_validate(&[value.clone(), declared_type.clone()], span, env, sym)
}

// ─── the 5 foreign dynamic-value accessors (plain values) ──────────────────

/// `(:wat::edn::ForeignRecord/get fr key)` → `(:wat::core::Option :- [:wat::core::Value])`.
/// Navigate a foreign record BY KEY (the consumer holds no type). Same contract as
/// `HashMap/get`/`PersistentMap/get`: miss is `None`, never a raise.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     fr :wat::edn::ForeignRecord the foreign record navigated
/// @arg     key :wat::core::keyword the field key looked up
/// @ret     (:wat::core::Option :- [:wat::core::Value]) `Some` the field value, or `None` on a miss
/// @example (:wat::edn::ForeignRecord/get (:wat::core::match (:wat::edn::read-foreign "#some.unknown/Rec {:kind #some.unknown.Kind/Click [42]}") ((:wat::edn::ReadForeignOutcome::Value fr) fr) ((:wat::edn::ReadForeignOutcome::Malformed _) (:wat::kernel::assertion-failed! "bad fixture" :wat::core::None :wat::core::None))) :kind) #=> (:wat::core::Some (:wat::core::match (:wat::edn::read-foreign "#some.unknown.Kind/Click [42]") ((:wat::edn::ReadForeignOutcome::Value fv) fv) ((:wat::edn::ReadForeignOutcome::Malformed _) (:wat::kernel::assertion-failed! "bad fixture" :wat::core::None :wat::core::None))))
/// @see     :wat::edn::read-foreign
/// @see     :wat::edn::ForeignRecord/class
#[wat_intrinsic(":wat::edn::ForeignRecord/get")]
pub(crate) fn eval_foreign_record_get_home(
    fr: &WatAST,
    key: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::edn::render::eval_foreign_record_get(&[fr.clone(), key.clone()], span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::edn::ForeignRecord/class fr)` → `:wat::core::String`. The record's fully-qualified
/// (colon-free) class string.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     fr :wat::edn::ForeignRecord the foreign record probed
/// @ret     :wat::core::String `fr`'s fully-qualified class name
/// @example (:wat::edn::ForeignRecord/class (:wat::core::match (:wat::edn::read-foreign "#some.unknown/Rec {:kind #some.unknown.Kind/Click [42]}") ((:wat::edn::ReadForeignOutcome::Value fr) fr) ((:wat::edn::ReadForeignOutcome::Malformed _) (:wat::kernel::assertion-failed! "bad fixture" :wat::core::None :wat::core::None)))) #=> "some::unknown::Rec"
/// @see     :wat::edn::ForeignRecord/get
#[wat_intrinsic(":wat::edn::ForeignRecord/class")]
pub(crate) fn eval_foreign_record_class_home(
    fr: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::edn::render::eval_foreign_record_class(std::slice::from_ref(fr), span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::edn::ForeignVariant/variant v)` → `:wat::core::Keyword`. The variant name as a
/// keyword (`:Click`). Traffics in `:wat::core::Value` at the argument boundary (heterogeneous),
/// runtime-checking it is a `ForeignVariant` and raising a clean located error otherwise.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     v :wat::core::Value the foreign variant probed
/// @ret     :wat::core::Keyword the variant's name
/// @example (:wat::edn::ForeignVariant/variant (:wat::core::match (:wat::edn::read-foreign "#some.unknown.Kind/Click [42]") ((:wat::edn::ReadForeignOutcome::Value fv) fv) ((:wat::edn::ReadForeignOutcome::Malformed _) (:wat::kernel::assertion-failed! "bad fixture" :wat::core::None :wat::core::None)))) #=> :Click
/// @see     :wat::edn::ForeignVariant/enum-class
#[wat_intrinsic(":wat::edn::ForeignVariant/variant")]
pub(crate) fn eval_foreign_variant_variant_home(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::edn::render::eval_foreign_variant_variant(std::slice::from_ref(v), span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::edn::ForeignVariant/enum-class v)` → `:wat::core::String`. The enum's
/// fully-qualified (colon-free) class string.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     v :wat::core::Value the foreign variant probed
/// @ret     :wat::core::String `v`'s fully-qualified enum class name
/// @example (:wat::edn::ForeignVariant/enum-class (:wat::core::match (:wat::edn::read-foreign "#some.unknown.Kind/Click [42]") ((:wat::edn::ReadForeignOutcome::Value fv) fv) ((:wat::edn::ReadForeignOutcome::Malformed _) (:wat::kernel::assertion-failed! "bad fixture" :wat::core::None :wat::core::None)))) #=> "some::unknown::Kind"
/// @see     :wat::edn::ForeignVariant/variant
#[wat_intrinsic(":wat::edn::ForeignVariant/enum-class")]
pub(crate) fn eval_foreign_variant_enum_class_home(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::edn::render::eval_foreign_variant_enum_class(std::slice::from_ref(v), span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::edn::ForeignVariant/fields v)` → `(:wat::core::Vector :- [:wat::core::Value])`. The
/// positional fields as a vector (each element a `Value`, itself possibly a nested foreign
/// value).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     v :wat::core::Value the foreign variant probed
/// @ret     (:wat::core::Vector :- [:wat::core::Value]) `v`'s positional fields, in order
/// @example (:wat::edn::ForeignVariant/fields (:wat::core::match (:wat::edn::read-foreign "#some.unknown.Kind/Click [42]") ((:wat::edn::ReadForeignOutcome::Value fv) fv) ((:wat::edn::ReadForeignOutcome::Malformed _) (:wat::kernel::assertion-failed! "bad fixture" :wat::core::None :wat::core::None)))) #=> (:wat::core::Vector :- [:wat::core::Value] 42)
/// @see     :wat::edn::ForeignVariant/variant
#[wat_intrinsic(":wat::edn::ForeignVariant/fields")]
pub(crate) fn eval_foreign_variant_fields_home(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::edn::render::eval_foreign_variant_fields(std::slice::from_ref(v), span, env, sym)
        .map_err(Into::into)
}
