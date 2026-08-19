//! Arc 259 S3.5a — `:wat::test::deftest'`: the test macro on the new substrate (the pipe model).
//!
//! With the thread-peer crash-reason IPC fix (S3.5a-0) in place, `recv'` now surfaces a crashed
//! peer's reason over the pipe — so `deftest'` is PURE user surface: `spawn-program'` + `recv'`.
//! The body runs in a self-peer and `send'`s a completion signal on success; a failing assertion
//! crashes the peer; `recv'` delivers the reason. NO outcome-capture side-channel, no internal
//! forms, no test privilege — the test harness dogfoods exactly what users use.
//!
//! CONTRACT (pass-or-raise; the test runner's `Ok(Ok)` / `Ok(Err)` arms, test_runner.rs:297-330):
//!   - a PASSING `deftest'` fn RETURNS a clean `RunResult` (failure=None) → runner reports pass,
//!   - a FAILING `deftest'` fn RAISES, the assertion message in the raise → runner reports it.
//!
//! RED at HEAD: `:wat::test::deftest'` does not exist (unknown macro → startup fails).
//!
//! Run SERIALLY (spawns a thread):
//!   `cargo nextest run --release -E 'binary(kernel)' -F probe_arc259_deftest_prime`
//!
//! WAT fixtures: tests/kernel/probe_arc259_deftest_prime_{passing,failing}.wat

use wat::freeze::DeftestOutcome;
use wat::value::Value;


/// A PASSING `deftest'` — its fn RETURNS `:wat::kernel::RunResult::Passed`.
///
/// Arc 278 the vacuous-gate wall — this asserted only `apply_function(..).is_ok()`, i.e. "the
/// deftest evaluated", which a FAILING deftest also satisfies (a fired assertion is captured
/// into the returned RunResult, never raised). It now reads the VERDICT.
#[test]
fn deftest_prime_passing_returns() {
    let world = wat::freeze::startup_from_file("tests/kernel/probe_arc259_deftest_prime_passing.wat")
        .expect("startup should succeed (deftest' macro must exist + expand)");
    wat::freeze::deftest_verdict(&world, ":user::passing")
        .expect_passed("a passing deftest' must return RunResult::Passed");
}

/// A FAILING `deftest` — its fn RETURNS `RunResult::Failed` carrying the assertion message.
///
/// This test used to assert the OPPOSITE: `Ok(()) => panic!("a failing deftest' must RAISE")`,
/// against a hand-typed rust-debug golden of the raise. Both halves are superseded.
///
/// Arc 278 R55 (the no-hidden-failures LAW reaching its own VERIFIER) made the harness
/// value-based precisely because a raise UNWINDS PAST the reader: `deftest`/`run-thread` now
/// RETURN the verdict, never raise to signal it. So `Ok(...)` is the correct outcome and a
/// raise would be the bug — the assertion was inverted relative to the shipped contract. And
/// the golden compared `format!("{:?}", ..)` text, which arc 296 flipped to EDN.
///
/// Both are cured the same way as the arc-198 restriction goldens: read the VERDICT, assert
/// the STRUCTURE. This goes RED if a failing deftest stops reporting its failure, or reports
/// the wrong message — and does NOT go red when a rendering changes.
#[test]
fn deftest_prime_failing_returns_failed_with_message() {
    let world = wat::freeze::startup_from_file("tests/kernel/probe_arc259_deftest_prime_failing.wat")
        .expect("startup should succeed (the deftest macro must exist + expand)");
    match wat::freeze::deftest_verdict(&world, ":user::failing") {
        DeftestOutcome::Failed { failure } => {
            let msg = failure_message(&failure);
            assert_eq!(
                msg, "DEFTEST-FAIL-SENTINEL",
                "a failing deftest must surface its assertion message in RunResult::Failed"
            );
        }
        DeftestOutcome::Passed => {
            panic!("a failing deftest must return RunResult::Failed; got Passed")
        }
        DeftestOutcome::DidNotRun { error } => panic!(
            "a failing deftest must RETURN a verdict, not raise (arc 278 R55 — a raise \
             unwinds past the reader); got: {error:?}"
        ),
    }
}

/// A `:wat::kernel::Failure`'s field[0] is its `error` (a `:wat::core::Fault`), whose field[0]
/// is the message String. Mirrors `probe_arc278_eprintln_terminal.rs`.
fn failure_message(failure: &Value) -> String {
    let f = match failure {
        Value::Aggregate(f) => f,
        other => panic!("RunResult::Failed must carry a Failure record; got {other:?}"),
    };
    assert_eq!(
        f.class.as_ref(), "wat::kernel::Failure",
        "RunResult::Failed must carry a :wat::kernel::Failure; got class {:?}", f.class
    );
    match &f.fields[0] {
        Value::Aggregate(err) => match &err.fields[0] {
            Value::String(s) => (**s).clone(),
            other => panic!("Failure.error.message is not a String; got {other:?}"),
        },
        other => panic!("Failure.error (field[0]) is not an Aggregate; got {other:?}"),
    }
}
