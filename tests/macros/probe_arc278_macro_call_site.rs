//! Arc 278 §4 — `:wat::kernel::macro-call-site` expand-time verb probe (the `log`-macro
//! enabling primitive). See DESIGN-telemetry-caller-and-capacity.md §4.
//!
//! `macro-call-site`, used inside a macro body, returns the macro INVOCATION's own call-site
//! Frame (from the engine's `call_site_span`) — NOT the runtime stack. The fixture invokes a
//! probe macro on two ADJACENT source lines and asserts the captured Frame lines differ by
//! exactly 1, proving PER-INVOCATION capture: a constant enclosing-fn frame would give
//! difference 0. This is the exact property `emitted-from`-at-the-log-line needs.
//!
//! RED at HEAD: `:wat::kernel::macro-call-site` is not on `is_pure_total` → the default-deny
//!   purity gate refuses it in the macro body at expand → startup fails.
//! GREEN after: startup succeeds; the deftest' fn RETURNS (adjacent invocations differ by 1).
//!
//! WAT fixture: tests/macros/probe_arc278_macro_call_site.wat

use wat::freeze::{deftest_verdict, startup_from_file, DeftestOutcome};

/// Arc 278 the vacuous-gate wall — this was `apply_function(..)` + `Ok(_) => Ok(())`, whose
/// `Ok` answers "did the deftest EVALUATE?" while the gate below read it as "did it PASS?".
/// A fired assertion is captured into the returned `:wat::kernel::RunResult`, not raised, so
/// every assert-eq in the fixture was decoration. `deftest_verdict` reads the verdict.
fn run_test_fn(path: &str, name: &str) -> DeftestOutcome {
    let world = startup_from_file(path)
        .expect("startup should succeed (:wat::kernel::macro-call-site must exist + be expand-legal)");
    deftest_verdict(&world, name)
}

/// `(:wat::kernel::macro-call-site)` captures each macro invocation's OWN source line — two
/// invocations on adjacent lines differ by exactly 1.
#[test]
fn macro_call_site_captures_invocation_line() {
    run_test_fn(
        "tests/macros/probe_arc278_macro_call_site.wat",
        ":user::macro-call-site-captures-invocation-line",
    )
    .expect_passed(
        "macro-call-site must capture each invocation's own line (adjacent calls differ by 1)",
    );
}
