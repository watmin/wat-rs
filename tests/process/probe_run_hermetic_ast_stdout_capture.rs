//! Arc 170 slice 3 Gap K — spawn-process stdout-capture verification.
//!
//! Arc 278 IPC de-prime (MAP unit): migrated off the non-prime
//! `:wat::test::run-hermetic` capture model (fork + OS-pipe stdout scrape →
//! `:wat::kernel::RunResult`) onto the PRIMED peer wire (`spawn-program'`
//! (`:wat::spawn::process`) + `recv'`), the same shape `run-hermetic'` and the
//! already-migrated `wat_run_sandboxed_ast` fixture ride.
//!
//! ## Path exercised
//!
//! The fixture forks an inner `:user::main` via `(:wat::spawn::process)`. The
//! child's `(:wat::kernel::println "hello-from-probe")` writes a value that,
//! on the primed wire, crosses to the parent as a DECODED
//! `RecvOutcome::Message[m]` — `m` is the native String `"hello-from-probe"`,
//! NOT the EDN-quoted stdout scrape (`"\"hello-from-probe\""`) the old
//! `RunResult/stdout` read produced. `Lost[LociDiedError]` / `Closed` are never
//! swallowed (the fixture re-raises via `assertion-failed!`).

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// ─── Probe 1 — spawn-process child prints a value; parent recv's it decoded ─

/// `spawn-program' (:wat::spawn::process)` with a child that calls
/// `(:wat::kernel::println "hello-from-probe")`.
///
/// Verifies the child's printed value crosses the primed wire DECODED: the
/// parent receives the native String `"hello-from-probe"` (no EDN quotes),
/// which is exactly the value the retired stdout scrape captured (as EDN text).
///
/// Path: `spawn-program' (:wat::spawn::process)` + `recv'`.
#[test]
fn probe_run_hermetic_ast_child_stdout_captured() {
    // World loaded from co-located probe_run_hermetic_ast_stdout_capture.wat via call_beside_value.
    let result = call_beside_value(file!(), ":probe::ast::capture-stdout")
        .expect("probe::ast::capture-stdout should run without panicking");

    // Arc 278 IPC de-prime: the primed wire delivers the child's `println`'d
    // value as a DECODED message, so the parent receives the native String
    // "hello-from-probe" (the retired RunResult/stdout scrape captured the
    // EDN-quoted `"\"hello-from-probe\""`).
    let got = match result {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String message; got {:?}", other),
    };
    assert_eq!(
        got, "hello-from-probe",
        "expected the child's println'd value decoded off the wire (native String, no EDN quotes)"
    );
}
