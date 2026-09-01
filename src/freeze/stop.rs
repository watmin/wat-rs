//! The `:wat::kernel::StopFailure`/`StopFailed` diagnostic vocabulary — arc 109
//! Stone 4c (`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-died-error-cluster-
//! decomposes.md`, map item 4c). Eight items: the two field-name caches
//! (`stop_failure_names`, `stop_failed_names`), the three value builders
//! (`stop_failure_value`, `stop_failure_from_panic`, `stop_failed_value`), and
//! the single-slot publish/take hand-off main uses to carry its collected
//! failures from `ProcessRuntime::ask_stop_and_collect_failures`
//! (`src/freeze.rs`) to the exit path (`src/distribution/mod.rs`):
//! `publish_stop_failures`, `take_stop_failures`, and the `STOP_FAILURES_PTR`
//! static they swap — a MEMBER of this vocabulary, not a reach-back
//! dependency, because only these two functions touch it.
//!
//! Measured: `src/freeze.rs` and `src/distribution/mod.rs` are this
//! vocabulary's only callers anywhere in the tree; every other reference is
//! a doc mention. `fault_from_runtime_error`/`fault_from_panic_payload`
//! (called here to build each `StopFailure`'s cause) stay in `runtime.rs` —
//! they are the shared `:wat::core::Fault` diagnostic language (map item 4d,
//! eight consuming homes), not this vocabulary's to own.
//!
//! Functions lifted out of `runtime.rs` — bodies verbatim; only the
//! visibility keyword changed.

use crate::runtime::{
    fault_from_panic_payload, fault_from_runtime_error, STOP_FAILED_FIELDS, STOP_FAILURE_FIELDS,
};
use crate::value::{AggregateValue, RuntimeError, Value};
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub(crate) fn stop_failure_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(STOP_FAILURE_FIELDS))
        .clone()
}
pub(crate) fn stop_failed_names() -> Arc<Vec<String>> {
    static N: std::sync::OnceLock<Arc<Vec<String>>> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(STOP_FAILED_FIELDS))
        .clone()
}

/// Build one `:wat::kernel::StopFailure` — the service's display name + its structured cause.
pub(crate) fn stop_failure_value(service: &str, err: &RuntimeError) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::kernel::StopFailure".to_string(),
        stop_failure_names(),
        Arc::new(vec![
            Value::String(Arc::new(service.to_string())),
            fault_from_runtime_error(err),
        ]),
    )))
}

/// Build one `:wat::kernel::StopFailure` from a caught panic (see [`fault_from_panic_payload`]).
pub(crate) fn stop_failure_from_panic(
    service: &str,
    payload: &(dyn std::any::Any + Send),
) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::kernel::StopFailure".to_string(),
        stop_failure_names(),
        Arc::new(vec![
            Value::String(Arc::new(service.to_string())),
            fault_from_panic_payload(payload),
        ]),
    )))
}

/// Build `:wat::kernel::StopFailed { :services [...] }` from main's collected failures.
/// `pub(crate)` — the exit path (`src/distribution/mod.rs`) calls this after
/// [`take_stop_failures`] to build the value it serializes to stderr.
pub(crate) fn stop_failed_value(failures: Vec<Value>) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::kernel::StopFailed".to_string(),
        stop_failed_names(),
        Arc::new(vec![Value::Vec(Arc::new(failures))]),
    )))
}

/// Main's collected `StopFailure` values, published ONCE (if non-empty) by
/// `ProcessRuntime::ask_stop_and_collect_failures` (`src/freeze.rs`), read ONCE by the exit path
/// (`src/distribution/mod.rs`) right after `invoke_user_main` returns. `null` = no failures were
/// recorded — either no stop ever happened, or one happened with nothing to report.
///
/// Same-thread hand-off now (both the write and the read happen on main, in the same call chain,
/// with no other thread able to observe or race this slot in between) — kept as a global rather
/// than threaded through `invoke_user_main`'s return type because that signature is public API
/// dozens of tests call directly and don't care about this.
static STOP_FAILURES_PTR: std::sync::atomic::AtomicPtr<Vec<Value>> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

/// Publish main's collected failures. Called at most once per `:user::main` return that observed
/// `KERNEL_STOPPED`.
pub(crate) fn publish_stop_failures(failures: Vec<Value>) {
    let boxed = Box::into_raw(Box::new(failures));
    let old = STOP_FAILURES_PTR.swap(boxed, Ordering::SeqCst);
    if !old.is_null() {
        unsafe { drop(Box::from_raw(old)) };
    }
}

/// Take the collected stop failures (swap-to-null; idempotent — a second call sees null and
/// returns empty). See [`STOP_FAILURES_PTR`]'s doc.
pub(crate) fn take_stop_failures() -> Vec<Value> {
    let ptr = STOP_FAILURES_PTR.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if ptr.is_null() {
        Vec::new()
    } else {
        // SAFETY: ptr was Box::into_raw'd in publish_stop_failures and is no longer
        // reachable via STOP_FAILURES_PTR after this swap.
        *unsafe { Box::from_raw(ptr) }
    }
}
