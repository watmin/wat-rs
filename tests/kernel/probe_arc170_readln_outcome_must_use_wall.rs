//! Arc 170 #24 readln-must-use wall — the twin gate that completes the readln outcome wall.
//!
//! WHY: #24 made `:wat::kernel::readln` return a matchable `:wat::kernel::ReadlnOutcome<T>`
//! (`Datum [value] | Eof | Stopped`) instead of RAISING on end-of-input and on a process-wide
//! stop — it was the LAST IPC verb still raising, and a raise in a language with no try/catch
//! unwinds PAST the reader (R53 `VERBO MEO CAPTVS`).
//!
//! That closed the *raise* mask. It did NOT close the *swallow*: a `_`-bound or `do`-dropped
//! `ReadlnOutcome` compiled clean, silently discarding an Eof or a Stop. That is precisely the
//! half-state `recv'` sat in after R53 and before its own twin gate — value-faced but not
//! swallow-gated — and the send'-wall needed BOTH Phase 3a (`do`) and 3b (`let`-`_`) to close
//! it, because a wall with one door open is not a wall.
//!
//! `ReadlnOutcome` therefore joins `MUST_USE_PARAMETRIC_HEADS` (it is a `TypeExpr::Parametric`,
//! not a bare `Path` — the parametric head convention is a BARE FQDN, no leading colon). A
//! FACED readln has type `T` or a joined arm type, never `ReadlnOutcome<T>`, so this fires ONLY
//! on the raw dropped form — every one of the 77 migrated call sites stays legal.
//!
//! NOTE ON SCOPE, kept honest: after the #24 corpus migration this gate has ZERO offending sites
//! — it is a PRE-ARM that gates the first FUTURE caller who drops one, the same standing
//! `CloseOutcome`'s gate has. That is not vacuous: these two fixtures prove it CAN fire
//! (`NISI FRANGAS, NIHIL PROBAS` — a gate that cannot go red proves nothing).

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn discarded_readln_outcome_in_let_underscore_is_compile_error() {
    let err =
        startup_from_file("tests/kernel/probe_arc170_readln_outcome_must_use_wall.wat.bad")
            .expect_err("a discarded ReadlnOutcome bound to `_` in a `let` must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::let"
            && reason.contains(":wat::kernel::ReadlnOutcome")
            && reason.contains("must be faced"));
}

#[test]
fn discarded_readln_outcome_in_do_non_final_is_compile_error() {
    let err =
        startup_from_file("tests/kernel/probe_arc170_readln_outcome_must_use_wall_do.wat.bad")
            .expect_err("a discarded ReadlnOutcome in a `do` non-final position must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::do"
            && reason.contains(":wat::kernel::ReadlnOutcome")
            && reason.contains("must be faced"));
}

/// The gate's own honesty check: the remedy must name the READLN verb and ITS arms, not
/// another wall's. `push_must_use_error` picks the verb by the outcome type name, and its
/// fall-through is `send'` — so a missing arm would silently mis-teach the caller, telling
/// them to face `Sent/Closed/Lost` on a readln. The system educates the caller (R29
/// `RVINA ERVDIT`); a diagnostic that names the wrong verb is a lie in the teaching position.
#[test]
fn the_remedy_names_readln_and_its_own_arms() {
    let err =
        startup_from_file("tests/kernel/probe_arc170_readln_outcome_must_use_wall.wat.bad")
            .expect_err("fixture must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { reason, .. }
            if reason.contains("readln")
            && reason.contains("Datum/Eof/Stopped")
            && !reason.contains("Sent/Closed/Lost"));
}
