//! PROBE — `is-type?` and the annotation wall give DIFFERENT answers to "is this a type?"
//!
//! Arc 296 Stone Q shipped `:wat::runtime::is-type?` as *"one authority over three mechanisms."*
//! Stone P-1's RELAND-1 measured the question at FOUR stores and widened the annotation wall to
//! ask all four. `is-type?` was not widened. Measured on the green tree at `f302681e7`:
//!
//! ```text
//!   (:wat::core::use! :rust::sqlite::Connection)
//!   (:wat::runtime::is-type? :rust::sqlite::Connection)   -> false   ⛔ the wall ACCEPTS it
//!   (:wat::runtime::is-type? :wat::spawn::Spawned)        -> false   ⛔ a live derive marker
//! ```
//!
//! ⛔ **TWO CONTRADICTORY ANSWERS TO ONE QUESTION NOW SHIP IN ONE BINARY.** P-2 (a variant is a
//! type) rests on this verb, so this is P-2's prerequisite, not a tidy-up.
//!
//! ## ★★★ And the store it cannot see is itself split into two HAND-CURATED halves
//!
//! ```text
//!   is-type? :rust::crossbeam_channel::Sender   -> true    typed into a list in src/types.rs
//!   is-type? :rust::sqlite::Connection          -> false   use!'d in the same file
//! ```
//!
//! The registry answers differently for two `:rust::*` types whose only difference is whether
//! someone transcribed one into `register_builtin_types`' Group 3 — a list whose own comment
//! sources it to *"a rider's convergence on branch `arc109-type-refs-parked`."* The two
//! populations are DISJOINT, and neither mechanism knows the other exists:
//!
//! ```text
//!   hand-list ONLY   :rust::crossbeam_channel::{Sender,Receiver}   annotated in the stdlib,
//!                                                                  never use!'d, and NOT in
//!                                                                  RustDepsRegistry — a
//!                                                                  `use!` of it is REFUSED
//!   use!      ONLY   :rust::cache::Lru · :rust::sqlite::{Connection,ReadConnection}
//! ```
//!
//! ⚠ So the hand-list CANNOT simply be deleted: `handlist_control` below is the fixture that
//! says so, and it must stay green. Why crossbeam is absent from `RustDepsRegistry` is
//! **unmeasured** — do not theorise from this file.
//!
//! ## The isolation
//!
//! `use_then_is_type` and `no_use_is_not_a_type` are THE SAME NAME in two programs, differing
//! only by the `use!` line. `use!` is a PER-PROGRAM declaration — a program that did not declare
//! the type does not have it, exactly as `resolve/walk.rs`'s coverage rule holds for call heads.
//! A fix that seeds membership from the BUILD-TIME registry instead of from `use!` turns both
//! true, and the detector catches it. That pair is the whole reason this probe is two fixtures
//! and not one.
//!
//! The subject test is `#[ignore]`d and is UN-IGNORED BY THE STONE.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Runs the fixture and returns its trimmed stdout.
fn run(case: &str) -> String {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc296_p2prereq_is_type_asks_the_same_union__{case}.wat"));
    assert!(path.exists(), "fixture missing: {}", path.display());
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    assert!(
        out.status.success(),
        "fixture {case} did not run cleanly: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// CONTROL — the hand-listed half of the `:rust::*` population. TRUE now, and this stone does
/// NOT remove its row: `:rust::crossbeam_channel::Sender` is not in `RustDepsRegistry`, so it
/// cannot be `use!`d and has no other way to be known.
#[test]
fn a_hand_listed_rust_type_stays_a_type() {
    assert_eq!(run("handlist_control"), "true");
}

/// ⛔ THE OVER-REACH DETECTOR. The same name as the subject, in a program with no `use!`.
/// `use!` is per-program; membership seeded from the build-time registry would turn this true
/// and would be wrong — the program never declared the type.
#[test]
fn a_rust_type_that_was_not_declared_is_not_a_type_here() {
    assert_eq!(
        run("no_use_is_not_a_type"),
        "false",
        "seeding membership from RustDepsRegistry rather than from this program's use! \
         declarations would flip this, and it must not flip"
    );
}

/// CONTROL — a `:rust::` name nothing knows, in neither store.
#[test]
fn a_phantom_rust_name_is_not_a_type() {
    assert_eq!(run("phantom_rust_name"), "false");
}

/// THE SUBJECT — the annotation wall accepts this name; `is-type?` denies it.
#[test]
#[ignore = "arc 296 P-2 prereq — is-type? does not ask the union the annotation wall asks"]
fn a_use_declared_rust_type_is_a_type() {
    assert_eq!(
        run("use_then_is_type"),
        "true",
        "the annotation wall accepts `:rust::sqlite::Connection` in this exact program via \
         UseDeclarations::covers; is-type? must not disagree with it"
    );
}
