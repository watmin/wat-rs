//! Probe (excursus 003, stone E) — **THE HASHABILITY CHECK LOOKS INSIDE THE KEY.**
//!
//! Before this stone, `value_is_hashable` (`src/runtime.rs`) — the one predicate every hashed
//! container consults before hashing — read only the key's OUTER variant. A hashable container
//! holding an opaque handle passed it, and `impl Hash for Value` (`src/value/value.rs`) then
//! recursed into the container and reached an `unreachable!()` arm:
//!
//! ```text
//!   thread 'main' panicked at src/value/value.rs:914:37:
//!   internal error: entered unreachable code: Value::RustOpaque is not atomizable; is_atomizable
//!   predicate in src/check.rs should have rejected this. If you see this panic, the predicate has
//!   drifted.
//! ```
//!
//! Every cell of the container × verb matrix below produced that panic (exit 2) before the cure —
//! measured, not inferred. The design is
//! `docs/excursus/2026/09/003-the-little-wat-findings/DESIGN-stone-E-hashability-looks-inside.md`.
//!
//! ## The matrix these fixtures pin
//!
//! Containers, each wrapping an `Lru` handle: `Option.Some`, `Tuple`, `Vector`, `List`, a `HashMap`
//! whose VALUE is the handle, a record field (laundered through a generic `T` — a record declaring
//! the handle type directly is refused by the containment rule), an enum variant field (an
//! `:wat::enum::Impure` enum). Verbs, and the normal refusal each must give:
//!
//! | verb | fixture | answer |
//! |---|---|---|
//! | `Lru/put` | `<c>_lru` (first line) | an `Err` value — `key must be a hashable value; got …` |
//! | `Lru/get` | `<c>_lru` (last line) | a miss |
//! | `HashMap` assoc | `<c>_hashmap` | a wat `TypeMismatch` (exit 1), golden-pinned |
//! | `HashSet` conj | `<c>_hashset` | a wat `TypeMismatch` (exit 1), golden-pinned |
//!
//! ⚠ **`Lru/get` only hashes a NON-EMPTY table** — lru's hashbrown map answers `None` on an empty
//! table without hashing the key, so a get against an empty cache misses before and after the
//! cure and proves nothing. Each `<c>_lru` fixture therefore stores a pure sibling of the key's
//! type first (`SIBLING STORED` pins that it landed). `Tuple` and the record have no handle-free
//! value of their type, so their cache can never be non-empty and their get cell is a miss by
//! construction — that is a property of the verb, recorded, not a gap in the probe.
//!
//! More fixtures:
//! - `result_hashset`, `persistent_vector_hashset`, `persistent_map_hashset` — one HashSet cell for
//!   each further recursive arm reachable from wat with a handle inside, so mutation (a) is seen
//!   per arm. Arms with NO wat-reachable handle inside, and therefore no cell: `HashSet` (and a
//!   `HashMap`'s KEY side) — every insert path is this same guard; `ForeignRecord`/`ForeignVariant`
//!   — decoded from the wire, pure data by construction; a STAMPED `Aggregate` — see the arm's
//!   comment in `src/runtime.rs`.
//! - `stream_hashset` — the DRIFT the single-sourcing closes: the hand list the cure replaced named
//!   13 of the 14 `unreachable!()` variants and omitted `wat__stream__Stream`, so a BARE Stream key
//!   panicked (`value.rs:968`) with no nesting at all.
//! - `pure_deep_keys` — the positive control: a deep key of pure data, a `List` key and a
//!   `PersistentMap` key still insert and are found. `List`/`PersistentMap` are `ExcludedByDesign`
//!   in `key_eligibility()` but have REAL `Hash` arms; refusing them would outlaw a truth.
//!
//! ## ⛔ The driver is the binary — the same contract decision as the sibling ex003 probes
//!
//! A the-little-wat finding is a claim about what a user experiences running `wat foo.wat`, and
//! this defect lives at RUN time. `probe_ex003_silent_failure_pair.rs`'s header states it in full.
//!
//! ## Mutations (this gate has been driven RED)
//!
//! - **(a) the recursion made shallow again** (every recursive arm of `value_is_hashable` →
//!   `true`): 25 of 27 RED — every nested cell, all three verbs, plus the no-panic sweep. Green:
//!   `hashset_bare_stream_is_a_type_mismatch` (a leaf, untouched) and
//!   `pure_deep_keys_still_insert_and_are_found` (nothing to refuse).
//! - **(b) a `_ => true` wildcard** added to the exhaustive match: alone it reddens NOTHING — it is
//!   only an `unreachable pattern` warning, because every variant is already named. What it costs
//!   is the next variant: with the wildcard present, deleting the `wat__stream__Stream` leaf arm
//!   (standing in for "a new variant nobody classified") compiles and reddens
//!   `hashset_bare_stream_is_a_type_mismatch` + the sweep; WITHOUT the wildcard the same deletion
//!   is `error[E0004]: non-exhaustive patterns: … wat__stream__Stream(_) not covered`. The
//!   exhaustiveness is the gate for drift; this probe is the gate for behaviour.

