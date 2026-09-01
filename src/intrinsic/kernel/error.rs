//! `:wat::kernel::` error-surface intrinsics — arc 255 home #6
//! (255.1c-kernel-error). Four verbs over the two error types —
//! `LociDiedError/message`, `Failure/message`, `Failure/location`,
//! `LociDiedError/to-failure` — one *subject* (the surface of
//! `LociDiedError` and `Failure`), but **NOT one `@Category`**.
//!
//! ## ★ A HOME is a code-organization unit; a CATEGORY is a per-row label
//!
//! Home #4 ruled the carve boundary is the CATEGORY, never the decomposition
//! table's row — a mis-drawn table row must not split a category. That was
//! never "a module must be single-category," and this home is the first
//! proof: three rows are `@Category Projection` (each returns a component that
//! was already there — `wat/runtime-meta.wat`'s `:Projection` prose names all
//! three outright), and the fourth, `LociDiedError/to-failure`, is
//! `@Category Transform` — it matches `ev.variant_name`
//! (`runtime.rs:27812`) and CONSTRUCTS a `Failure`, a **different-kind**
//! value, not a part that was already there. See
//! `DESIGN-STONE-255.1c-kernel-error.md` for the full derivation.
//!
//! ## ★ The `Failure/*` pair projects one hop deeper
//!
//! Arc 278's string-wrap annihilation removed `Failure`'s stored
//! `message`/`location` fields — `Failure` now carries the raised
//! `:wat::core::Error` structurally in a mandatory `error` field.
//! `Failure/message` / `Failure/location` read `error.message` /
//! `error.location` (`runtime.rs:6757`'s dispatch comment). Still
//! `@Category Projection` — a part that already existed — through a hop the
//! rider derives from the body, not the name.
//!
//! ## ★ The bodies do NOT live here
//!
//! Every one of the four delegates to the SAME `crate::kernel::error::eval_*`
//! fn — arc 109 Stone 4a homed the died-error cluster in `src/kernel/error.rs`
//! (docs/arc/2026/04/109-kill-std/); it previously existed as a literal-match
//! arm in `runtime.rs`. See `kernel/mod.rs` for the tier-wide "bodies do not
//! live here" claim this home is an instance of.
//!
//! ## ★ The gate is LIVE here — unlike home #5
//!
//! All four have registered `TypeScheme`s (`check.rs:18101, 18121, 18130,
//! 18147`), so `doc_arg_ret_types_match_checker_scheme` checks every
//! `@arg`/`@ret` below against them. `Failure/*` take `:wat::core::Record`
//! in their schemes — NOT a `:wat::kernel::Failure` path (preserving the
//! prior auto-generated non-generic-record accessor's contract exactly,
//! `register_aggregate_methods`, arc 293.R2.2). The `@arg` types below match
//! the schemes, not the more specific type a reader might expect.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::kernel::LociDiedError/message err)` → `:wat::core::String`.
/// Extracts the carried message from any `:wat::kernel::LociDiedError`
/// variant — a constant string for the unit variants (`Disconnected` /
/// `Stopped`), the carried String (or a structured error's derived
/// `:message`) for the rest. Routes around the wat-side enum-variant
/// pattern-matcher gap — callers ask for a generic message without
/// discriminating variants.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Projection
/// @arg     err :wat::kernel::LociDiedError the death report to read a message from
/// @ret     :wat::core::String the carried message
/// @example (:wat::kernel::LociDiedError/message :wat::kernel::LociDiedError::Disconnected) #=> "disconnected"
// Deciding line for `@Category Projection`: the fn returns a component
// (the message payload) that was already there on the matched variant —
// `runtime.rs:27716` `eval_died_error_message`, every arm reads
// `ev.fields.first()` or returns a fixed literal for a unit variant; no
// new-kind value is built. `wat/runtime-meta.wat`'s `:Projection` prose names
// `LociDiedError/message` outright.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`: the body
// only matches on an already-evaluated `Value::Enum` and reads/returns
// Strings — no I/O, no ambient state, no randomness; same input always
// yields the same output.
#[wat_intrinsic(":wat::kernel::LociDiedError/message")]
pub(crate) fn eval_died_error_message(
    err: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::kernel::error::eval_died_error_message(
        std::slice::from_ref(err),
        env,
        sym,
        ":wat::kernel::LociDiedError",
        list_span,
    )
}

