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

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::Environment;

fn run_test_fn(path: &str, name: &str) -> Result<(), String> {
    let world = startup_from_file(path)
        .expect("startup should succeed (deftest-hermetic' macro must exist + expand)");
    let ast = wat::parse_one!(name).expect("parse test-fn call");
    match eval_in_frozen(&ast, &world, &Environment::new()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{e:?}")),
    }
}

/// A PASSING `deftest-hermetic'` — its fn RETURNS (a clean RunResult); eval is `Ok`.
#[test]
fn deftest_hermetic_prime_passing_returns() {
    let r = run_test_fn(
        "tests/kernel/probe_arc259_deftest_hermetic_prime_passing.wat",
        "(:user::passing)",
    );
    assert!(r.is_ok(), "a passing deftest-hermetic' must RETURN (not raise); got Err: {r:?}");
}

/// A FAILING `deftest-hermetic'` — its fn RAISES, the assertion message carried over the process
/// Err channel and surfaced by `recv'`. The runner's `Ok(Err)` arm reports exactly this.
#[test]
fn deftest_hermetic_prime_failing_raises_with_message() {
    let r = run_test_fn(
        "tests/kernel/probe_arc259_deftest_hermetic_prime_failing.wat",
        "(:user::failing)",
    );
    match r {
        Ok(()) => panic!("a failing deftest-hermetic' must RAISE; it returned Ok"),
        Err(text) => {
            assert!(
                text.contains("HERMETIC-FAIL-SENTINEL"), // rune:lint(loose-assert) — process crash error embeds machine-specific absolute path in process crash frames
                "the failing assertion's message must surface through deftest-hermetic' (pipe model, \
                 process Err channel); got: {text}"
            );
        }
    }
}
