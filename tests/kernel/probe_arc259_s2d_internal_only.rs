//! Arc 259 S2d — the surface cut: `spawn-thread'` / `spawn-process'` / `close'` are
//! INTERNAL-only (the converged model: the ONLY user concurrency entry points are
//! `spawn-program` and `brackets`; `spawn-thread'`/`spawn-process'` are called only
//! by the `spawn-program'` defclause; `close'` is replaced by RAII Drop).
//!
//! Enforced by the `:restricted-to` caller-prefix whitelist (arc 198): these verbs
//! are restricted to `:wat::kernel::` callers, so a `:user::` call is a CHECK error
//! (the `walk_for_restricted_call` walker fires), not a runtime surprise.
//!
//! RED at HEAD: no restriction yet → a `:user::` fn calling `spawn-thread'` /
//! `close'` type-checks fine (startup succeeds). Post-S2d it is rejected at check.
//!
//! Run: `cargo nextest run --release -E 'binary(kernel)' -F probe_arc259_s2d_internal_only`
//!
//! WAT fixtures: tests/kernel/probe_arc259_s2d_internal_only_{spawn_thread,close}.wat.bad

use wat::check::error::CheckErrorKind;
use wat::freeze::startup_from_file;

/// A `:user::` fn calling the internal `spawn-thread'` must be a CHECK error
/// (restricted-to `:wat::kernel::`). RED at HEAD where it is allowed.
#[test]
fn user_calling_spawn_thread_prime_is_a_check_error() {
    let result = startup_from_file(
        "tests/kernel/probe_arc259_s2d_internal_only_spawn_thread.wat.bad",
    );
    wat::assert_startup_error!(result, check
        CheckErrorKind::DefRestrictedCallerNotAllowed { callee, enclosing_fn, prefixes }
            if callee == ":wat::kernel::spawn-thread"
            && enclosing_fn == ":user::compute"
            && prefixes.as_slice() == [":wat::kernel::".to_string()]
    );
}

/// A `:user::` fn calling the internal `close'` must be a CHECK error — teardown is
/// RAII, the user never holds the rope. RED at HEAD where close' is user-callable.
#[test]
fn user_calling_close_prime_is_a_check_error() {
    let result = startup_from_file(
        "tests/kernel/probe_arc259_s2d_internal_only_close.wat.bad",
    );
    wat::assert_startup_error!(result, check
        CheckErrorKind::DefRestrictedCallerNotAllowed { callee, enclosing_fn, prefixes }
            if callee == ":wat::kernel::close"
            && enclosing_fn == ":user::compute"
            && prefixes.as_slice() == [":wat::kernel::".to_string()]
    );
}
