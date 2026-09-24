//! Stone 255.16 — a type binds a parametric surface ONCE (arc 255).
//!
//! WHY: dispatch keys on the flat `<Type>/<method>`, so a type that `extend-type`s one parametric
//! surface at two instantiations (`(Loc :- [Shared])` with a body, `(Loc :- [Wire])` without)
//! served the `Shared` body wherever `Wire` was demanded — measured in WEIGH-STONE-255.15:
//! `--check` rc=0, run rc=2 "expected receiver of class `:probe::Wire`, got class
//! `:probe::Shared`". `TypeEnv::register_parametric_extension` — the one door every parametric
//! `extend-type` target reaches — now refuses the second binding (`ParametricSurfaceBoundTwice`),
//! naming the type and both bindings.
//!
//! Rows here (the bodied-first negative row is 255.15's `ambiguous` fixture, whose assertion
//! 255.16 updated in `probe_arc255_15_infer_transport.rs`):
//! - `bodiless_first` — order-independence: bodiless first, bodied second → refused the same way.
//! - `two_spellings` — the second binding spells the type as a symbol reference → still refused.
//! - `identical` — the same binding re-registered (bodied, then bodiless) → legal, runs.
//! - `different_types` — `Th` → Shared, `Pr` → Wire → legal, runs.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use wat::freeze::{startup_from_file, StartupError};
use wat::types::error::TypeErrorKind;

const DIR: &str = "tests/types/probe_arc255_16_one_surface_one_binding";

/// (type, existing, second) of the registration-time refusal.
fn refusal(suffix: &str) -> (String, String, String) {
    let path = format!("{DIR}_{suffix}.wat.bad");
    let err = startup_from_file(&path).expect_err(&format!("{path} must be refused"));
    let StartupError::Type(e) = err else {
        panic!("{path}: expected a registration-time type error, got {err:?}");
    };
    match e.kind() {
        TypeErrorKind::ParametricSurfaceBoundTwice { ty, existing, second } => {
            (ty.clone(), existing.clone(), second.clone())
        }
        other => panic!("{path}: expected ParametricSurfaceBoundTwice, got {other:?}"),
    }
}

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

// rune:lint(no-inlined-wat) — every `"(:probe::Loc :- [...])"` literal in this file is golden
// COMPARISON text for the refusal's rendered `existing`/`second` fields (a parametric type renders
// as a real `(Head :- [args])` form, so the reader happens to parse it); nothing here builds, evals,
// or runs a wat program from a string — every program is a `.wat` fixture.
#[test]
fn a_bodiless_first_binding_refuses_the_bodied_second() {
    let (ty, existing, second) = refusal("bodiless_first");
    assert_eq!(ty, ":probe::Both");
    assert_eq!(existing, "(:probe::Loc :- [:probe::Wire])");
    assert_eq!(second, "(:probe::Loc :- [:probe::Shared])");
}

#[test]
fn a_second_spelling_of_the_type_is_still_one_type() {
    let (_ty, existing, second) = refusal("two_spellings");
    assert_eq!(existing, "(:probe::Loc :- [:probe::Shared])");
    assert_eq!(second, "(:probe::Loc :- [:probe::Wire])");
}

#[test]
fn the_identical_binding_re_registered_is_legal() {
    assert_eq!(runs_and_prints("identical"), vec!["\"w\""]);
}

#[test]
fn different_types_bind_the_surface_differently() {
    assert_eq!(runs_and_prints("different_types"), vec!["\"1\"", "\"w\""]);
}
