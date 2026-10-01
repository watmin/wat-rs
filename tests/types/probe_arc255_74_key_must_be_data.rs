//! Arc 255 Stone 255.74 — a set element / map key must be data (key-eligible), refused by
//! the checker, through ONE door (`is_atomizable`, `src/check.rs`).
//!
//! 255.73's census found `wat-scripts/probes/arc-170/probe-compound-upcast.wat` reaching a
//! Rust `unreachable!()` — a service HANDLE (a `RustOpaque` at runtime) accepted as a
//! `HashSet` element by the checker, panicking `impl Hash for Value` at run time. Measured,
//! `is_atomizable` had exactly ONE caller (`to-holon`/`leaf`) — building a `HashSet`, a
//! `HashMap`/`PersistentMap` key, a `#{}`/`{}` literal, `conj`, or `assoc` never consulted it.
//!
//! This file drives EVERY site Stone 255.74 wired `is_atomizable` into (one row per site, a
//! `.wat` fixture per row — `tests/types/probe_arc255_74_key_must_be_data__<case>.wat`):
//!
//! | fixture                              | site (`src/check.rs` unless noted)              |
//! |---------------------------------------|-------------------------------------------------|
//! | `hashmap_fn_key`                      | `infer_hashmap_constructor` (declared K)         |
//! | `hashmap_call_arg_fn_key`              | `check_map_literal_against` (expected K)         |
//! | `persistentmap_fn_key`                 | `infer_persistentmap_constructor` (declared K)   |
//! | `set_literal_fn`                       | `infer_set_literal` (bottom-up `#{}`)            |
//! | `map_literal_fn_key`                   | `infer_map_literal` (bottom-up `{}`)             |
//! | `conj_fn_into_empty_set`               | `collection/infer.rs::infer_conj` (HashSet arm)  |
//! | `assoc_fn_key_into_empty_map`          | `collection/infer.rs::infer_assoc` (HashMap arm) |
//!
//! Three more live as this stone's permanent NEGATIVE fixtures under `wat-scripts/probes/`
//! (`.wat.bad` — excluded from `every_wat_scripts_file_loads`'s `*.wat` glob by extension, per
//! `wat-scripts/fmt/fixtures/spelling-dotted.wat.bad`'s existing convention) rather than
//! `tests/types/`, because they are the exact scratch probes 255.73's orchestrator isolated
//! the original bug with, promoted to recorded fixtures:
//!
//! | fixture                                                            | site                        |
//! |---------------------------------------------------------------------|------------------------------|
//! | `arc-255/probe-255.74-a-key-must-be-data.wat.bad`                   | `infer_hashset_constructor` (shallow: fn element) |
//! | `arc-255/probe-255.74-a-key-must-be-data-deep.wat.bad`              | `infer_hashset_constructor` (deep: `Vector<fn>` element) |
//! | `arc-255/probe-255.74-set-of-capability.wat.bad`                    | `check_set_literal_against` (expected elem; extracted from `probe-compound-upcast.wat`'s retired Set case) |
//!
//! Acceptance rows (`acceptance.wat`) prove key-eligible types are UNAFFECTED: i64, String,
//! keyword, a record, a vector of i64, and a tuple all still build and RUN as HashSet elements
//! / HashMap keys. `control.wat` is the harness sanity bar (no HashSet/Map at all).
//!
//! EVERY BAR IS THE CONTROL, RUN IN THE SAME TEST — never a hand-written exit code alone: each
//! refusal row also asserts the error names BOTH the offending type and the key-eligibility
//! wall's own message, via an exact substring COUNT (`.matches(needle).count()`, never
//! `.contains()` inside an assert — `no_loose_string_assert`'s own remedy), so a row cannot
//! pass by accident on an unrelated error.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Runs `wat --check <path>` (relative to the crate root unless `path` is `wat-scripts/...`,
/// in which case it's relative to the crate root too — both live under `CARGO_MANIFEST_DIR`).
/// Returns `(exit_code, combined stdout+stderr)`.
fn check(rel: &str) -> (Option<i32>, String) {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join(rel);
    assert!(path.exists(), "fixture missing: {}", path.display());
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg("--check")
        .arg(&path)
        .current_dir(&manifest)
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
    (out.status.code(), hay)
}

