//! Arc 170 slice 3 Gap K — spawn-process lockstep (no-deadlock) verification.
//!
//! Arc 278 IPC de-prime (MAP unit): migrated off the non-prime
//! `:wat::test::run-hermetic` + `run-hermetic-driver` drain-then-join
//! restructure onto the PRIMED peer wire (`spawn-program'`
//! (`:wat::spawn::process`) + `recv'`). The no-deadlock point is preserved: the
//! primed wire ALSO does not deadlock — the parent's `recv'` COMPLETES for both
//! a clean child and a dying one.
//!
//! ## Path exercised
//!
//! Both probes fork an inner `:user::main` via `(:wat::spawn::process)` and
//! `recv'` its outcome:
//!
//! - Probe 1: a clean child returns nil / prints nothing / sends nothing →
//!   `recv'` → `RecvOutcome::Closed`. The fixture returns "closed".
//!   (Mirrors the old `RunResult.failure = None` clean-exit read.)
//!
//! - Probe 2: a child calls `assertion-failed!` → the peer CRASHES before any
//!   send → `recv'` → `RecvOutcome::Lost[cause]`. The fixture returns the
//!   death message (`LociDiedError/message`). (Mirrors the old
//!   `RunResult.failure = Some(...)` read with a non-empty diagnostic.)
//!
//! If a deadlock category were present, `recv'` would hang and neither test
//! would complete. Completing without hang IS the positive verification.

use wat::freeze::call_beside;
use wat::runtime::Value;

fn run_fn(fn_name: &str) -> Value {
    call_beside(file!(), fn_name).expect("probe should run without panicking")
}

fn as_string(v: Value) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Probe 1 — clean child → recv' → Closed; completing = no hang ──────────

/// `spawn-program' (:wat::spawn::process)` with a clean child returning nil.
///
/// The child prints nothing and sends nothing; the parent's `recv'` sees a
/// clean terminal → `RecvOutcome::Closed`. Under a deadlock this `recv'` would
/// hang; completing (and returning "closed") is the positive no-deadlock
/// verification. Path: `spawn-program' (:wat::spawn::process)` + `recv'`.
#[test]
fn probe_run_hermetic_clean_exit_no_deadlock() {
    assert_eq!(
        as_string(run_fn(":probe::test::clean-exit")),
        "closed",
        "expected the clean child to close the wire (RecvOutcome::Closed) without hanging"
    );
}

// ─── Probe 2 — dying child → recv' → Lost[cause]; completing = no hang ─────

/// `spawn-program' (:wat::spawn::process)` with a child that calls
/// `assertion-failed!` (intentional crash).
///
/// The child crashes before any send → the parent's `recv'` returns
/// `Lost[cause]` (a `LociDiedError` carrying the diagnostic). Under a deadlock
/// this `recv'` would hang even on the failure path; completing (and returning
/// a non-empty death message) is the positive verification. Path:
/// `spawn-program' (:wat::spawn::process)` + `recv'`.
#[test]
fn probe_run_hermetic_panic_body_no_deadlock() {
    let msg = as_string(run_fn(":probe::test::intentional-panic"));
    assert!(
        !msg.is_empty(),
        "expected a non-empty death message from the crashed child (Lost[cause]); got empty"
    );
    assert_ne!(
        msg, "UNEXPECTED-MESSAGE",
        "expected Lost (crashed child), not a Message"
    );
    assert_ne!(
        msg, "UNEXPECTED-CLOSED",
        "expected Lost (crashed child), not a clean Closed"
    );
}
