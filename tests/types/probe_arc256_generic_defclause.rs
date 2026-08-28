//! Strike (examinare probe) — Arc 256 generic defclause (parametric clause dispatch).
//!
//! Ports 251.7's implicit-generics recipe to the `defclause`/`ClauseSet` entity. The RUNTIME
//! already dispatches generic clauses (`value_matches_type_pattern` treats Uppercase-bare Paths
//! as match-anything); the CHECKER's call-site dispatch (`check.rs:5362`) does NOT instantiate
//! clause type-vars, so `assignable(i64, :T)` fails → the clause never matches → spurious reject.
//!
//! Asserts the DESIRED end-state, so the build turns these green WITHOUT editing the probe.
//! At HEAD, the call-site rows (C02/C04/C05) are expected RED; C01 (def-only) + C03 (guard)
//! reveal whether the gap is call-side-only.
//!
//! C01 def-only generic defclause checks      (should already pass at HEAD → gap is call-side)
//! C02 generic clause call checks             (RED at HEAD → GREEN after build) — LOAD-BEARING
//! C03 ill-typed generic call is REJECTED     (guard — rejected at HEAD and after)
//! C04 two distinct instantiations both check  (RED at HEAD → GREEN; distinct fresh vars)
//! C05 parametric (Vector T) clause checks     (RED at HEAD → GREEN; container-head + inner var)
//!
//! Run: `cargo test --release --test probe_arc256_generic_defclause`

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_from_file, StartupError};

fn check_file(path: &str) -> Result<(), StartupError> {
    startup_from_file(path).map(|_| ())
}

#[test]
fn c01_def_only_generic_defclause_checks() {
    // Definition with bare type-vars, not called. Reveals whether the gap is call-side only.
    let r = check_file("tests/types/probe_arc256_generic_defclause_c01.wat");
    assert!(r.is_ok(), "generic defclause DEFINITION should check (gap is call-side). Got: {r:?}");
}

#[test]
fn c02_generic_clause_call_checks() {
    // LOAD-BEARING. (firstof 1 2) — both i64 → T:=i64 → returns i64. RED at HEAD.
    let r = check_file("tests/types/probe_arc256_generic_defclause_c02.wat");
    assert!(r.is_ok(), "generic defclause CALL must check after 256 (T:=i64). Got: {r:?}");
}

#[test]
fn c03_illtyped_generic_call_rejected() {
    // Guard: T:=i64 from a=1, then b="two" (String) must fail to unify → REJECT.
    // Bypasses `check_file` — the discriminant needs to match the inner `CheckErrorKind`
    // structurally via `assert_startup_error!`, not just discard it into `Result<(), _>`.
    let r = startup_from_file("tests/types/probe_arc256_generic_defclause_c03.wat.bad");
    wat::assert_startup_error!(r, check
        CheckErrorKind::NoMatchingClauseAtCallSite { name, called_arity, called_arg_types, .. }
            if name == ":user::firstof"
            && *called_arity == 2
            && called_arg_types.as_slice() == [":wat::core::i64".to_string(), ":wat::core::String".to_string()]
    );
}

#[test]
fn c04_two_instantiations_both_check() {
    let r = check_file("tests/types/probe_arc256_generic_defclause_c04.wat");
    assert!(r.is_ok(), "generic defclause must check at both i64 and bool call sites. Got: {r:?}");
}

#[test]
fn c05_parametric_container_clause_checks() {
    // A clause over a parametric container head (Vector T). Dispatch matches by head; inner var
    // is a wildcard. `(len-of (vector ...))` returns i64.
    let r = check_file("tests/types/probe_arc256_generic_defclause_c05.wat");
    assert!(r.is_ok(), "parametric-container generic clause must check (head match + inner var). Got: {r:?}");
}
