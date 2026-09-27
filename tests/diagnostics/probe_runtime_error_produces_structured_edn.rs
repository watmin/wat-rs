//! Probe — runtime-error path crosses the primed wire as a structured cause
//! (arc 170 slice 1i; arc 278 IPC de-prime).
//!
//! Path exercised: a forked child whose body errors at RUNTIME (not a Rust
//! panic). Integer division by zero — `(:wat::i64::/ 1 0)` — passes the
//! type-checker but fails at child runtime, flowing through `apply_function` as
//! `Err(RuntimeError)` (the Ok(Err(runtime)) arm of the forked child).
//!
//! IPC de-prime (arc 278): migrated off the non-prime `:wat::test::run-hermetic`
//! (fork + OS-pipe scrape → `:wat::kernel::RunResult`) onto the PRIMED peer wire —
//! the fixture now `spawn-program' :process` + `recv'` and returns the crash
//! cause's message as a plain `:wat::core::String`.
//!
//! Mapping: a runtime error in the child surfaces over the wire as `recv'` →
//! `Lost[cause]` with `cause = LociDiedError::RuntimeError` (NOT Panic — a runtime
//! error is not a Rust panic; same mapping wat_run_sandboxed's missing-main case
//! grounds). The retired capture model's contract ("Failure.message carries the
//! actual runtime error text, NOT 'forked program exited N'") is preserved: the
//! returned String is that runtime-error text.
//!
//! Row G (path-honesty): the child exercises ONLY the runtime-error exit path.
//! No AssertionPayload, no plain panic.

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
fn probe_runtime_error_produces_structured_edn() {
    // The child divides by zero → RuntimeError::DivisionByZero; the parent's
    // recv' sees Lost[RuntimeError]; the fixture returns RuntimeError.message.
    let msg = run_fn(":probe::runtime-err");

    eprintln!("===== probe_runtime_error_produces_structured_edn =====");
    eprintln!("RuntimeError.message: {:?}", msg);
    eprintln!("=======================================================");

    // Excursus 003 step 3b: `LociDiedError/message` derives from `failure.error.message`
    // for every variant, and for a `RuntimeError` that is now the one-line headline
    // directly — NOT the whole `to_wire_edn(re)` blob a prior version of this test
    // re-parsed as EDN (per the brief's own prediction: "for a RuntimeError it was the
    // whole EDN blob, and becomes the one-line headline"). `msg` IS the diagnostic text.
    // The exact-match assertion is still the guard against a wrong variant: each
    // `WRONG:<variant>` sentinel in the fixture is itself a plain, DIFFERENT string, so a
    // wrong branch firing still fails loudly here — it is neither this text nor the
    // retired plain-text fallback "forked program exited N".
    assert_eq!(
        msg, "division by zero",
        "a runtime error must surface over the primed wire as LociDiedError::RuntimeError \
         carrying the division-by-zero headline; got: {msg:?}"
    );
}
