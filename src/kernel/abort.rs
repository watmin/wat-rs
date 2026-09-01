//! Kernel sub-module mirroring `src/intrinsic/kernel/abort.rs` — arc 109
//! Stone B (the seven kernel sub-modules). One item: `eval_kernel_raise`,
//! the impl `(:wat::kernel::raise!)` delegates to. `raise!` is one of the
//! edge file's two verbs (the other, `assertion-failed!`, already lives in
//! `src/assertion.rs` — never `runtime.rs` — so this module holds exactly
//! the one item the edge names as `runtime.rs`'s).
//!
//! Functions lifted out of `runtime.rs` — see `src/kernel/mod.rs` for the
//! layer's scope. Bodies verbatim; only the visibility keyword changed.

use crate::ast::WatAST;
use crate::runtime::{eval_inner, record_field_by_name};
use crate::span::Span;
use crate::value::{
    snapshot_call_stack, Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable,
    Value,
};

/// `(:wat::kernel::raise! data) -> :T` — arc 296 re-gate.
/// The structured-error sibling of `assertion-failed!`.
///
/// **Arc 278 the string-wrap annihilation.** The raised `:wat::core::Error`
/// is carried STRUCTURALLY on the panic payload's `raised_error` field —
/// it is NEVER `edn::write`'d into a String. `failure_value_from_assertion_payload`
/// reads it into `:wat::kernel::Failure`'s mandatory `error` field, so the
/// receiver recovers the original error RECORD directly:
/// `(:wat::kernel::Failure/error f)` yields the `:wat::core::Fault` (or any
/// Error) value — no `edn::read`, no string re-parse. The derived
/// `(:wat::kernel::Failure/message f)` reads `error.message` (the human string).
///
/// The checker enforces the `:wat::core::Error` constraint at compile time
/// (re-gate in check.rs); this function accepts any `Value` and the structural
/// guarantee is the wall, not a runtime gate.
///
/// Argument: `:wat::core::Error`. Return type: polymorphic `:T`
/// (never returns; same convention as assertion-failed!).
pub(crate) fn eval_kernel_raise(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::raise!";
    if args.len() != 1 {
        // arc 138: no span — leaf helper without list_span; threading
        // would require touching the entire dispatcher arm chain.
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let data = eval_inner(&args[0], env, sym)?.value_owned();
    // Arc 278 the string-wrap annihilation — carry the raised `:wat::core::Error`
    // STRUCTURALLY on the payload (never `edn::write` it into `message`). The
    // human `message` field is the error's OWN `message` field (a String), so
    // the `#wat.kernel/AssertionFailure` envelope + `LociDiedError::Panic.message`
    // stay honest human strings; the structured error rides on `raised_error` and
    // lands in `Failure.error` (recovered via `(:wat::kernel::Failure/error f)`).
    let types = sym.types().map(|a| a.as_ref());
    let message = record_field_by_name(&data, "message", types)
        .and_then(|v| match v {
            Value::String(s) => Some((*s).clone()),
            _ => None,
        })
        // Defensive: the checker gates `data` to `:wat::core::Error` (a String
        // `message` field), so this only fires for an out-of-band caller — fall
        // back to the EDN rendering rather than an empty message.
        .unwrap_or_else(|| wat_edn::write(&crate::edn::render::value_to_edn_with(&data, types)));
    let frames = snapshot_call_stack();
    let location = frames.first().map(|f| f.call_span.clone());
    let payload = crate::assertion::AssertionPayload {
        message,
        actual: None,
        expected: None,
        location,
        frames,
        upstream_chain: None,
        // Arc 138 F-NAMES-1d — capture name on the panicking thread.
        thread_name: std::thread::current().name().map(String::from),
        // Arc 278 — the raised Error, carried as a structured record.
        raised_error: Some(data),
    };
    std::panic::panic_any(payload);
}
