//! Stone A of BRIEF-zero-is-not-a-wait.md — a wait of zero has no form.
//!
//! Relocated from `wat-scripts/scratch-pad/probe-zero-duration-disarms-at-process.wat`
//! because `tests/lint/wat_scripts_fixes_load.rs` type-checks every `.wat` under
//! wat-scripts/. Shape: `tests/kernel/probe_arc170_readln_outcome_must_use_wall.rs`
//! (StartupError::Check + `assert_check_error_present!`). The control remains
//! under wat-scripts/ — it still compiles, and it is what proves the wall
//! discriminates rather than rejecting everything.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn literal_zero_wait_has_no_form() {
    let err = startup_from_file("tests/kernel/probe_zero_is_not_a_wait.wat.bad")
        .expect_err("(:wat::time::Nanosecond 0) must fail check — zero is not a wait");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::time::Nanosecond"
            && reason.contains("positive")
            && (reason.contains("COMMITMENT") || reason.contains("MEASUREMENT"))
            && !reason.contains("non-negative")
            && !reason.contains("sign of the duration"));
}
