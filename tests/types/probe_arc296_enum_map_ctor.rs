//! PROBE — an enum variant is constructed by a MAP THAT NAMES ITS FIELDS, and only that way.
//!
//! Today a variant is named three ways in three places, and exactly one of them is positional:
//!
//! ```text
//!   WIRE       #probe/Box.Full {:payload 7}      map, named   (296 H)
//!   PATTERN    [:probe::Box::Full {:payload p}]  map, named   (the arm migration)
//!   CONSTRUCT  (:probe::Box::Full 7)             POSITIONAL   <- the last one left
//! ```
//!
//! `defrecord` already made this move: `(Pt 1 2)` is REFUSED, `(Pt :x 1 :y 2)` is the only way
//! in. Enum variants never did, so variant construction is the last positional constructor in
//! the language.
//!
//! ⚠ EVERY SLOT HERE IS TYPED, AND THAT IS THE WHOLE POINT OF THE PROBE. In an UNTYPED slot —
//! e.g. inside `(:wat::core::show …)`, which accepts anything — `(:wat::core::Option::Some
//! {:value 1})` ALREADY checks clean, because the map is swallowed as the PAYLOAD and the value
//! types as `Option<HashMap<keyword,i64>>`. A probe written that way returns green and proves
//! nothing. The orchestrator wrote exactly that probe first, read the green, and reported "no
//! new machinery needed" — which was false. Only a typed slot can tell the target apart from
//! the accident.
//!
//! ⚠ EVERY BAR IS THE CONTROL, RUN IN THE SAME TEST — never a hand-written exit code. A row
//! asserting `== 0` outright also passes on a mis-aimed harness that returns 0 for everything;
//! a row asserting `!= 0` is satisfied by a typo in its own fixture.
//!
//! Row 5 is the one that proves the migration FINISHED rather than merely ADDED a spelling:
//! it is red today in the OPPOSITE direction from rows 2-4, because the retired form is
//! currently accepted.
//!
//! Un-ignored by the stone.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Fixture-local error count: how many `--check` diagnostics cite THIS file.
/// Never a bare exit code — while the stdlib is red, every `--check` exits 1.
fn check(case: &str) -> i32 {
    let rel = format!("tests/types/probe_arc296_enum_map_ctor__{case}.wat");
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&rel);
    assert!(path.exists(), "fixture missing: {}", path.display());
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg("--check")
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat --check");
    let hay = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let needle = format!("\"{rel}\"");
    hay.matches(&needle).count() as i32
}

/// The bar. GREEN at HEAD and must stay green — if this fails, nothing below means anything.
#[test]
fn the_control_program_checks_clean() {
    assert_eq!(
        check("control"),
        0,
        "the control constructs no enum at all — non-zero here means the harness, the binary \
         or the fixture path is broken, not that the stone's subject is"
    );
}

#[test]
fn a_payload_variant_is_built_from_a_map_naming_its_field() {
    assert_eq!(
        check("user_map"),
        check("control"),
        "(:probe::Box::Full {{:payload 7}}) must check as clean as the control, in a slot TYPED \
         :probe::Box — the map names the declared field, it is not the payload"
    );
}

#[test]
fn a_unit_variant_is_built_from_an_empty_map() {
    assert_eq!(
        check("user_unit_map"),
        check("control"),
        "(:probe::Box::Empty {{}}) must check clean — a unit variant is a variant with no fields, \
         not a variant with no constructor"
    );
}

#[test]
fn option_is_an_ordinary_enum_and_takes_the_same_ctor() {
    assert_eq!(
        check("option_map"),
        check("control"),
        "Option/Result are parametric enums declared in wat (296 H-3); they get the SAME map \
         ctor as any user enum, with no special case"
    );
}

#[test]
fn the_retired_positional_ctor_is_refused() {
    assert_ne!(
        check("positional"),
        check("control"),
        "(:probe::Box::Full 7) must be REFUSED once the flip lands — defrecord already refuses \
         (Pt 1 2); two calling conventions accepted at the end is a migration that never finished"
    );
}
