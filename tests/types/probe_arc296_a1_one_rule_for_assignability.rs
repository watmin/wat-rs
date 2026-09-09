//! PROBE — two positions give OPPOSITE answers about the same pair of types.
//!
//! Measured on the pushed green tree `4d1720f9f`, with plain records and one `derive` edge —
//! **no variant machinery involved at all**:
//!
//! ```text
//!   (:usr::Child 1) passed to [p <- :usr::Parent]     check=0   the parameter SUBSUMES
//!   (if b (:usr::Child 1) (:usr::Parent 2))           check=1   ⛔
//!     ":wat::core::if: parameter else-branch expects :usr::Child; got :usr::Parent"
//! ```
//!
//! `assignable` (`check.rs:16960+`) walks `is_subtype`. `infer_if` (`check.rs:8143`) states its own
//! rule outright — *"The form's type is unify(then, else)"* — and `unify` knows nothing about the
//! subtype graph. `infer_send_prime` (`check.rs:11619`) unifies its payload the same way.
//!
//! ★ Note the diagnostic is not even SYMMETRIC: `if` takes the THEN branch as authoritative and
//! reports the ELSE branch as the offender. Swap the branches and the message swaps with them.
//!
//! ## This is the same disease as P-1/P-2prereq/P-3, one layer down
//!
//! Those three closed *"is this a type?"* being answered differently by different stores. This is
//! *"can this value go here?"* being answered differently by different POSITIONS. It is a defect
//! today, independent of anything else, and arc 209 already ships depending on the subsuming half.
//!
//! ## ⛔ THE RULED CONTRACT — builder, 2026-09-08
//!
//! > **unify when either side is still a type VARIABLE; subsume when both are concrete.**
//!
//! Unification is not only a check — it is how a type variable gets SOLVED from the branches.
//! `if_still_solves_a_type_var` is the row that guards that, and it is the reason the rule is
//! conditional rather than "subsume everywhere": the unconditional version passes every other row
//! in this file while silently weakening inference across the corpus.
//!
//! ⚠ The ruling's own admitted cost: a partially-concrete pair (`Box<T>` with `T` unsolved, meeting
//! `Box.Full<i64>`) takes the unify path and does NOT widen. An annotation is the fix. Named here so
//! nobody later reads it as a bug.
//!
//! The subject test is `#[ignore]`d and is UN-IGNORED BY THE STONE.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn check(case: &str) -> i32 {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc296_A1_one_rule_for_assignability__{case}.wat"));
    assert!(p.exists(), "fixture missing: {}", p.display());
    Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg("--check").arg(&p)
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .output().expect("spawn wat --check")
        .status.code().unwrap_or(-1)
}

/// CONTROL — the parameter position already subsumes. If this is not 0 the harness is broken.
#[test]
fn a_parameter_position_subsumes_a_subtype() {
    assert_eq!(check("parameter_subsumes"), 0);
}

/// ⛔ OVER-REACH DETECTOR — two unrelated concrete types must STILL be refused. A stone that
/// made `if` accept anything would satisfy the subject and fail here.
#[test]
fn if_still_refuses_unrelated_branch_types() {
    assert_eq!(check("if_branches_unrelated"), 1);
}

/// ⛔⛔ THE CAPABILITY GUARD. The ruled contract exists to protect exactly this: the `None` branch
/// leaves `Option`'s `T` unsolved and the `Some` branch pins it. UNIFICATION solves it. An
/// unconditional "subsume everywhere" keeps every other row in this file green and breaks this one
/// — or worse, leaves it green while the variable goes unsolved somewhere no test looks.
#[test]
fn if_still_solves_a_type_variable_from_its_branches() {
    assert_eq!(
        check("if_still_solves_a_type_var"),
        0,
        "unify is how a type variable is SOLVED from the branches; subsuming unconditionally \
         weakens inference corpus-wide while every other bar stays green"
    );
}

/// SUBJECT — the same pair of types the parameter position already accepts.
#[test]
#[ignore = "arc 296 A-1 — infer_if unifies; it does not consult the subtype graph"]
fn if_subsumes_subtype_related_concrete_branches() {
    assert_eq!(
        check("if_branches_subtype_related"),
        0,
        "`assignable` says :usr::Child <: :usr::Parent at a parameter; `if` must not say otherwise \
         about the same two types in the same program"
    );
}