use std::path::PathBuf;
use std::process::{Command, Stdio};

const PREFIX: &str = "probe_ex003_hashability_looks_inside";

fn fixture(case: &str) -> PathBuf {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/diagnostics")
        .join(format!("{PREFIX}__{case}.wat"));
    assert!(p.exists(), "fixture missing: {}", p.display());
    p
}

/// One invocation of the binary against a fixture. Returns `(exit code, stdout, stderr)`.
fn run(case: &str) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(fixture(case))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// The `Lru` cells: the whole observable — exit 0, exact stdout, EMPTY stderr. The empty stderr is
/// the no-panic assertion (a panic writes its banner and `RUST_BACKTRACE` note there).
fn assert_lru_cell(container: &str, expected_stdout: &str) {
    let case = format!("{container}_lru");
    assert_eq!(
        run(&case),
        (0, expected_stdout.to_string(), String::new()),
        "`{case}`: Lru/put on a key nesting an opaque handle must be an Err value and Lru/get \
         must miss — exit 0, nothing on stderr (no panic banner, no RUST_BACKTRACE note)"
    );
}

/// The `HashMap`/`HashSet` cells: exit 1 (an unhandled wat `RuntimeError`, not a panic's 2), an
/// empty stdout (the insert never happened), and the whole stderr pinned as an EDN golden.
///
/// Captured, never hand-authored:
/// `UPDATE_EDN=1 cargo nextest run --release -E 'test(/hashability_looks_inside/)'`.
/// The face goes through [`with_stringified_payload_unwrapped`] first — see there for why.
/// `assert_edn_eq!` then blanks the `:line` of any span whose `:file` is a `src/**.rs` path on
/// both sides, so the `src/collection/eval.rs` location in these faces cannot rot the gate on an
/// unrelated edit. (That the location is a Rust file and not the user's `.wat` is its own stone —
/// DESIGN § Out of scope.)
macro_rules! refusal_cell {
    ($name:ident, $case:literal) => {
        #[test]
        fn $name() {
            let (rc, stdout, stderr) = run($case);
            assert_eq!(
                (rc, stdout.as_str()),
                (1, ""),
                "`{}` must die as a wat TypeMismatch (exit 1) before inserting. stderr:\n{stderr}",
                $case
            );
            wat::assert_edn_matches_file!(
                with_stringified_payload_unwrapped(&stderr),
                concat!(
                    "probe_ex003_hashability_looks_inside__",
                    $case,
                    "_stderr.edn"
                ),
                "the refusal face changed — check it is still a #wat.runtime/TypeMismatch naming \
                 the verb, and not a panic"
            );
        }
    };
}

