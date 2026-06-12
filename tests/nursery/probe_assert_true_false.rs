//! `:wat::test::assert-true` / `assert-false` — the basic boolean assertions.
//!
//! Reach-stumble tool: a test reaches for `assert-true` before anything else, and
//! it was absent (only `assert-eq` / `assert-contains` / `assert-coincident`
//! existed). Made first-class in `wat/test.wat`. Each fires `assertion-failed!`
//! (failing the run) on the wrong bool, and is a no-op on the right one.
//!
//! Run: `cargo test --release -p wat --test nursery probe_assert_true_false`

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;

/// Run a `:user::main` whose body is `body`; Ok = the assertions passed, Err = an
/// assertion fired.
fn run_main(body: &str) -> Result<(), String> {
    let src = format!(
        "(:wat::core::defn :user::main [] -> :wat::core::nil \
           (:wat::core::do {body} nil))"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {e:?}"))?;
    invoke_user_main(&world, vec![])
        .map(|_| ())
        .map_err(|e| format!("eval: {e:?}"))
}

#[test]
fn assert_true_passes_on_true() {
    assert!(run_main("(:wat::test::assert-true (:wat::core::= 1 1))").is_ok());
}

/// Outside a sandbox an assertion fires by PANIC (the sandbox would catch it into
/// a RunResult; raw `invoke_user_main` lets it propagate — "an assertion firing
/// outside a harness IS a program error", per assertion.rs).
#[test]
#[should_panic]
fn assert_true_panics_on_false() {
    let _ = run_main("(:wat::test::assert-true (:wat::core::= 1 2))");
}

#[test]
fn assert_false_passes_on_false() {
    assert!(run_main("(:wat::test::assert-false (:wat::core::= 1 2))").is_ok());
}

#[test]
#[should_panic]
fn assert_false_panics_on_true() {
    let _ = run_main("(:wat::test::assert-false (:wat::core::= 1 1))");
}