fn needle_count(hay: &str, needle: &str) -> usize {
    hay.matches(needle).count()
}

const WALL_MESSAGE: &str = "key-eligible type (is_atomizable)";

/// The bar. GREEN at HEAD and must stay green — if this fails, nothing below means anything.
#[test]
fn the_control_program_checks_clean() {
    let (code, hay) = check("tests/types/probe_arc255_74_key_must_be_data__control.wat");
    assert_eq!(
        code,
        Some(0),
        "the control has no HashSet/HashMap/PersistentMap at all; non-zero here means the \
         harness, the binary, or the fixture path is broken, not that the stone's subject is: \
         {hay}"
    );
}

/// Acceptance: i64, String, keyword, a record, a vector of i64, and a tuple all still build
/// (as HashSet elements and a HashMap key) AND run — key-eligible types are unaffected.
#[test]
fn key_eligible_types_still_check_and_run_clean() {
    let rel = "tests/types/probe_arc255_74_key_must_be_data__acceptance.wat";
    let (code, hay) = check(rel);
    assert_eq!(code, Some(0), "key-eligible HashSet/HashMap construction must stay green: {hay}");

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(manifest.join(rel))
        .current_dir(&manifest)
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat (run)");
    assert_eq!(
        out.status.code(),
        Some(0),
        "acceptance fixture must RUN clean too (construction alone proving nothing about \
         `impl Hash for Value` not panicking); stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "\"acceptance: ok\"\n"
    );
}

/// One row per checker-side site Stone 255.74 wired `is_atomizable` into. Each fixture's
/// offending element/key is the Fn type `[:wat::core::i64 :-> :wat::core::i64]` (`:probe::inc`
/// used as a first-class value) — chosen uniformly so one assertion shape covers every row.
fn assert_fn_key_refused(case: &str) {
    let rel = format!("tests/types/probe_arc255_74_key_must_be_data__{case}.wat");
    let (code, hay) = check(&rel);
    assert_ne!(code, Some(0), "fixture `{case}` must be REFUSED by --check: {hay}");
    assert_eq!(
        needle_count(&hay, "TypeMismatch"),
        1,
        "fixture `{case}` must be refused by EXACTLY ONE error (the key-eligibility wall), \
         not some unrelated error: {hay}"
    );
    assert!(
        needle_count(&hay, WALL_MESSAGE) >= 1,
        "fixture `{case}` must be refused by the key-eligibility wall: {hay}"
    );
    assert!(
        needle_count(&hay, ":wat::core::i64 :-> :wat::core::i64]") >= 1,
        "fixture `{case}`'s diagnostic must name the offending Fn type: {hay}"
    );
}

// `infer_hashset_constructor`'s own direct-constructor wall is driven by the two permanent
// negative fixtures below (`hashset_fn_element_shallow_wat_bad_is_refused` / `..._deep_...`) —
// the exact probes 255.73's orchestrator isolated the original bug with, promoted intact.

#[test]
fn hashmap_constructor_fn_key_is_refused() {
    assert_fn_key_refused("hashmap_fn_key");
}

#[test]
fn hashmap_call_arg_fn_key_is_refused() {
    let (code, hay) = check("tests/types/probe_arc255_74_key_must_be_data__hashmap_call_arg_fn_key.wat");
    assert_ne!(code, Some(0), "a {{…}} map literal up-cast against a declared Fn-keyed HashMap param must be refused: {hay}");
    assert!(needle_count(&hay, WALL_MESSAGE) >= 1, "{hay}");
    assert!(needle_count(&hay, "…} map literal") >= 1, "must name the map-literal container: {hay}");
}

#[test]
fn persistentmap_constructor_fn_key_is_refused() {
    assert_fn_key_refused("persistentmap_fn_key");
}