/// A `#wat.kernel/LociDiedError.RuntimeError` carries its payload — the
/// `#wat.runtime/TypeMismatch` — as an EDN document STRINGIFIED into `:message`. Pinned raw, that
/// string holds `:file "src/collection/eval.rs" :line 449`, which `blank_rust_source_lines` cannot
/// see inside a string, so every golden would rot on any edit above line 449 of an unrelated file.
///
/// This re-reads every `:message` string that is itself a tagged EDN document and puts the parsed
/// document in its place. ⛔ It is NOT a filter: every byte of the payload is still compared, now
/// as data; a `:message` that is prose (the TypeMismatch's own) is left a string.
fn with_stringified_payload_unwrapped(stderr: &str) -> String {
    fn unwrap(v: &mut wat_edn::OwnedValue) {
        use wat_edn::Value;
        match v {
            Value::Map(entries) => {
                for (k, val) in entries.iter_mut() {
                    let is_message = matches!(k, Value::Keyword(kw) if kw.name() == "message");
                    let parsed = match (&*val, is_message) {
                        (Value::String(text), true) if text.starts_with('#') => {
                            wat_edn::parse_owned(text).ok()
                        }
                        _ => None,
                    };
                    match parsed {
                        Some(doc @ Value::Tagged(..)) => *val = doc,
                        _ => unwrap(val),
                    }
                }
            }
            Value::List(xs) | Value::Vector(xs) | Value::Set(xs) => xs.iter_mut().for_each(unwrap),
            Value::Tagged(_, inner) => unwrap(inner),
            _ => {}
        }
    }
    let mut face = wat_edn::parse_owned(stderr.trim())
        .unwrap_or_else(|e| panic!("stderr is not one EDN document: {e}\nstderr:\n{stderr}"));
    unwrap(&mut face);
    wat_edn::write(&face)
}

const REFUSED: &str = "\"key must be a hashable value; got ";

fn put_refused_then(type_name: &str, rest: &str) -> String {
    format!("{REFUSED}{type_name}\"\n{rest}")
}

const POPULATED_MISS: &str = "\"SIBLING STORED\"\n\"MISS\"\n";
const EMPTY_MISS: &str = "\"MISS\"\n";

#[test]
fn lru_option_some_handle_put_errs_get_misses() {
    assert_lru_cell(
        "option",
        &put_refused_then("wat::core::Option", POPULATED_MISS),
    );
}

#[test]
fn lru_tuple_handle_put_errs_get_misses() {
    assert_lru_cell("tuple", &put_refused_then("wat::core::Tuple", EMPTY_MISS));
}

#[test]
fn lru_vector_handle_put_errs_get_misses() {
    assert_lru_cell(
        "vector",
        &put_refused_then("wat::core::Vector", POPULATED_MISS),
    );
}

#[test]
fn lru_list_handle_put_errs_get_misses() {
    assert_lru_cell("list", &put_refused_then("wat::core::List", POPULATED_MISS));
}

#[test]
fn lru_hashmap_value_handle_put_errs_get_misses() {
    assert_lru_cell(
        "hashmap_value",
        &put_refused_then("wat::core::HashMap", POPULATED_MISS),
    );
}

#[test]
fn lru_record_field_handle_put_errs_get_misses() {
    assert_lru_cell("record", &put_refused_then("wat::core::Record", EMPTY_MISS));
}

#[test]
fn lru_enum_field_handle_put_errs_get_misses() {
    assert_lru_cell("enum", &put_refused_then("wat::core::Enum", POPULATED_MISS));
}

