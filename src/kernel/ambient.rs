//! Kernel sub-module mirroring `src/intrinsic/kernel/ambient.rs` — arc 109
//! Stone B (the seven kernel sub-modules). Three items backing the edge
//! file's seven verbs: `eval_kernel_stopped` (the impl `stopped?` calls),
//! and the two shared bodies `eval_user_signal_query` /
//! `eval_user_signal_reset` that `sigusr1?`/`sigusr2?`/`sighup?` and
//! `reset-sigusr1!`/`reset-sigusr2!`/`reset-sighup!` each parametrize over
//! their own atomic flag (`KERNEL_SIGUSR1`/`KERNEL_SIGUSR2`/`KERNEL_SIGHUP`,
//! passed in by the edge file — those flags stay in `runtime.rs`, unmoved,
//! same as `KERNEL_STOPPED`).
//!
//! Functions lifted out of `runtime.rs` — see `src/kernel/mod.rs` for the
//! layer's scope. Bodies verbatim; only the visibility keyword changed.

use crate::ast::WatAST;
use crate::runtime::KERNEL_STOPPED;
use crate::span::Span;
use crate::value::{EvalBreak, RuntimeError, RuntimeErrorKind, Value};
use std::sync::atomic::{AtomicBool, Ordering};

/// `(:wat::kernel::stopped?)` — nullary predicate; returns the kernel
/// stop flag as a `:bool`. The wat's signal handler sets the flag
/// on SIGINT / SIGTERM; user programs poll it in their loops.
///
/// `?` suffix per the 2026-04-19 naming-convention stance —
/// predicates end in `?`.
pub(crate) fn eval_kernel_stopped(args: &[WatAST], list_span: &Span) -> Result<Value, EvalBreak> {
    if !args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::kernel::stopped?".into(),
                expected: 0,
                got: args.len(),
            },
        )
        .into());
    }
    Ok(Value::bool(KERNEL_STOPPED.load(Ordering::SeqCst)))
}

/// Shared body for the three user-signal predicates — nullary, reads a
/// given atomic flag. `op` is the wat-facing keyword path for error
/// messages.
pub(crate) fn eval_user_signal_query(
    args: &[WatAST],
    op: &str,
    flag: &AtomicBool,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if !args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: op.into(),
                expected: 0,
                got: args.len(),
            },
        )
        .into());
    }
    Ok(Value::bool(flag.load(Ordering::SeqCst)))
}

/// Shared body for the three user-signal resetters — nullary, flips a
/// given atomic flag back to `false`. Unlike the terminal stop flag
/// (set-once), user-signal flags are designed to be toggled by userland
/// after the signal's condition has been handled.
pub(crate) fn eval_user_signal_reset(
    args: &[WatAST],
    op: &str,
    flag: &AtomicBool,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    if !args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: op.into(),
                expected: 0,
                got: args.len(),
            },
        )
        .into());
    }
    flag.store(false, Ordering::SeqCst);
    Ok(Value::Unit)
}