#[test]
fn set_literal_bottom_up_fn_element_is_refused() {
    let (code, hay) = check("tests/types/probe_arc255_74_key_must_be_data__set_literal_fn.wat");
    assert_ne!(code, Some(0), "a bare #{{fn}} set literal must be refused: {hay}");
    assert!(needle_count(&hay, WALL_MESSAGE) >= 1, "{hay}");
    assert!(needle_count(&hay, "…} set literal") >= 1, "must name the set-literal container: {hay}");
}

#[test]
fn map_literal_bottom_up_fn_key_is_refused() {
    let (code, hay) = check("tests/types/probe_arc255_74_key_must_be_data__map_literal_fn_key.wat");
    assert_ne!(code, Some(0), "a bare {{fn 1}} map literal must be refused: {hay}");
    assert!(needle_count(&hay, WALL_MESSAGE) >= 1, "{hay}");
    assert!(needle_count(&hay, "…} map literal") >= 1, "must name the map-literal container: {hay}");
}

#[test]
fn conj_onto_empty_set_resolving_fn_element_is_refused() {
    let (code, hay) = check("tests/types/probe_arc255_74_key_must_be_data__conj_fn_into_empty_set.wat");
    assert_ne!(code, Some(0), "(conj #{{}} fn) must be refused — conj resolves the set's fresh element type from its argument: {hay}");
    assert!(needle_count(&hay, WALL_MESSAGE) >= 1, "{hay}");
    assert!(needle_count(&hay, ":wat::core::conj") >= 1, "must name conj as the offending verb: {hay}");
}

#[test]
fn assoc_onto_empty_map_resolving_fn_key_is_refused() {
    let (code, hay) = check("tests/types/probe_arc255_74_key_must_be_data__assoc_fn_key_into_empty_map.wat");
    assert_ne!(code, Some(0), "(assoc {{}} fn 1) must be refused — assoc resolves the map's fresh key type from its argument: {hay}");
    assert!(needle_count(&hay, WALL_MESSAGE) >= 1, "{hay}");
    assert!(needle_count(&hay, ":wat::core::assoc") >= 1, "must name assoc as the offending verb: {hay}");
}

// ─── The three permanent negative fixtures under wat-scripts/probes/arc-255/ ───────────────

#[test]
fn hashset_fn_element_shallow_wat_bad_is_refused() {
    let (code, hay) = check("wat-scripts/probes/arc-255/probe-255.74-a-key-must-be-data.wat.bad");
    assert_ne!(code, Some(0), "the ORIGINAL shallow probe (255.73's isolation) must be refused: {hay}");
    assert!(needle_count(&hay, WALL_MESSAGE) >= 1, "{hay}");
    assert!(needle_count(&hay, ":wat::core::HashSet") >= 1, "{hay}");
}

#[test]
fn hashset_vector_of_fn_deep_wat_bad_is_refused() {
    let (code, hay) = check("wat-scripts/probes/arc-255/probe-255.74-a-key-must-be-data-deep.wat.bad");
    assert_ne!(code, Some(0), "the ORIGINAL deep probe (255.73's isolation) must be refused: {hay}");
    assert!(needle_count(&hay, WALL_MESSAGE) >= 1, "{hay}");
    assert!(
        needle_count(&hay, ":wat::core::Vector :- [[:wat::core::i64 :-> :wat::core::i64]])") >= 1,
        "the diagnostic must name the NESTED Vector<Fn> type, not just \"Fn\" — proving the \
         wall recurses through is_atomizable's Vector arm: {hay}"
    );
}

#[test]
fn set_of_capability_wat_bad_is_refused() {
    let (code, hay) = check("wat-scripts/probes/arc-255/probe-255.74-set-of-capability.wat.bad");
    assert_ne!(
        code,
        Some(0),
        "a service handle (Capability) up-cast into a HashSet element must be refused — \
         extracted from probe-compound-upcast.wat's retired Set case: {hay}"
    );
    assert!(needle_count(&hay, WALL_MESSAGE) >= 1, "{hay}");
    assert!(needle_count(&hay, ":wat::capability::Capability") >= 1, "{hay}");
    assert!(needle_count(&hay, "…} set literal") >= 1, "must name the set-literal call-arg path: {hay}");
}
