//! PROBE — three positions still give three answers to "is this a type?"
//!
//! Arc 296 P-1 built the annotation wall over four stores. The P-2 prereq made `is-type?` agree
//! with `resolve` for names THIS program declares. Two disagreements survive, and they are the
//! two halves of one thing. Measured on the pushed green tree `680d6718e`:
//!
//! ## Half 1 — the wall counts a STDLIB `use!` as the user's own
//!
//! ```text
//!   same name, same program, NO user use!:
//!     resolve   (call head)    :rust::sqlite::Connection::open   REFUSED
//!     is-type?                 :rust::sqlite::Connection         false
//!     the WALL  (annotation)   [c <- :rust::sqlite::Connection]  ACCEPTED   ⛔ the outlier
//! ```
//!
//! `resolve` Pass 1 walks USER RESIDUE ONLY, so a stdlib `use!` never covers a user call head.
//! The wall consults the MERGED set — RELAND-1 folded stdlib `use!` into `use_decls` so the
//! stdlib's own annotations would stop screaming, and that merge is what makes the wall
//! scope-blind. Coverage is per-DECLARING-SCOPE, not per-program.
//!
//! ⚠ **The predicate that selects the scope is `is_reserved_prefix` — the same one RELAND-1
//! DELETED.** That deletion removed a `continue` (a SKIP: those declarations were not validated
//! at all). Selecting WHICH set to validate against is not a skip: every declaration is still
//! checked, against the declarations its own scope actually made. A future reader must not read
//! this as the blanket returning.
//!
//! ## Half 2 — `is-type?` still cannot see store 4
//!
//! ```text
//!   is-type? :wat::spawn::Spawned   ->  false      and the WALL accepts it
//! ```
//!
//! `:wat::spawn::Spawned` exists ONLY as a derive parent — `wat/spawn.wat:235-236`, *"the
//! owner-side spawn-handle marker (typesub/derive axis; no methods)"*. There is no declaration
//! form for a marker; deriving to it is what mints it, which is Clojure's open `derive`/`isa?`
//! hierarchy and is what `subtype_edges`' own field doc cites.
//!
//! ⚠ MEASURED AND OUT OF SCOPE — `derive` does not validate its marker:
//!
//! ```text
//!   (:wat::core::derive :usr::A :usr::TotallyMadeUpMarker)
//!   (:user::f [m <- :usr::TotallyMadeUpMarker] …)          --check EXIT 0
//! ```
//!
//! Two lines turn any phantom into an accepted type. Under the open-hierarchy reading that is
//! the mechanism working, not a hole — but a MISTYPED parent silently mints a new marker instead
//! of relating to the intended one. Whether markers should be declared-first is the builder's
//! ruling, and it does not gate this stone either way: one-question-one-answer says the verb must
//! agree with the wall, and it must agree whichever way that rules.
//!
//! ## After this stone, all three positions agree in BOTH directions
//!
//! ```text
//!   no user use!    resolve REFUSES  ·  wall REFUSES  ·  is-type? false
//!   user use!       resolve accepts  ·  wall accepts  ·  is-type? true
//! ```
//!
//! The two subject tests are `#[ignore]`d and are UN-IGNORED BY THE STONE.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn fixture(case: &str) -> PathBuf {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc296_p3_one_question_one_answer__{case}.wat"));
    assert!(p.exists(), "fixture missing: {}", p.display());
    p
}

fn check(case: &str) -> i32 {
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg("--check")
        .arg(fixture(case))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat --check");
    out.status.code().unwrap_or(-1)
}

fn run(case: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(fixture(case))
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

// ── Half 1 — the wall's scope ────────────────────────────────────────────────

/// ⛔ THE WIDEST CONTROL. If scope-awareness is drawn wrong, the stdlib's own `:rust::`
/// annotations stop resolving and NOTHING loads. This fixture is the first to say so, and a
/// failure here means the stone broke the world, not that its subject moved.
#[test]
fn the_stdlib_still_loads() {
    assert_eq!(run("stdlib_annotation_still_loads"), "\"loaded\"");
}

/// CONTROL — the SAME annotation as the subject, with this program's own `use!`. The stone
/// narrows the SCOPE; it does not ban the annotation.
#[test]
fn a_user_annotation_with_its_own_use_is_accepted() {
    assert_eq!(check("user_annotation_with_user_use"), 0);
}

/// HALF 1 SUBJECT — accepted today only because the wall counts the stdlib's `use!` as this
/// program's. `resolve` refuses the same name in call-head position.
#[test]
#[ignore = "arc 296 P-3 — the annotation wall counts a stdlib use! as the user program's own"]
fn a_user_annotation_without_its_own_use_is_refused() {
    assert_eq!(
        check("user_annotation_without_user_use"),
        1,
        "resolve refuses `:rust::sqlite::Connection::open` in this exact program; the \
         annotation position must not be more permissive than the call position"
    );
}

// ── Half 2 — the verb's stores ───────────────────────────────────────────────

/// CONTROL — a name nothing ever derived to. FALSE now, must stay FALSE. This is the over-reach
/// detector for store 4: accepting every keyword would satisfy the subject and fail here.
#[test]
fn a_name_that_was_never_derived_to_is_not_a_type() {
    assert_eq!(run("is_type_on_a_non_marker"), "false");
}

/// HALF 2 SUBJECT — the wall accepts a derive marker as a bound; the verb denies it.
#[test]
#[ignore = "arc 296 P-3 — is-type? does not ask subtype_edges (store 4)"]
fn a_derive_marker_is_a_type() {
    assert_eq!(
        run("is_type_on_a_derive_marker"),
        "true",
        "`[m <- :wat::spawn::Spawned]` type-checks, so the marker IS a usable type; is-type? \
         must not contradict the wall that accepts it"
    );
}
