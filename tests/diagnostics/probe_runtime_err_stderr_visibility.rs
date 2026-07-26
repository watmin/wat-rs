//! Probe — a crashed child's reason crosses the primed wire STRUCTURALLY, not as
//! dropped stderr text (arc 278 IPC de-prime).
//!
//! ORIGINAL PURPOSE (now obsolete): under the retired non-prime
//! `:wat::test::run-hermetic` (fork + OS-pipe scrape → `:wat::kernel::RunResult`),
//! this probe surfaced `RunResult.stderr` to expose a harness lossiness bug — the
//! match fallback used ONLY join-result's exit-code chain and DISCARDED the drained
//! stderr-lines Vec, reporting a bare "forked program exited N". It asserted
//! nothing; it just surfaced data for gap analysis.
//!
//! IPC de-prime (arc 278): migrated onto the PRIMED peer wire — `spawn-program'
//! :process` + `recv'`. There is NO OS-stderr side-channel to drop: a crashed
//! child's reason crosses the wire STRUCTURALLY as the `recv'` Lost cause (a
//! `LociDiedError`). The "drop the stderr Vec" lossiness the probe hunted cannot
//! exist over the wire, so the probe's surveying purpose is retired. What it now
//! pins is the surviving contract: an assertion death arrives as a structured
//! `LociDiedError::Panic` carrying its message — not as raw, droppable text.
//!
//! Body: assert-eq with mismatched values → AssertionPayload panic → recv' →
//! Lost[Panic]; the fixture returns Panic.message as a plain `:wat::core::String`.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// Call a zero-arg compute fn in the co-located fixture and return its
/// `:wat::core::String` result (the crash cause's message).
fn run_fn(fn_name: &str) -> String {
    match call_beside_value(file!(), fn_name).expect("compute should run") {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

#[test]
fn probe_runtime_err_stderr_visibility() {
    // The child asserts "intentional" == "different" → AssertionPayload panic; the
    // parent's recv' sees Lost[Panic]; the fixture returns Panic.message.
    let msg = run_fn(":probe::structured");

    eprintln!("===== probe_runtime_err_stderr_visibility =====");
    eprintln!("Panic.message: {:?}", msg);
    eprintln!("================================================");

    // The assertion death crossed the wire as a structured Lost[Panic] — not a
    // Message, Closed, or a non-Panic Lost cause (distinct sentinels). This is the
    // surviving contract: the reason is delivered structurally, never a dropped
    // stderr blob or an exit-code-only fallback.
    assert_ne!(msg, "UNEXPECTED-MESSAGE", "child should have crashed, not sent a message");
    assert_ne!(msg, "UNEXPECTED-CLOSED", "child should have crashed, not closed cleanly");
    assert_ne!(
        msg, "LOST-NON-PANIC",
        "an assertion failure must surface as LociDiedError::Panic over the wire"
    );
    assert!(!msg.is_empty(), "expected non-empty assertion message; got empty string");
}
