//! Arc 278 "caller.2" — telemetry `Log/emitted-from` acceptance gate.
//!
//! caller.2 flips the forgeable hand-typed `:caller` keyword to `emitted-from <-
//! :wat::kernel::Frame`, populated via the native `(:wat::kernel::call-site)` verb (caller.1,
//! tests/kernel/probe_arc278_call_site.rs). This probe constructs a `:wat::telemetry::Log`
//! with `:emitted-from (:wat::kernel::call-site)`, reads it back via
//! `(:wat::telemetry::Log/emitted-from log)`, and asserts the Frame's `file` is `Some` and names
//! this fixture. `file` (not `symbol`) is the robust gate — `symbol` is `None` inside an
//! anonymous fn body (a known arc-109 wart), while `file` is always populated.
//!
//! RED at HEAD: `unknown field :emitted-from for aggregate :wat::telemetry::Log` +
//! `unknown callee: :wat::telemetry::Log/emitted-from` (two type errors — startup fails).
//! GREEN after: startup succeeds; the deftest' fn RETURNS (not raises).
//!
//! WAT fixture: tests/services/probe_arc278_emitted_from.wat

use wat::freeze::{deftest_verdict, startup_from_file, DeftestOutcome};

/// Arc 278 the vacuous-gate wall — this was `apply_function(..)` + `Ok(_) => Ok(())`, whose
/// `Ok` answers "did the deftest EVALUATE?" while the gate below read it as "did it PASS?".
/// A fired assertion is captured into the returned `:wat::kernel::RunResult`, not raised, so
/// every assert-eq in the fixture was decoration. `deftest_verdict` reads the verdict.
fn run_test_fn(path: &str, name: &str) -> DeftestOutcome {
    let world = startup_from_file(path)
        .expect("startup should succeed (:wat::telemetry::Log/emitted-from must exist + type-check)");
    deftest_verdict(&world, name)
}

/// A `:wat::telemetry::Log` built with `:emitted-from (:wat::kernel::call-site)` round-trips a
/// `Some` file through `Log/emitted-from` -> `Frame/file`.
#[test]
fn emitted_from_round_trips_through_log() {
    run_test_fn(
        "tests/services/probe_arc278_emitted_from.wat",
        ":user::emitted-from-round-trips",
    )
    .expect_passed("Log/emitted-from must round-trip a Frame with Some file");
}
