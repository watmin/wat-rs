//! Arc 278 recv'-must-use wall — the symmetric completion of the send'-wall.
//!
//! WHY: R53 made `recv'` return a matchable `:wat::kernel::RecvOutcome<O>` (never a raise
//! that flees past the reader). That closed the *raise* mask — but not the *swallow*:
//! `(:wat::core::let [_ (:wat::kernel::recv p)] …)` and `(:wat::core::do (:wat::kernel::recv p) …)`
//! silently DROPPED the outcome, hiding a `Lost`/`Closed` failure (the exact R55 harness sin,
//! which R55 patched at one site but never gated as a class). This wall adds `RecvOutcome`
//! to the must-use set (`is_must_use_type` gained a parametric-head arm — `RecvOutcome<O>` is
//! `TypeExpr::Parametric`, not a bare `Path`), so a dropped recv' outcome — literal recv' OR a
//! generated `:nature :Peer` client-method call, both `RecvOutcome<Response>` — is a located
//! compile error in BOTH discard doors. A *faced* recv' (matched over Message/Closed/Lost) has
//! type `O`, not `RecvOutcome<O>`, so this fires ONLY on the raw dropped form. Now both verbs
//! are symmetric: a hidden `recv'`/`send'` error is unrepresentable.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn discarded_recv_outcome_in_let_underscore_is_compile_error() {
    let err = startup_from_file("tests/services/probe_arc278_recv_outcome_must_use_wall.wat.bad")
        .expect_err("a discarded RecvOutcome bound to `_` in a `let` must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::let"
            && reason.contains(":wat::kernel::RecvOutcome")
            && reason.contains("must be faced"));
}

#[test]
fn discarded_recv_outcome_in_do_non_final_is_compile_error() {
    let err = startup_from_file("tests/services/probe_arc278_recv_outcome_must_use_wall_do.wat.bad")
        .expect_err("a discarded RecvOutcome in a `do` non-final position must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::do"
            && reason.contains(":wat::kernel::RecvOutcome")
            && reason.contains("must be faced"));
}
