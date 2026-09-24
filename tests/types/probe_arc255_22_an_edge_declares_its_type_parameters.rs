//! Stone 255.22 — an `extend-type` declares its type parameters (arc 255).
//!
//! WHY: `extend-type` had no binder. `(extend-type (Box :- [T]) (Greets :- [T]))` used a `T`
//! nothing declared; it was a parameter because of its SPELLING (`is_type_param_letter`, and the
//! `(Head :- [:T])`/`(Head :- [:Xt])` keys the checker guessed). Measured before this stone: the
//! builder's hello-world checked and ran with `T` and was refused with `Elem` — renaming a type
//! parameter changed the verdict.
//!
//! THE FORM: `(:wat::core::extend-type :- [P …] <child> <target> <methods…>)` — *for any `P`, a
//! `<child>` is a `<target>`* (Rust's `impl<T>`). The binder is what makes a name a parameter:
//! the checker pattern-matches the edge's CHILD against an actual type, binding exactly the
//! binder's names, and instantiates the TARGET under those bindings. Two walls at registration:
//! every binder name appears in the child (`EdgeParamAbsentFromChild`), and every name in the
//! child or target is a declared parameter or a known type (`EdgeFreeTypeName`).
//!
//! Rows (fixtures beside this file, `probe_arc255_22_an_edge_declares_its_type_parameters_*`):
//! - `hello_T` / ⭐ `hello_Elem` — the hello-world in both spellings: check, run, print.
//! - `hello_mixed` — surface spells `T`, edge spells `E`: runs; the method result is `String`.
//! - `seqable_Elem` — the four Seqable-shaped edges with the child spelled `Elem` under a surface
//!   spelled `T`: generic fn, concrete bound, and a direct method call all check and run.
//! - `lie_T` / `lie_Elem` / `lie_mixed` — the greeting added to an i64: refused, naming `String`.
//! - `param_absent_from_child` — a binder name the child does not carry: refused at registration.
//! - `free_letter` — the hello-world with no binder: `T` is free, refused at registration.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};
use wat::types::error::{TypeError, TypeErrorKind};

const DIR: &str = "tests/types/probe_arc255_22_an_edge_declares_its_type_parameters";

fn runs_and_prints(suffix: &str) -> Vec<String> {
    let rel = format!("{DIR}_{suffix}.wat");
    startup_from_file(&rel).unwrap_or_else(|e| panic!("{rel} must be accepted: {e:?}"));
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .current_dir(&root)
        .arg(&rel)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{rel}: run must succeed; stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout.lines().map(str::to_string).collect()
}

fn check_errors(suffix: &str) -> Vec<wat::check::error::CheckError> {
    let path = format!("{DIR}_{suffix}.wat.bad");
    let err = startup_from_file(&path).expect_err(&format!("{path} must fail check"));
    let StartupError::Check(CheckErrors(errs)) = err else {
        panic!("{path}: expected a type-check error, got {err:?}");
    };
    errs
}

fn registration_refusal(suffix: &str) -> TypeError {
    let path = format!("{DIR}_{suffix}.wat.bad");
    let err = startup_from_file(&path).expect_err(&format!("{path} must be refused"));
    let StartupError::Type(e) = err else {
        panic!("{path}: expected a registration-time type error, got {err:?}");
    };
    e
}

/// The lie: `(:wat::i64::+ 1 <greeting>)` — exactly one refusal, and it names `String`.
fn the_lie_is_refused(suffix: &str) {
    let errs = check_errors(suffix);
    assert_eq!(errs.len(), 1, "{suffix}: {errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { callee, expected, got, .. }
            if callee == ":wat::i64::+" && expected == ":wat::core::i64" && got == ":wat::core::String");
}

#[test]
fn the_hello_world_spelled_t_checks_and_runs() {
    assert_eq!(runs_and_prints("hello_T"), vec!["\"hello, world!\""]);
}

#[test]
fn the_hello_world_spelled_elem_checks_and_runs() {
    assert_eq!(runs_and_prints("hello_Elem"), vec!["\"hello, world!\""]);
}

#[test]
fn an_edge_spelling_its_parameter_unlike_the_surface_checks_and_runs() {
    assert_eq!(runs_and_prints("hello_mixed"), vec!["\"hello, world!\""]);
}

#[test]
fn the_seqable_edges_with_the_child_spelled_elem_check_and_run() {
    assert_eq!(runs_and_prints("seqable_Elem"), vec!["\"3,4,5,2,6,10,15,30,9\""]);
}

#[test]
fn the_lie_spelled_t_is_refused() {
    the_lie_is_refused("lie_T");
}

#[test]
fn the_lie_spelled_elem_is_refused() {
    the_lie_is_refused("lie_Elem");
}

#[test]
fn the_lie_through_a_differently_spelled_edge_is_refused_naming_string() {
    the_lie_is_refused("lie_mixed");
}

#[test]
fn a_binder_name_absent_from_the_child_is_refused() {
    match registration_refusal("param_absent_from_child").kind() {
        TypeErrorKind::EdgeParamAbsentFromChild { param, child, .. } => {
            assert_eq!(param, "T");
            assert_eq!(child, ":hello::Plain");
        }
        other => panic!("expected EdgeParamAbsentFromChild, got {other:?}"),
    }
}

#[test]
fn an_undeclared_free_letter_is_refused() {
    match registration_refusal("free_letter").kind() {
        TypeErrorKind::EdgeFreeTypeName { name, slot, .. } => {
            assert_eq!(name, ":T");
            assert_eq!(slot, "child");
        }
        other => panic!("expected EdgeFreeTypeName, got {other:?}"),
    }
}
