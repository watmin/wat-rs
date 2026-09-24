//! Stone 255.25a — a variant is a type the moment its enum is (arc 255).
//!
//! WHY: 255.22's edge wall (`EdgeFreeTypeName`) runs when an `extend-type` registers and asks
//! `is_known_type` of every name in the edge. An enum's variant singletons (`E.V`) were
//! registered only LATER, by a separate whole-env `register_variant_types` pass, so a variant
//! named in an edge was "neither declared nor a known type" while an empty `defstruct` in the
//! same position was known. Measured before this stone (binary from `7b0cbcc20`): the variant
//! fixture rc=3 `EdgeFreeTypeName :probe::Tr.Wi`; the struct fixture rc=0 `"hello"`.
//!
//! THE FIX: one act — `TypeEnv::insert_enum_with_variants` registers the enum AND its variant
//! singletons, at both doors that insert a `TypeDef::Enum` (`register_validated`,
//! `register_builtin`). The separate pass is retired.
//!
//! Rows (fixtures beside this file, `probe_arc255_25a_a_variant_is_a_type_when_its_enum_is_*`):
//! - ⭐ `variant` — a Pure enum's variant in an `extend-type` target: checks, runs, prints
//!   `"hello"` (pre-stone: refused, `EdgeFreeTypeName`).
//! - `struct` — the control, the marker an empty `defstruct`: checks, runs, prints (both).
//! - `struct_declared_after` — the struct declared AFTER the edge naming it: refused at
//!   registration (both binaries). Declaration order matters for structs today.
//! - `variant_declared_after` — the enum declared AFTER the edge naming its variant: refused
//!   exactly as the struct is (matches the struct behaviour).
//! - `undeclared_variant` — `:probe::Tr.Nope`, a variant the enum does not declare: refused.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use wat::freeze::{startup_from_file, StartupError};
use wat::types::error::{TypeError, TypeErrorKind};

const DIR: &str = "tests/types/probe_arc255_25a_a_variant_is_a_type_when_its_enum_is";

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

fn registration_refusal(suffix: &str) -> TypeError {
    let path = format!("{DIR}_{suffix}.wat.bad");
    let err = startup_from_file(&path).expect_err(&format!("{path} must be refused"));
    let StartupError::Type(e) = err else {
        panic!("{path}: expected a registration-time type error, got {err:?}");
    };
    e
}

fn refused_as_free_target(suffix: &str, want: &str) {
    match registration_refusal(suffix).kind() {
        TypeErrorKind::EdgeFreeTypeName { name, slot, .. } => {
            assert_eq!(name, want);
            assert_eq!(slot, "target");
        }
        other => panic!("{suffix}: expected EdgeFreeTypeName, got {other:?}"),
    }
}

#[test]
fn a_variant_in_an_edge_target_checks_and_runs() {
    assert_eq!(runs_and_prints("variant"), vec!["\"hello\""]);
}

#[test]
fn a_struct_in_an_edge_target_checks_and_runs() {
    assert_eq!(runs_and_prints("struct"), vec!["\"hello\""]);
}

#[test]
fn a_struct_declared_after_the_edge_is_refused() {
    refused_as_free_target("struct_declared_after", ":probe::Wi");
}

#[test]
fn a_variant_of_an_enum_declared_after_the_edge_is_refused_as_a_struct_is() {
    refused_as_free_target("variant_declared_after", ":probe::Tr.Wi");
}

#[test]
fn an_undeclared_variant_in_an_edge_target_is_refused() {
    refused_as_free_target("undeclared_variant", ":probe::Tr.Nope");
}
