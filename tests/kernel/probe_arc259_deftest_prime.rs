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

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::Environment;

/// Build a world from the fixture; eval the named test fn and return the eval Result's
/// text on Err (or `None` on Ok).
fn run_test_fn(path: &str, name: &str) -> Result<(), String> {
    let world = startup_from_file(path)
        .expect("startup should succeed (deftest' macro must exist + expand)");
    let ast = wat::parse_one!(name).expect("parse test-fn call");
    match eval_in_frozen(&ast, &world, &Environment::new()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{e:?}")),
    }
}

/// A PASSING `deftest'` — its fn RETURNS (a clean RunResult); eval is `Ok`.
#[test]
fn deftest_prime_passing_returns() {
    let r = run_test_fn(
        "tests/kernel/probe_arc259_deftest_prime_passing.wat",
        "(:user::passing)",
    );
    assert!(r.is_ok(), "a passing deftest' must RETURN (not raise); got Err: {r:?}");
}

/// A FAILING `deftest'` — its fn RAISES, and the raise carries the assertion message (surfaced
/// over the pipe by the S3.5a-0 IPC fix). The test runner's `Ok(Err)` arm reports exactly this.
#[test]
fn deftest_prime_failing_raises_with_message() {
    let r = run_test_fn(
        "tests/kernel/probe_arc259_deftest_prime_failing.wat",
        "(:user::failing)",
    );
    match r {
        Ok(()) => panic!("a failing deftest' must RAISE; it returned Ok"),
        Err(text) => assert!(
            text.contains("DEFTEST-FAIL-SENTINEL"),
            "the failing assertion's message must surface through deftest' (pipe model); got: {text}"
        ),
    }
}
