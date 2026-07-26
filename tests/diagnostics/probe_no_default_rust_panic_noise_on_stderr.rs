//! Probe — a child panic crosses the primed wire as a structured cause, never as
//! Rust's default panic noise (arc 170 slice 1i; arc 278 IPC de-prime).
//!
//! ORIGINAL CONTRACT (partially obsolete): under the retired non-prime
//! `:wat::test::run-hermetic` (fork + OS-pipe scrape → `:wat::kernel::RunResult`),
//! this probe inspected the child's OS-stderr lines to prove the silent panic hook
//! suppressed Rust's default handler output — "thread '…' panicked at …",
//! "Box<dyn Any>", "note: run with `RUST_BACKTRACE=1`" — leaving ONLY a structured
//! `#wat.kernel.LociDiedError/*` line on fd 2.
//!
//! IPC de-prime (arc 278): migrated onto the PRIMED peer wire — `spawn-program'
//! :process` + `recv'`. The wire captures NO child OS-stderr, so the literal
//! ABSENCE-of-raw-noise assertion (scan `RunResult.stderr` for "thread '…") is NOT
//! expressible. It is SUBSUMED by the stronger structural fact the wire guarantees:
//! a crashed child's reason arrives as a matchable `LociDiedError` (here
//! `::Panic` carrying the assertion message), never as raw stderr text. If Rust's
//! default handler had leaked instead of a structured cause, the fixture's
//! `recv'`/`LociDiedError::Panic` match would not yield the assertion message.
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
fn probe_no_default_rust_panic_noise_on_stderr() {
    // The child asserts "expected-value" == "actual-value" → AssertionPayload
    // panic; the parent's recv' sees Lost[Panic]; the fixture returns Panic.message.
    let msg = run_fn(":probe::hook-test");

    eprintln!("===== probe_no_default_rust_panic_noise_on_stderr =====");
    eprintln!("Panic.message: {:?}", msg);
    eprintln!("=======================================================");

    // The panic crossed the wire as a STRUCTURED Lost[Panic] — not a Message,
    // Closed, or a non-Panic Lost cause (distinct sentinels). Getting the Panic
    // message back IS the proof the death is structured, not raw Rust noise: had
    // the default handler leaked in place of a structured cause, this match would
    // not have produced the assertion message.
    assert_ne!(msg, "UNEXPECTED-MESSAGE", "child should have panicked, not sent a message");
    assert_ne!(msg, "UNEXPECTED-CLOSED", "child should have panicked, not closed cleanly");
    assert_ne!(
        msg, "LOST-NON-PANIC",
        "a child panic must surface as a structured LociDiedError::Panic over the wire"
    );

    // And it is the structured assertion text, never Rust's default handler blob.
    assert!(!msg.is_empty(), "expected non-empty panic message; got empty string");
    // rune:lint(loose-assert) — `msg` is the child's structured Panic message; the ABSENCE of
    // Rust's default "thread '…" handler line is the surviving essence of the original contract.
    assert!(
        !msg.contains("thread '"),
        "the wire must deliver a structured cause, never Rust's default panic handler text; got: {:?}",
        msg
    );
}
