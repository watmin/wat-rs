//! Arc 255 Stone 255.74 — a set element / map key must be data, refused by the checker,
//! through ONE door.
//!
//! **AMEND-255.74 D3 (builder ruling):** the door is `:< :wat::core::Equatable`
//! (`wat/class.wat`), asked via `require_class` — the SAME predicate `=` already asks. The
//! original brief named `is_atomizable` as the door; that predicate answers *"can be encoded
//! as a holon atom"*, a different property, and it refused real data (`u8`, `bigint`,
//! `rational`, `Instant`, `(Option :- [i64])`, `(PersistentVector :- [i64])` were all refused
//! by `--check` while the runtime hashed every one of them, pre-amendment). `is_atomizable`
//! stays exactly what it was — `to-holon`/`leaf`'s own door — just no longer tied to
//! key-eligibility.
//!
//! 255.73's census found `wat-scripts/probes/arc-170/probe-compound-upcast.wat` reaching a
//! Rust `unreachable!()` — a service HANDLE (a `RustOpaque` at runtime) accepted as a
//! `HashSet` element by the checker, panicking `impl Hash for Value` at run time. Measured,
//! the key door had exactly ONE caller (`to-holon`/`leaf`, and it was the wrong door) —
//! building a `HashSet`, a `HashMap`/`PersistentMap` key, a `#{}`/`{}` literal, `conj`, or
//! `assoc` never consulted any class membership at all.
//!
//! This file drives EVERY site Stone 255.74 wired the key door into (one row per site, a
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
//! Acceptance rows (`acceptance.wat`) prove key-eligible types still build and RUN as HashSet
//! elements / HashMap keys: i64, f64, String, keyword, a record, a Pure enum, a vector of i64,
//! and a tuple (already admitted pre-amendment, by EITHER door) — AND the nine types D3's own
//! measurement named as newly-admitted (`u8`, `bigint`, `rational`, `Instant`,
//! `(Option :- [i64])`, `(PersistentVector :- [i64])`), refused pre-amendment,
//! `is_atomizable`-only. `control.wat` is the harness sanity bar (no HashSet/Map at all).
//! `generic_bounded.wat`/`generic_unbounded.wat` prove a generic function building
//! `(HashSet :- [T])` checks with `[T :< Equatable]` and is refused (naming `T`) without it.
//!
//! EVERY BAR IS THE CONTROL, RUN IN THE SAME TEST — never a hand-written exit code alone: each
//! refusal row also asserts the error names BOTH the offending type and the key-eligibility
//! wall's own message, via a substring COUNT (`.matches(needle).count()`, never `.contains()`
//! inside an assert — `no_loose_string_assert`'s own remedy), so a row cannot pass by accident
//! on an unrelated error.

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

// AMEND-255.74 D3 — the key door is `:< :wat::core::Equatable` (require_class), not
// is_atomizable. Every refusal below names the class, whether as a plain TypeMismatch
// ("expects :wat::core::Equatable") or (for a conditional extend-type edge whose bound
// failed, e.g. `(Vector :- [T :< Equatable])`) a MembershipBound ("is not a
// :wat::core::Equatable" / "bounded by :wat::core::Equatable") — both spellings contain this
// substring.
const WALL_MESSAGE: &str = ":wat::core::Equatable";

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

/// One row per checker-side site Stone 255.74 wired the key door into. Each fixture's
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
         wall recurses through wat/class.wat's conditional `(Vector :- [T :< Equatable]) :< \
         Equatable` edge (surfaced as a MembershipBound naming T, not a plain TypeMismatch, \
         when the edge matches but T's bound fails): {hay}"
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

// ─── AMEND-255.74 D3 item 2 — a generic function keying a HashSet by its OWN type parameter ──

#[test]
fn generic_function_with_equatable_bound_checks() {
    let (code, hay) = check("tests/types/probe_arc255_74_key_must_be_data__generic_bounded.wat");
    assert_eq!(
        code,
        Some(0),
        "(defn singleton :- [[T :< Equatable]] [x <- :T] -> (HashSet :- [:T]) …) must check \
         clean — require_class -> assignable consults env.bound_of(\"T\") for a declared, \
         bounded type parameter: {hay}"
    );
}

#[test]
fn generic_function_without_equatable_bound_is_refused_naming_t() {
    let (code, hay) = check("tests/types/probe_arc255_74_key_must_be_data__generic_unbounded.wat");
    assert_ne!(
        code,
        Some(0),
        "the same function with the [T :< Equatable] bound dropped must be refused: {hay}"
    );
    assert!(needle_count(&hay, WALL_MESSAGE) >= 1, "{hay}");
    assert_eq!(
        needle_count(&hay, "got \":T\""),
        1,
        "the diagnostic must name the unbounded parameter BY NAME (\"T\"), not a resolved \
         type — proving this is the declared-type-parameter path (env.bound_of(\"T\") finds \
         nothing), not the fresh-inference-variable path (BoundUnresolved/BoundNotSatisfied): \
         {hay}"
    );
}
