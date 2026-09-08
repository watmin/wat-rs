//! PROBE — a type ANNOTATION does not validate that its name names a type.
//!
//! Measured on the green tree at `8022e21b7` (5245/5245), `wat --check`:
//!
//! ```text
//!   (defn :user::f [s <- :usr::TotallyMadeUp] -> :usr::TotallyMadeUp s)   check=0  ⛔
//!   (defrecord :usr::Holder [thing <- :usr::AlsoMadeUp])                  check=0  ⛔
//! ```
//!
//! ⛔ A NAME THAT IS DECLARED NOWHERE IS A FIRST-CLASS OPAQUE NOMINAL TYPE, as long as it is
//! used consistently. Make it INCONSISTENT and the checker arbitrates between two types that
//! do not exist:
//!
//! ```text
//!   ":user::f: body produces :usr::TotallyMadeUp; signature declares :usr::AlsoMadeUp"
//! ```
//!
//! The mismatch is reported against a PHANTOM. The diagnostic is impeccable and the premise is
//! fiction — which is worse than silence, because it reads as the checker doing its job.
//!
//! ★ THE AUTHORITY NOW EXISTS. Arc 296 Stone Q built `:wat::runtime::is-type?` over the union
//! `TypeEnv::contains(name) ∨ is_builtin_primitive(stripped)` — the first predicate in the
//! substrate that can answer "is this a type?" against every store rather than one.
//!
//! ⛔⛔ THE TRAP THIS PROBE EXISTS TO PIN — `generic_type_param`. In
//! `(defn :user::id :- [T] [x <- :T] -> :T x)`, `:T` is a type PARAMETER. It is not in
//! `TypeEnv`, `is-type?` answers `false` for it, and it MUST STAY ACCEPTED. A wall that asks
//! `is-type?` and nothing else refuses every generic in the corpus. The bound parameters of the
//! enclosing form are a THIRD store the question must consult — which is this campaign's own
//! disease (`[[feedback_a_name_checked_against_a_partial_set]]`) reappearing inside its cure.
//!
//! ⚠ EVERY BAR IS A CONTROL RUN IN THE SAME TEST. Three fixtures are GREEN NOW and must stay
//! green; two are the subject. A hand-written `== 1` would also pass on a mis-aimed harness —
//! a control that must stay green cannot.
//!
//! The two subject tests are `#[ignore]`d and are UN-IGNORED BY THE STONE.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn check(case: &str) -> i32 {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc296_p1_annotation_names_a_type__{case}.wat"));
    assert!(path.exists(), "fixture missing: {}", path.display());
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg("--check")
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat --check");
    out.status.code().unwrap_or(-1)
}

/// CONTROL — a declared user type in param and return position. If this is not 0 the harness
/// is broken, not the subject.
#[test]
fn a_declared_type_is_accepted_in_both_positions() {
    assert_eq!(check("control_declared_type"), 0);
}

/// CONTROL — builtins and an instantiated generic.
#[test]
fn builtins_and_an_instantiated_generic_are_accepted() {
    assert_eq!(check("builtin_and_generic_instantiation"), 0);
}

/// ⛔ THE TRAP. A type PARAMETER is not a registered type and must stay accepted. This test is
/// the reason the stone cannot be "call is-type? at the annotation position" and stop there.
#[test]
fn a_bound_type_parameter_is_not_a_phantom() {
    assert_eq!(
        check("generic_type_param"),
        check("control_declared_type"),
        "`:T` is bound by `:- [T]` on the enclosing defn; it is not in TypeEnv and `is-type?` \
         answers false for it. A wall that consults only the type registry refuses every \
         generic in the corpus."
    );
}

/// THE SUBJECT — param + return position.
#[test]
#[ignore = "arc 296 P-1 — the annotation position does not yet validate its type name"]
fn a_phantom_type_name_is_refused_in_param_and_return_position() {
    assert_eq!(
        check("phantom_param_and_return"),
        1,
        "`:usr::TotallyMadeUp` is declared nowhere; a signature that names it is not a \
         signature over an opaque type, it is a signature over nothing"
    );
}

/// THE SUBJECT — aggregate field position.
#[test]
#[ignore = "arc 296 P-1 — the annotation position does not yet validate its type name"]
fn a_phantom_type_name_is_refused_in_a_field_annotation() {
    assert_eq!(check("phantom_record_field"), 1);
}
