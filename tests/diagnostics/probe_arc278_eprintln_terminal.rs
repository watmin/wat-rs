//! Arc 278 no-hidden-failures — SUB-STRIKE "eprintln is terminal".
//!
//! `:wat::kernel::eprintln` was DESIGNED as a dying declaration (builder, arc
//! 109 `INVENTORY.md:1284`: "eprintln is a 'we are crashing, here's what I
//! know' and exits") but was IMPLEMENTED as a benign stderr write that returned
//! `Value::Unit` and let the caller continue — the masking law's own shape,
//! baked into the primitive. This probe pins the corrected behavior: a program
//! that runs `(do (eprintln <v>) (println "AFTER"))` must terminate at the
//! eprintln so the following form never runs.
//!
//! IPC de-prime (arc 278): migrated off the non-prime `run-hermetic` (forked +
//! scraped OS pipes into RunResult.stdout/stderr) onto the primed
//! `run-hermetic'` (peer wire; RunResult.stdout/stderr are EMPTY, a crash lands
//! in RunResult.failure = Some[cause]). What of "eprintln is terminal" survives
//! the wire model:
//!
//!   (a) TERMINATION — run-hermetic' appends a `(println 0)` pass-marker after
//!       the body. A terminal eprintln crashes the child BEFORE that marker →
//!       recv' returns Lost[cause] → RunResult.failure = Some. (Were eprintln
//!       benign, the following forms — incl. "AFTER" AND the pass-marker — would
//!       run → the parent recv's a Message → failure = None.) So failure=Some IS
//!       the wire-observable "the following forms never ran" signal; it subsumes
//!       the old "stdout has no AFTER" + "non-zero exit" assertions in one.
//!   (b) The emitted VALUE — `eprintln_terminate` (src/services/verbs.rs) panics
//!       with the value's EDN as the crash reason, which rides the Lost cause's
//!       `:wat::kernel::Failure.message`. So the dying declaration's content
//!       survives via the FAILURE cause, NOT the OS-stderr capture the wire model
//!       drops. We read it out of RunResult.failure and assert it carries the value.
//!
//! `epprintln` (the pretty twin) is pinned identically.

use wat::freeze::{call_beside, DeftestOutcome};
use wat::runtime::Value;

/// Run a zero-arg `:wat::kernel::RunResult`-returning compute fn in the co-located fixture
/// and reduce its verdict to the crash message: `None` if the child signaled its pass-marker,
/// `Some(message)` carrying the crash reason otherwise.
///
/// Arc 278 the vacuous-gate wall — `RunResult` is an ENUM (`:Passed` / `:Failed[failure]`),
/// and a fn with this signature IS a test by the substrate's own criterion (zero params,
/// returning RunResult / TestResult — the same shape `test_runner` discovers), so it is
/// driven by the VERDICT verb `call_beside`, not by the value verb. `Failed` hands the
/// `:wat::kernel::Failure` over directly — no nullable slot to unwrap.
///
/// A `:wat::kernel::Failure`'s field[0] is its `error` (a `:wat::core::Fault`), whose
/// field[0] is the message String.
fn run_result_failure_message(fn_name: &str) -> Option<String> {
    let failure = match call_beside(file!(), fn_name) {
        DeftestOutcome::Passed => return None,
        DeftestOutcome::Failed { failure } => failure,
        DeftestOutcome::DidNotRun { error } => {
            panic!("{fn_name} must evaluate to a RunResult, not raise; got: {error:?}")
        }
    };
    let f = match failure {
        Value::Aggregate(f) => f,
        other => panic!("RunResult::Failed must carry a Failure record; got {other:?}"),
    };
    assert_eq!(
        f.class.as_ref(), "wat::kernel::Failure",
        "RunResult::Failed must carry a :wat::kernel::Failure; got class {:?}",
        f.class
    );
    // Arc 278 — fields[0] is the `error` (Fault); its fields[0] is the message String.
    Some(match &f.fields[0] {
        Value::Aggregate(err) => match &err.fields[0] {
            Value::String(s) => (**s).clone(),
            other => panic!("Failure.error.message is not a String; got {other:?}"),
        },
        other => panic!("Failure.error (field[0]) is not an Aggregate; got {other:?}"),
    })
}

// ─── eprintln is terminal ────────────────────────────────────────────────────

#[test]
fn eprintln_terminates_before_following_forms() {
    let failure = run_result_failure_message(":probe::compute-eprintln-terminates");

    // (a) TERMINATION — the child crashed at the eprintln, so the following forms
    // (`(println "AFTER")` AND run-hermetic's `(println 0)` pass-marker) never ran;
    // recv' saw Lost → RunResult.failure = Some. (Were eprintln benign, the
    // pass-marker would arrive → Message → failure = None.)
    let message = failure.unwrap_or_else(|| {
        panic!(
            "a terminal eprintln must crash the child BEFORE the following forms → \
             RunResult::Failed; got Passed (the child ran to its pass-marker, \
             i.e. eprintln did NOT terminate)"
        )
    });

    // (b) the dying declaration's value survives — it rides the crash cause's
    // Failure.message (the wire model drops OS-stderr capture, but the value's
    // EDN is the panic reason). The message is EXACTLY the value's EDN — a
    // String value writes as its EDN-quoted literal; the crash frames (which
    // carry machine-specific paths) live in Failure.frames, not .message.
    assert_eq!(
        message, "\"dying words\"",
        "the emitted value's EDN (the terminal crash reason) must ride the crash \
         cause's Failure.message over the wire; got: {:?}",
        message
    );
}

// ─── epprintln (pretty twin) is terminal too ─────────────────────────────────

#[test]
fn epprintln_terminates_before_following_forms() {
    let failure = run_result_failure_message(":probe::compute-epprintln-terminates");

    let message = failure.unwrap_or_else(|| {
        panic!(
            "a terminal epprintln must crash the child BEFORE the following forms → \
             RunResult::Failed; got Passed (the child ran to its pass-marker, \
             i.e. epprintln did NOT terminate)"
        )
    });

    assert_eq!(
        message, "\"pretty dying words\"",
        "the emitted value's (pretty) EDN (the terminal crash reason) must ride the \
         crash cause's Failure.message over the wire; got: {:?}",
        message
    );
}