refusal_cell!(
    hashmap_option_some_handle_is_a_type_mismatch,
    "option_hashmap"
);
refusal_cell!(hashmap_tuple_handle_is_a_type_mismatch, "tuple_hashmap");
refusal_cell!(hashmap_vector_handle_is_a_type_mismatch, "vector_hashmap");
refusal_cell!(hashmap_list_handle_is_a_type_mismatch, "list_hashmap");
refusal_cell!(
    hashmap_hashmap_value_handle_is_a_type_mismatch,
    "hashmap_value_hashmap"
);
refusal_cell!(
    hashmap_record_field_handle_is_a_type_mismatch,
    "record_hashmap"
);
refusal_cell!(hashmap_enum_field_handle_is_a_type_mismatch, "enum_hashmap");

refusal_cell!(
    hashset_option_some_handle_is_a_type_mismatch,
    "option_hashset"
);
refusal_cell!(hashset_tuple_handle_is_a_type_mismatch, "tuple_hashset");
refusal_cell!(hashset_vector_handle_is_a_type_mismatch, "vector_hashset");
refusal_cell!(hashset_list_handle_is_a_type_mismatch, "list_hashset");
refusal_cell!(
    hashset_hashmap_value_handle_is_a_type_mismatch,
    "hashmap_value_hashset"
);
refusal_cell!(
    hashset_record_field_handle_is_a_type_mismatch,
    "record_hashset"
);
refusal_cell!(hashset_enum_field_handle_is_a_type_mismatch, "enum_hashset");

// One HashSet cell per remaining recursive arm the wat surface can reach with a handle inside.
refusal_cell!(
    hashset_result_ok_handle_is_a_type_mismatch,
    "result_hashset"
);
refusal_cell!(
    hashset_persistent_vector_handle_is_a_type_mismatch,
    "persistent_vector_hashset"
);
refusal_cell!(
    hashset_persistent_map_value_handle_is_a_type_mismatch,
    "persistent_map_hashset"
);

// The drift the hand list carried: a BARE Stream, no nesting.
refusal_cell!(hashset_bare_stream_is_a_type_mismatch, "stream_hashset");

/// EXPECTATIONS rows 2 and 4: the deep check refuses handles, not depth. A deep pure-data key
/// inserts and is found in all three containers, and `List` / `PersistentMap` keys stay legal.
#[test]
fn pure_deep_keys_still_insert_and_are_found() {
    assert_eq!(
        run("pure_deep_keys"),
        (
            0,
            "\"PUT OK\"\ntrue\ntrue\ntrue\ntrue\n42\n".to_string(),
            String::new()
        ),
        "a deep pure-data key, a List key or a PersistentMap key was refused or lost — the \
         predicate has become stricter than impl Hash"
    );
}

/// ⭐ What the face must NEVER be, over every refusal fixture. The goldens and exact stdouts above
/// already pin the whole observable, so this cannot fail while they pass; it is the assertion that
/// refuses a re-capture of a panic when someone runs `UPDATE_EDN=1` after a legitimate change.
#[test]
fn no_rust_panic_face_reaches_the_user() {
    let containers = [
        "option",
        "tuple",
        "vector",
        "list",
        "hashmap_value",
        "record",
        "enum",
    ];
    let mut cases: Vec<String> = Vec::new();
    for c in containers {
        for verb in ["lru", "hashmap", "hashset"] {
            cases.push(format!("{c}_{verb}"));
        }
    }
    for extra in [
        "result_hashset",
        "persistent_vector_hashset",
        "persistent_map_hashset",
        "stream_hashset",
    ] {
        cases.push(extra.to_string());
    }
    for case in &cases {
        let (_rc, _stdout, stderr) = run(case);
        for needle in ["RUST_BACKTRACE", "panicked at", "unreachable code"] {
            // rune:lint(loose-assert) — a TARGETED ABSENCE over a large output, the rubric's own
            // named exemption. The positive shape is pinned exactly by the cells above; what is
            // asserted here is that three specific strings are NOWHERE in the face.
            assert!(
                !stderr.contains(needle),
                "stone E is back: `{needle}` reached a wat user's stderr from fixture `{case}`.\n\
                 stderr:\n{stderr}"
            );
        }
    }
}
