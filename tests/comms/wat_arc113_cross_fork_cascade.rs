//! Arc 113 slice 3 — cross-fork cascade. Proves the cascade chain
//! preserves AssertionPayload structure (location, actual, expected,
//! frames) across a real fork boundary, via stderr-EDN as the
//! transport.
//!
//! Arc 278 IPC de-prime (MAP unit): the driver migrated off the retired
//! non-prime `:wat::test::run-hermetic` (fork + stderr-EDN scrape into a
//! RunResult) onto the PRIMED peer wire. The outer wat program now
//! `spawn-program' (process)`s a child whose `:user::main` triggers
//! `assert-eq`; the child body is UNCHANGED.
//!
//! Pattern: the body runs in a forked OS process; the substrate's
//! catch_unwind captures the AssertionPayload panic and surfaces it to the
//! parent's `recv'` as `RecvOutcome::Lost[LociDiedError::Panic{failure}]`.
//! `failure` is `Some(Failure)` — the substrate's
//! `failure_value_from_assertion_payload` preserves the original assertion's
//! structured `message`/`actual`/`expected` across the fork boundary, exactly
//! as if the assertion had fired in-process. The fixture reads all three
//! straight off that Failure record (Failure/message, Failure/actual,
//! Failure/expected) — no stderr EDN, no string re-parse.
//!
//! Symmetry: the slice-2 thread cascade proves the same chain shape
//! reaches the caller through crossbeam channels (zero-copy).
//! Slice 3 proves the same shape reaches the caller through kernel
//! pipes. The user-visible death report (LociDiedError::Panic carrying
//! a structured Failure) is identical regardless of tier.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Outer uses :my::compute; inner uses canonical nil main.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn hermetic_assertion_failure_preserves_actual_and_expected() {
    // Inner program: `(assert-eq 1 2)` triggers the structured
    // assertion-failed! panic in a forked process. Outer matches the
    // peer's death off the primed wire (recv' → Lost →
    // LociDiedError::Panic → its Some(Failure) payload) and emits the
    // rendered (message, actual, expected) triple so the Rust caller
    // can assert it.
    //
    // The structured AssertionPayload — message="assert-eq failed",
    // actual="1", expected="2" — is preserved verbatim across the fork
    // boundary in Panic.failure; the fixture reads all three straight
    // off the Failure record. (The old run-hermetic path stitched them
    // back from stderr EDN into a RunResult; the primed wire delivers
    // the same structured Failure as the peer's own Lost cause.)
    //
    // Arc 170 slice 1f-ζ: outer uses :my::compute; inner uses canonical nil main.
    let result = call_beside_value(file!(), ":my::compute").expect("compute should run");
    let lines: Vec<String> = match result {
        Value::Vec(items) => items
            .iter()
            .map(|v| match v {
                Value::String(s) => (**s).clone(),
                other => panic!("expected String, got {:?}", other),
            })
            .collect(),
        other => panic!("expected Vec<String>, got {:?}", other),
    };
    assert_eq!(
        lines.len(),
        3,
        "expected (message, actual, expected) triple; got {:?}",
        lines
    );
    assert_eq!(lines[0], "assert-eq failed", "message field");
    assert_eq!(lines[1], "1", "actual field — round-trip across fork");
    assert_eq!(lines[2], "2", "expected field — round-trip across fork");
}
