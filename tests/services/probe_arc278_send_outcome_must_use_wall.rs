//! Arc 278 send'-outcome wall Phase 3 — the MUST-USE FORCE.
//!
//! WHY: Phases 1-2 made `send'` return a matchable `:wat::kernel::SendOutcome` (never raises)
//! and faced all 183 pre-existing sites. But nothing yet FORCED facing — before this wall,
//! `(:wat::core::do (:wat::kernel::send' p m) nil)` compiled clean, silently dropping the
//! outcome (a swallow, worse than the old raise: no error at all). This wall makes a
//! must-use-typed value in a discard position (a `do` non-final expr) a located compile error,
//! so a future `send'` swallow is unrepresentable (R57 "unrepresentable > flagged"). A *faced*
//! send' (wrapped in a `match` naming Sent/Closed/Lost) has type `nil`, not `SendOutcome`, so
//! this fires ONLY on the raw unfaced form — before this strike the fixture below compiled;
//! after, it's a located `MalformedForm` naming the must-use type.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn discarded_send_outcome_in_do_non_final_is_compile_error() {
    let err = startup_from_file("tests/services/probe_arc278_send_outcome_must_use_wall.wat.bad")
        .expect_err("a discarded SendOutcome in a `do` non-final position must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::do"
            && reason.contains(":wat::kernel::SendOutcome")
            && reason.contains("must be faced"));
}
