//! Stone 255.15 — infer a function's type variable from the `extend-type` binding of the
//! implementor it is given (arc 255, step 1 of the locus narrow waist).
//!
//! WHY: `(defn start :- [T] [loc <- (Loc :- [T])] -> :T …)` refused `Th` even though `Th` does
//! `extend-type Th (Loc :- [Shared])`; the CONCRETE parameter `(Loc :- [Shared])` already worked.
//! `assignable`'s arc-170 Gap-1 arm decided only by an exact-string edge, and an expectation that
//! still carries a unification variable renders `(Loc :- [_])`, which no edge string can equal.
//! The cure keeps each parametric edge's target as the `TypeExpr` it was parsed from
//! (`TypeEnv::register_parametric_extension`) and UNIFIES the target's args with the expected
//! ones on a cloned substitution, committing only a UNIQUE solution.
//!
//! This cure runs in the PERMISSIVE direction, so the refusals below are the stone:
//! - `wrong_transport` — T inferred `Shared`, signature declares `Wire` → `ReturnTypeMismatch`
//!   (refused for the RIGHT reason; before the cure it was refused by the gap itself).
//! - `conflicting_bindings` — one T given `Th` (Shared) and `Pr` (Wire) → refused at param #2.
//! - `non_implementor` — no edge at all → `TypeMismatch`.
//! - `ambiguous` — one type extends `Loc` at TWO instantiations → refused, never picked.
//!   (Proven load-bearing under 255.15: a mutation committing the first of several solutions
//!   accepted it.) Stone 255.16 refuses it earlier, at REGISTRATION —
//!   `TypeErrorKind::ParametricSurfaceBoundTwice` — a type binds a parametric surface once.
//! - `nested_mismatch` — `(Loc :- [(Vector :- [U])])` given `Th` → refused (Shared ≠ Vector).
//! - `derive_chain` — a type that only `derive`s an implementor is NOT inferred (direct edges
//!   only; the concrete arm accepts it and it then fails at run — a hole this stone does not widen).
//! - `concrete_ok` / `concrete_wrong` — the concrete A/B, unchanged.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};
use wat::types::error::TypeErrorKind;

const DIR: &str = "tests/types/probe_arc255_15_infer_transport";

fn check_errors(suffix: &str) -> Vec<wat::check::error::CheckError> {
    let path = format!("{DIR}_{suffix}.wat.bad");
    let err = startup_from_file(&path).expect_err(&format!("{path} must fail check"));
    let StartupError::Check(CheckErrors(errs)) = err else {
        panic!("{path}: expected a type-check error, got {err:?}");
    };
    errs
}

#[test]
fn generic_start_binds_t_from_the_implementor_and_runs() {
    let rel = format!("{DIR}_ok.wat");
    startup_from_file(&rel).expect(
        "one generic `start` must accept both implementors, binding T from each one's extend-type",
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .current_dir(&root)
        .arg(&rel)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "run must succeed; stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Th's transport (Shared :a 1), Pr's transport (Wire :b "w"), and the two-param call.
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["\"1\"", "\"w\"", "\"1\""], "stdout:\n{stdout}");
}

#[test]
fn wrong_transport_is_a_return_type_mismatch() {
    let errs = check_errors("wrong_transport");
    assert_eq!(errs.len(), 1, "{errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":probe::f" && expected == ":probe::Wire" && got == ":probe::Shared");
}

// rune:lint(no-inlined-wat) — every `"(:probe::Loc :- [...])"` literal in this file is golden
// COMPARISON text for a rendered type field (a TypeMismatch's `expected`; 255.16's
// ParametricSurfaceBoundTwice `existing`/`second`) — the checker renders a parametric type as a
// real `(Head :- [args])` form, so the reader happens to parse it; nothing
// here builds, evals, or runs a wat program from a string — every program is a `.wat` fixture.
#[test]
fn conflicting_bindings_of_one_t_are_refused() {
    let errs = check_errors("conflicting_bindings");
    assert_eq!(errs.len(), 1, "{errs:?}");
    // Param #1 (Th) binds T := Shared; param #2 (Pr, which binds Wire) is then refused.
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { param, expected, got, .. }
            if param == "#2" && expected == "(:probe::Loc :- [:probe::Shared])" && got == ":probe::Pr");
}

#[test]
fn a_non_implementor_is_refused() {
    let errs = check_errors("non_implementor");
    assert_eq!(errs.len(), 1, "{errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { param, got, .. }
            if param == "#1" && got == ":probe::NotLoc");
}

/// Stone 255.16 — this fixture is now refused EARLIER, at registration: a type binds a
/// parametric surface once, so `Both`'s second (bodiless) binding `(Loc :- [Wire])` is a
/// `ParametricSurfaceBoundTwice` naming the type and both bindings — the ambiguity inference
/// used to refuse can no longer be declared. Kept as 255.16's own negative row.
#[test]
fn an_ambiguous_implementor_is_refused_not_picked() {
    let path = format!("{DIR}_ambiguous.wat.bad");
    let err = startup_from_file(&path).expect_err(&format!("{path} must be refused"));
    let StartupError::Type(e) = err else {
        panic!("{path}: expected a registration-time type error, got {err:?}");
    };
    assert!(
        matches!(e.kind(), TypeErrorKind::ParametricSurfaceBoundTwice { ty, existing, second }
            if ty == ":probe::Both"
                && existing == "(:probe::Loc :- [:probe::Shared])"
                && second == "(:probe::Loc :- [:probe::Wire])"),
        "{e:?}"
    );
}

#[test]
fn a_binding_that_does_not_unify_with_a_nested_expectation_is_refused() {
    let errs = check_errors("nested_mismatch");
    assert_eq!(errs.len(), 1, "{errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { param, got, .. }
            if param == "#1" && got == ":probe::Th");
}

#[test]
fn a_derive_chain_is_not_inferred() {
    let errs = check_errors("derive_chain");
    assert_eq!(errs.len(), 1, "{errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { param, got, .. }
            if param == "#1" && got == ":probe::Th2");
}

#[test]
fn the_concrete_parameter_still_accepts_its_implementor() {
    startup_from_file(&format!("{DIR}_concrete_ok.wat"))
        .expect("the concrete (Loc :- [Shared]) parameter accepts Th, as before");
}

#[test]
fn the_concrete_parameter_still_refuses_the_other_transport() {
    let errs = check_errors("concrete_wrong");
    assert_eq!(errs.len(), 1, "{errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { param, expected, got, .. }
            if param == "#1" && expected == "(:probe::Loc :- [:probe::Shared])" && got == ":probe::Pr");
}
