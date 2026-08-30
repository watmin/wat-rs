//! `:wat::kernel::` abort intrinsics — arc 255 home #8a
//! (255.1c-split-the-remainder, carved from `kernel_remainder.rs`). Two
//! verbs, ONE subject: a call that panics through the wat call stack and
//! never returns a value to its caller. Both are `@Category ControlFlow`.
//!
//! Both delegate to a `pub fn` that already existed before this carve
//! (`crate::runtime::eval_kernel_raise` for `raise!`;
//! `crate::assertion::eval_kernel_assertion_failed` for
//! `assertion-failed!`) — see `kernel/mod.rs` for the tier-wide "bodies do
//! not live here" claim this home is an instance of.
//!
//! ## Why `:ControlFlow`, not a body-read surprise
//!
//! Both bodies end `std::panic::panic_any(payload)` and never return.
//! `:ControlFlow`'s prose, before this carve, described *directing*
//! evaluation — choosing a branch (`if`, applying a callable). Neither verb
//! here does that: both *abandon* evaluation outright, unwinding the Rust
//! call stack until `run-sandboxed`'s `catch_unwind` recovers the payload
//! (`error` structurally for `raise!`; an
//! [`crate::assertion::AssertionPayload`] for `assertion-failed!`, so
//! `Failure.actual`/`Failure.expected` can be populated). Filing these under
//! the old prose without comment would have been a silent widening of what
//! `:ControlFlow` claims to mean. Instead `:ControlFlow` gained one sentence
//! in `wat/runtime-meta.wat`, this stone's carve: ABANDONING evaluation
//! (panic through the call stack) is now named alongside DIRECTING it
//! (choosing a branch) — the taxonomy's own deferred ruling, executed here,
//! not re-argued per verb.
//!
//! ## Purity / Determinism
//!
//! Both are `@Purity Effectful` (an unwind is a real, observable effect —
//! the surrounding `catch_unwind` sees it) and `@Determinism Deterministic`
//! (the same input arguments always produce the same panic payload; no
//! external actor's state changes the outcome).
//!
//! ## Gate coverage
//!
//! Both carry a registered `TypeScheme` (`check.rs`, near `16145` for
//! `raise!`, `16104` for `assertion-failed!`) — gate LIVE.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::kernel::raise! error)` → `:T`. Panics with the caller's
/// `:wat::core::Error` value carried STRUCTURALLY on the panic payload
/// (never `edn::write`'d into a string) — `run-sandboxed`'s `catch_unwind`
/// recovers it into `Failure.error`. Never returns.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      ControlFlow
/// @arg     error :wat::core::Error the error to raise; carried structurally, never stringified
/// @ret     :T never returns — `T` unifies with whatever the caller's context demands
/// @example-norun (:wat::kernel::raise! my-fault) #=> never returns
// Registered `TypeScheme` — `check.rs:16145` — gate LIVE.
//
// Deciding line for `@Category ControlFlow`: `runtime.rs:16197`
// `eval_kernel_raise` ends `std::panic::panic_any(payload)` — never returns.
// `:ControlFlow`'s prose ("directs evaluation") describes CHOOSING a branch;
// this ABANDONS evaluation instead. ARGUED — see the module doc's strain
// report and the strengthened `:ControlFlow` prose in `wat/runtime-meta.wat`.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`: an
// unwind is a real observable effect (the surrounding `catch_unwind` sees
// it); given the same `error` value, the panic payload is the same every
// time — no external actor's state changes the outcome.
#[wat_intrinsic(":wat::kernel::raise!")]
pub(crate) fn eval_kernel_raise(
    error: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_kernel_raise(std::slice::from_ref(error), list_span, env, sym)
}

/// `(:wat::kernel::assertion-failed! message actual expected)` → `:T`.
/// Panics with an [`crate::assertion::AssertionPayload`] so `run-sandboxed`'s
/// `catch_unwind` can populate `Failure.actual`/`Failure.expected`. Never
/// returns.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      ControlFlow
/// @arg     message :wat::core::String short diagnostic (e.g. "assert-eq failed")
/// @arg     actual (:wat::core::Option :- [:wat::core::String]) stringified actual value, when the caller has one
/// @arg     expected (:wat::core::Option :- [:wat::core::String]) stringified expected value, when the caller has one
/// @ret     :T never returns — `T` unifies with whatever the caller's context demands
/// @example-norun (:wat::kernel::assertion-failed! "assert-eq failed" (Some "1") (Some "2")) #=> never returns
// Registered `TypeScheme` — `check.rs:16104` — gate LIVE.
//
// Deciding line for `@Category ControlFlow`: `src/assertion.rs:110`
// `eval_kernel_assertion_failed` ends `std::panic::panic_any(payload)` —
// never returns. Same reasoning as `raise!`: abandons evaluation rather than
// directing it. ARGUED, same paragraph.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`: same
// as `raise!` — a real unwind, and the same three string args always produce
// the same payload.
#[wat_intrinsic(":wat::kernel::assertion-failed!")]
pub(crate) fn eval_kernel_assertion_failed(
    message: &WatAST,
    actual: &WatAST,
    expected: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::assertion::eval_kernel_assertion_failed(
        &[message.clone(), actual.clone(), expected.clone()],
        list_span,
        env,
        sym,
    )
    .map_err(Into::into)
}
