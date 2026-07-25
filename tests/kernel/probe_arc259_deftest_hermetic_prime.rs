//! Arc 259 S3.5a — `:wat::test::deftest-hermetic'`: the test macro on the PROCESS tier (pipe model).
//!
//! The forms sibling of `deftest'`. Same caller — `spawn-program'` + `recv'` — different body
//! PACKAGING: a thread shares memory so it ships a CLOSURE; a process/remote has SEPARATE memory
//! so it ships FORMS (program over the wire). "Separate memory" = same-host-process OR remote-host;
//! `deftest-hermetic'` (process) and the future `deftest-remote` share this one forms interface.
//!
//! The child runs the body as `:user::main` and `println`s a pass-marker (the process analog of the
//! thread's `send' self 0`); the parent `recv'`s it. A failing assertion crashes the child; the
//! reason travels over the process Err channel (fd 2) → `recv'` raises with it (the process tier
//! already surfaces crashes — it was the working model that exposed the thread gap).
//!
//! CONTRACT (pass-or-raise; test_runner.rs:297-330): passing → RETURNS a clean RunResult; failing →
//! RAISES with the message.
//!
//! RED at HEAD: `:wat::test::deftest-hermetic'` does not exist (unknown macro → startup fails).
//!
//! Run SERIALLY (forks a process):
//!   `cargo nextest run --release -E 'binary(kernel)' -F probe_arc259_deftest_hermetic_prime`
//!
//! WAT fixtures: tests/kernel/probe_arc259_deftest_hermetic_prime_{passing,failing}.wat

use wat::freeze::startup_from_file;
use wat::runtime::apply_function;
use wat::runtime::Value;

fn run_test_fn(path: &str, name: &str) -> Result<(), String> {
    let world = startup_from_file(path)
        .expect("startup should succeed (deftest-hermetic' macro must exist + expand)");
    let func = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("no {name} in {path:?}"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{e:?}")),
    }
}

/// Arc 278 recv'-wall: the deftest-hermetic' harness (`run-hermetic'`) now RETURNS a `RunResult`
/// VALUE — a failing test drops the crashed child's Lost cause into `RunResult.failure=Some(cause)`,
/// never a raise (a raise would unwind past the runner). Eval the test fn and hand back the returned
/// RunResult Value for the caller to inspect its `.failure` slot.
fn run_test_value(path: &str, name: &str) -> Value {
    let world = startup_from_file(path)
        .expect("startup should succeed (deftest-hermetic' macro must exist + expand)");
    let func = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("no {name} in {path:?}"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("run-hermetic' now RETURNS a RunResult VALUE; it must not raise: {e:?}"))
}

/// A PASSING `deftest-hermetic'` — its fn RETURNS (a clean RunResult); eval is `Ok`.
#[test]
fn deftest_hermetic_prime_passing_returns() {
    let r = run_test_fn(
        "tests/kernel/probe_arc259_deftest_hermetic_prime_passing.wat",
        ":user::passing",
    );
    assert!(r.is_ok(), "a passing deftest-hermetic' must RETURN (not raise); got Err: {r:?}");
}

/// A FAILING `deftest-hermetic'` — arc 278 recv'-wall: its fn RETURNS a `RunResult` whose `.failure`
/// slot is `Some(cause)` (the crashed child's Lost cause dropped straight in), NOT a raise. The
/// assertion message travels over the process Err channel and rides the Lost cause's `Failure`. We
/// assert the `.failure` slot IS `Some` (a failure) and that it carries the sentinel.
#[test]
fn deftest_hermetic_prime_failing_raises_with_message() {
    let result = run_test_value(
        "tests/kernel/probe_arc259_deftest_hermetic_prime_failing.wat",
        ":user::failing",
    );
    // RunResult field order (wat/test.wat): arc 278 wave 2d dropped stdout/stderr —
    // [failure <- Option<Failure>] is the sole field (index 0).
    let failure = match &result {
        Value::Aggregate(sv) if sv.class == "wat::kernel::RunResult" => match &sv.fields[0] {
            Value::Option(opt) => (**opt).clone(),
            other => panic!("RunResult.failure is not an Option; got {other:?}"),
        },
        other => panic!("expected a RunResult Aggregate; got {other:?}"),
    };
    let failure = failure.unwrap_or_else(|| {
        panic!(
            "a failing deftest-hermetic' must land its crash in RunResult.failure=Some (not None); \
             got a clean RunResult: {result:?}"
        )
    });
    let text = format!("{failure:?}");
    assert!(
        text.contains("HERMETIC-FAIL-SENTINEL"), // rune:lint(loose-assert) — process crash error embeds machine-specific absolute path in process crash frames
        "the failing assertion's message must surface through deftest-hermetic' (pipe model, process \
         Err channel) into RunResult.failure; got: {text}"
    );
}
