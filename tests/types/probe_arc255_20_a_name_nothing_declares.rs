//! Stone 255.20 — a name nothing declares does not type-check (arc 255).
//!
//! WHY: in `check.rs` `infer_list`, a call head with no `TypeScheme` fell through to a silent
//! accept that returned a free type, so the checker passed programs the runtime then failed
//! with `UnknownFunction`. The 255.19 weigh reproduced it: `(:wat::spawn::Locus/bogus-xyz …)`,
//! a member the `Locus` surface does not declare, gave `--check` rc=0. A second arm of the same
//! class, `:wat::kernel::`/`:wat::std::`, returned a free type for every scheme-less head under
//! those prefixes, even before the registry-arity door. Both are gone. A scheme-less head
//! checks only when a registry row, a type, a `def` value or a surface member declares it;
//! anything else is `UnknownCallee`.
//!
//! Rows (each `.wat.bad` measured rc=0 on the pre-stone binary):
//! - `undeclared_surface_member`: the reproducer, `UnknownCallee` naming the head.
//! - `declared_surface_member`: positive twin, a real `Locus` member keeps checking.
//! - `kernel_registry_row_arity`: `:wat::kernel::peer-pid` (registry row, arity 1, no scheme)
//!   called with 3 args. It now reaches the arity door: `ArityMismatch`.
//! - `registry_row_checks`: positive, a scheme-less registry intrinsic
//!   (`:wat::linkedlist::length`) at its declared arity keeps checking.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

const DIR: &str = "tests/types/probe_arc255_20_a_name_nothing_declares";

fn check_errors(suffix: &str) -> Vec<wat::check::error::CheckError> {
    let path = format!("{DIR}_{suffix}.wat.bad");
    let err = startup_from_file(&path).expect_err(&format!("{path} must fail check"));
    let StartupError::Check(CheckErrors(errs)) = err else {
        panic!("{path}: expected a type-check error, got {err:?}");
    };
    errs
}

fn accepted(suffix: &str) {
    let path = format!("{DIR}_{suffix}.wat");
    startup_from_file(&path).unwrap_or_else(|e| panic!("{path} must be accepted: {e:?}"));
}

#[test]
fn an_undeclared_surface_member_is_an_unknown_callee() {
    let errs = check_errors("undeclared_surface_member");
    assert_eq!(errs.len(), 1, "{errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::UnknownCallee { callee } if callee == ":wat::spawn::Locus/bogus-xyz");
}

#[test]
fn a_declared_surface_member_keeps_checking() {
    accepted("declared_surface_member");
}

#[test]
fn a_scheme_less_kernel_registry_row_reaches_the_arity_door() {
    let errs = check_errors("kernel_registry_row_arity");
    assert_eq!(errs.len(), 1, "{errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::ArityMismatch { callee, expected, got }
            if callee == ":wat::kernel::peer-pid" && *expected == 1 && *got == 3);
}

#[test]
fn a_scheme_less_registry_row_at_its_arity_keeps_checking() {
    accepted("registry_row_checks");
}
