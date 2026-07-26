//! Arc 278 "caller.1" — `(:wat::kernel::call-site)` native nullary verb probe.
//!
//! Verifies the verb returns the CALLER's `:wat::kernel::Frame` (file/line/symbol),
//! mirroring the mechanism `:wat::kernel::assertion-failed!` uses (src/assertion.rs) —
//! `snapshot_call_stack().first()` from inside a native verb IS the caller's frame
//! (native verbs push no `FrameGuard` of their own; only wat fn-calls do).
//!
//! RED at HEAD: `:wat::kernel::call-site` is unknown to the type checker → startup
//! fails (unresolved-verb error).
//!
//! GREEN after: startup succeeds; the deftest' fn RETURNS (not raises) — the returned
//! Frame's file/line/symbol are all `Some` and describe the caller.
//!
//! WAT fixture: tests/kernel/probe_arc278_call_site.wat

use wat::freeze::{deftest_verdict, startup_from_file, DeftestOutcome};

/// Arc 278 the vacuous-gate wall — this was `apply_function(..)` + `Ok(_) => Ok(())`, whose
/// `Ok` answers "did the deftest EVALUATE?" while the gate below read it as "did it PASS?".
/// A fired assertion is captured into the returned `:wat::kernel::RunResult`, not raised, so
/// every assert-eq in the fixture was decoration. `deftest_verdict` reads the verdict.
fn run_test_fn(path: &str, name: &str) -> DeftestOutcome {
    let world = startup_from_file(path)
        .expect("startup should succeed (:wat::kernel::call-site must exist + type-check)");
    deftest_verdict(&world, name)
}

/// `(:wat::kernel::call-site)` returns the caller's Frame — the deftest asserts the returned
/// file/line/symbol describe the caller (this fixture file, a positive line, and the
/// "probe::here" symbol).
#[test]
fn call_site_returns_caller_frame() {
    run_test_fn(
        "tests/kernel/probe_arc278_call_site.wat",
        ":user::call-site-returns-caller-frame",
    )
    .expect_passed("call-site's returned Frame must describe the caller");
}