/// `(:wat::kernel::Failure/message f)` → `:wat::core::String`. DERIVED
/// accessor — arc 278's string-wrap annihilation removed the stored
/// `message` field; `Failure` now carries the raised `:wat::core::Error`
/// structurally. Reads `error.message` off the mandatory `error` field.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Projection
/// @arg     f :wat::core::Record the Failure to read a message from
/// @ret     :wat::core::String the message carried on `f`'s `error` field
/// @example (:wat::kernel::Failure/message (:wat::kernel::message-only-failure "boom")) #=> "boom"
// Deciding line for `@Category Projection`: `runtime.rs:27423`
// `eval_failure_message` reads `record_field_by_name(&error, "message", …)`
// off the `error` field already held by `f` — a part that was already
// there, one hop deeper (`f.error.message`, not `f.message`) than the
// pre-arc-278 shape, still a projection.
//
// `@arg f :wat::core::Record`, NOT `:wat::kernel::Failure` — matches the
// registered scheme (`check.rs:18121`) exactly; see the module doc's "gate
// is LIVE" note.
#[wat_intrinsic(":wat::kernel::Failure/message")]
pub(crate) fn eval_failure_message(
    f: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::kernel::error::eval_failure_message(std::slice::from_ref(f), env, sym, list_span)
}

/// `(:wat::kernel::Failure/location f)` → `(:wat::core::Option :- [wat::kernel::Location])`.
/// DERIVED accessor — reads `error.location` (a mandatory
/// `:wat::kernel::Location` on the error) and wraps it in `Some` to keep the
/// accessor's historic `Option<Location>` return shape.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Projection
/// @arg     f :wat::core::Record the Failure to read a location from
/// @ret     (:wat::core::Option :- [:wat::kernel::Location]) `Some` of `f`'s `error.location`
/// @example (:wat::kernel::Failure/location (:wat::kernel::Failure :error (:wat::core::Fault :message "boom" :location (:wat::kernel::Location :file "test" :line 1 :col 1) :causes (:wat::core::Vector :- [:wat::core::Error])) :frames (:wat::core::Vector :- [:wat::kernel::Frame]) :actual :wat::core::None :expected :wat::core::None)) #=> (:wat::core::Some (:wat::kernel::Location :file "test" :line 1 :col 1))
// Deciding line for `@Category Projection`: `runtime.rs:27452`
// `eval_failure_location` reads `record_field_by_name(&error, "location", …)`
// off the `error` field already held by `f` — same one-hop-deeper
// projection as `Failure/message` above.
//
// `@arg f :wat::core::Record`, NOT `:wat::kernel::Failure` — matches the
// registered scheme (`check.rs:18130`) exactly.
#[wat_intrinsic(":wat::kernel::Failure/location")]
pub(crate) fn eval_failure_location(
    f: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::kernel::error::eval_failure_location(std::slice::from_ref(f), env, sym, list_span)
}

/// `(:wat::kernel::LociDiedError/to-failure err)` → `:wat::kernel::Failure`.
/// Always returns a structured Failure, preserving arc 064's
/// actual/expected/location/frames when the death carried an
/// `AssertionPayload`; plain panics / non-panic variants get a
/// message-only Failure.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     err :wat::kernel::LociDiedError the death report to convert
/// @ret     :wat::kernel::Failure a structured Failure built from `err`
/// @example (:wat::kernel::Failure/message (:wat::kernel::LociDiedError/to-failure :wat::kernel::LociDiedError::Disconnected)) #=> "disconnected"
// Deciding line for `@Category Transform`, NOT `Projection` — the other three
// rows in this home: `runtime.rs:27812` `eval_died_error_to_failure`
// matches `ev.variant_name` ("Panic" → extract the message + optional
// carried Failure; every other variant → a message-only Failure) and
// CONSTRUCTS a NEW `Failure` value via `message_only_failure` /
// `failure.clone()` — a different-KIND value than the `LociDiedError` it
// was given, not a component `err` already held. Neither `:Projection` (a
// part that already existed) nor `:Combine` (a larger value of the SAME
// kind) — a representation transform. `wat/runtime-meta.wat`'s `:Projection`
// prose does not name `to-failure`; its omission is deliberate, not an
// oversight (`DESIGN-STONE-255.1c-kernel-error.md`).
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`: same as
// `LociDiedError/message` above — matches an already-evaluated Value,
// builds a new record from it, no I/O/ambient/randomness.
#[wat_intrinsic(":wat::kernel::LociDiedError/to-failure")]
pub(crate) fn eval_died_error_to_failure(
    err: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::kernel::error::eval_died_error_to_failure(
        std::slice::from_ref(err),
        env,
        sym,
        ":wat::kernel::LociDiedError",
        list_span,
    )
}
