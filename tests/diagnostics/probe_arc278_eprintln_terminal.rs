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

use wat::freeze::call_beside;
use wat::runtime::Value;

/// Call a zero-arg compute fn in the co-located fixture and return its
/// `:wat::kernel::RunResult` value.
fn run_fn(fn_name: &str) -> Value {
    call_beside(file!(), fn_name).expect("compute should run")
}

/// Unwrap a `:wat::kernel::RunResult` (wire model — run-hermetic') into its
/// failure slot: `None` if the child signaled its pass-marker, `Some(message)`
/// carrying the crash reason otherwise. `RunResult.stdout`/`stderr` are always
/// empty over the wire and are not inspected.
///
/// RunResult field order (wat/test.wat): `[stdout, stderr, failure <- Option<Failure>]`.
/// A `:wat::kernel::Failure` record's field[0] is its `message` String
/// (src/runtime.rs — Failure ctor: message, location, frames, actual, expected).
fn run_result_failure_message(v: Value) -> Option<String> {
    let sv = match v {
        Value::Aggregate(sv) => sv,
        other => panic!("expected RunResult struct; got {:?}", other),
    };
    assert_eq!(sv.class, "wat::kernel::RunResult");
    assert_eq!(sv.fields.len(), 3);
    let opt = match &sv.fields[2] {
        Value::Option(opt) => (**opt).clone(),
        other => panic!("expected Option for RunResult.failure; got {:?}", other),
    };
    opt.map(|failure| match failure {
        Value::Aggregate(f) => {
            assert_eq!(
                f.class, "wat::kernel::Failure",
                "RunResult.failure must carry a :wat::kernel::Failure; got class {:?}",
                f.class
            );
            match &f.fields[0] {
                Value::String(s) => (**s).clone(),
                other => panic!("Failure.message (field[0]) is not a String; got {:?}", other),
            }
        }
        other => panic!("RunResult.failure = Some(_) must be a Failure record; got {:?}", other),
    })
}

// ─── eprintln is terminal ────────────────────────────────────────────────────

#[test]
fn eprintln_terminates_before_following_forms() {
    let failure = run_result_failure_message(run_fn(":probe::compute-eprintln-terminates"));

    // (a) TERMINATION — the child crashed at the eprintln, so the following forms
    // (`(println "AFTER")` AND run-hermetic's `(println 0)` pass-marker) never ran;
    // recv' saw Lost → RunResult.failure = Some. (Were eprintln benign, the
    // pass-marker would arrive → Message → failure = None.)
    let message = failure.unwrap_or_else(|| {
        panic!(
            "a terminal eprintln must crash the child BEFORE the following forms → \
             RunResult.failure = Some; got None (the child ran to its pass-marker, \
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
    let failure = run_result_failure_message(run_fn(":probe::compute-epprintln-terminates"));

    let message = failure.unwrap_or_else(|| {
        panic!(
            "a terminal epprintln must crash the child BEFORE the following forms → \
             RunResult.failure = Some; got None (the child ran to its pass-marker, \
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
