//! PROBE — the `:-` TYPE POSITION already has its own validating authority, and it is NOT
//! `is_resolvable_call_head`.
//!
//! Measured on the green tree at `6f3019833` (5292/5292, clippy 0).
//!
//! ## Why this probe exists
//!
//! Arc 255's blanket census leaves **14 names in 97 files** once the reserved-prefix blanket
//! becomes a registry GATE that falls through. Four of the fourteen are type names:
//!
//! ```text
//!   wat.type/Tuple 9 · wat.type/i64 7 · wat.type/String 5 · wat.type/Vector 2
//! ```
//!
//! and every one is refused with the context string
//! `"namespaced symbol ref — not a builtin, not a registered function (arc 251)"` —
//! `src/resolve/normalize.rs:461`, **never** `walk.rs`'s `"call head …"`.
//!
//! `normalize_form`'s List arm classifies its `Boundary` only when `items.first()` is a
//! `Keyword`. In `(wat.type/Tuple :- [wat.type/i64])` the head is a **Symbol**, so the form
//! falls to `Boundary::Ordinary`, every child is walked as live code, and each namespaced
//! symbol reaches `resolve_namespaced_symbol` — which asks `is_resolvable_call_head`
//! *"may this symbol be rewritten to this keyword FQDN?"* about a **TYPE**.
//!
//! `walk.rs:87` has carried the arc-109 `:-` type-reference guard since 109. `normalize.rs`
//! runs FIRST and has none. It is the THIRD independent consumer of that shape, and the
//! walk.rs comment already names the class:
//! *"the expander was taught this first; the resolver is a SECOND, INDEPENDENT consumer of
//! the same shape and was not."*
//!
//! ## ⛔ WHAT THIS PROBE PINS, AND WHAT SABOTAGE PROVED ABOUT IT
//!
//! The obvious row — *"the four names resolve"* — is **VACUOUS**: they resolve today (the
//! blanket accepts them) and they resolve after the stone. It could not fail.
//!
//! The six RED rows below are the **justification's** controls, not the stone's guard. They
//! prove arc 296 P-1's annotation wall (`UnknownNamedType`) covers all five `:-` positions,
//! which is *why* `normalize` may stop asking a call-head question here at all. They must not
//! rot; they are not what catches a bad stone.
//!
//! ⛔⛔ I FIRST CLAIMED THEY WERE THE GUARD — that an over-broad stone (skip the whole `:-`
//! subtree) would let a bogus type through and turn them green. **Sabotage refuted that: with
//! the subtree skipped wholesale, all six stayed RED.** The wall validates independently of
//! whether `normalize` rewrote anything, so no normalize bug can reach it. A guard is not a
//! guard until it has failed once, and these had not.
//!
//! ★★★ THE REAL FAILURE MODE, FOUND BY THAT SABOTAGE — a `:-` form is **not all types**.
//! `(:wat::core::HashSet :- [T] v1 v2)` carries a type-argument vector AND live VALUE
//! arguments. An over-broad stone leaves a namespaced symbol in those values un-rewritten,
//! and the result:
//!
//! ```text
//!   wat --check   EXIT=0                                    ← SILENT
//!   wat  (run)    #wat.runtime/UnboundSymbol "wat.core/str" ← the defect
//! ```
//!
//! `control_values_after_the_type_vector` is that row, and it is the ONLY row in this file
//! that goes red under the over-broad implementation. It is RUN, not checked — nine `--check`
//! rows and a green floor once shipped a stone whose program died on line three.
//! `[[feedback_a_green_test_can_prove_nothing]]`
//!
//! ⛔ AND THE OBVIOUS FIX IS DISCONFIRMED. Measured by dumping `TypeEnv` at step 7, on the
//! same tree, immediately before `normalize_symbol_refs` runs:
//!
//! ```text
//!   :wat::core::i64      contains=true      :wat::core::Tuple   contains=FALSE
//!   :wat::core::String   contains=true      :wat::type::Infer   contains=FALSE
//!   :wat::core::Vector   contains=true
//! ```
//!
//! So *"validate the `:-` position against the type registry"* refuses `Tuple` (9 of the 23
//! sites, the largest family) and `:wat::type::Infer` (46 corpus occurrences, a LIVE marker at
//! `src/types.rs:74`). `Tuple` is structural type SYNTAX, not a named type. The two control
//! rows below hold that door shut.
//!
//! ## ⚠ WHAT CAPTURING THE GOLDENS REVEALED — a finding this probe now PINS
//!
//! The six RED rows were captured as six separate `.edn` goldens. **All six came back
//! byte-identical, 287 bytes:**
//!
//! ```text
//!   #wat.type/UnknownNamedType {:message "annotation names unknown type :wat::core::Bogus …"
//!     :location #wat.core/Span {:file "src/check.rs" :line 15007 :col 13 …} …}
//! ```
//!
//! Five distinct declaration forms — param, return, defstruct field, defenum variant field,
//! and the keyword spelling — at six distinct source positions, produce ONE indistinguishable
//! diagnostic whose `:location` is **the checker's own source file**, not the user's span. A
//! program with two bad annotations cannot tell the reader which one it means.
//!
//! That is not this stone's to fix (arc 296 owns diagnostics), and it is not a reason to hold
//! this stone. It is pinned here: ONE golden serves all six rows, so the day the diagnostic
//! learns the user's span, all six go red and whoever fixes it must split the golden.
//!
//! ## The rows
//!
//! Six RED (the wall must not move) · two GREEN (the fix must not narrow). Every row runs
//! against the real binary, so a mis-aimed harness shows up as a control going the wrong way
//! rather than as a silent pass.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn check(case: &str) -> i32 {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/resolve")
        .join(format!(
            "probe_arc255_the_type_position_has_its_own_authority__{case}.wat"
        ));
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

fn check_output(case: &str) -> String {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/resolve")
        .join(format!(
            "probe_arc255_the_type_position_has_its_own_authority__{case}.wat"
        ));
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg("--check")
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat --check");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// The wall's own diagnostic, compared STRUCTURE-EXACT against the golden — not merely the
/// exit code, which any error at all satisfies (including the resolve refusal this stone
/// removes).
///
/// ⛔ ONE golden serves all six rows, and that is a FINDING, not a shortcut: see the file
/// header. When the diagnostic learns the user's span, these rows go red and the golden must
/// SPLIT — which is exactly the alarm wanted.
fn refused_as_unknown_named_type(case: &str) {
    assert_eq!(check(case), 1, "{case}: expected the type wall to refuse");
    wat::assert_edn_eq!(
        check_output(case),
        include_str!("probe_arc255_the_type_position_has_its_own_authority__unknown_named_type.edn")
    );
}

/// The head of a `:-` type reference, in param position.
#[test]
fn a_bogus_type_in_the_head_of_a_binder_form_is_refused() {
    refused_as_unknown_named_type("bogus_in_head");
}

/// A type ARGUMENT under a legitimate head — the position the DESIGN named.
#[test]
fn a_bogus_type_in_a_type_argument_is_refused() {
    refused_as_unknown_named_type("bogus_in_arg");
}

/// Return position.
#[test]
fn a_bogus_type_in_return_position_is_refused() {
    refused_as_unknown_named_type("bogus_in_return");
}

/// A `defstruct` field annotation.
#[test]
fn a_bogus_type_in_a_defstruct_field_is_refused() {
    refused_as_unknown_named_type("bogus_in_defstruct_field");
}

/// A `defenum` variant field annotation.
#[test]
fn a_bogus_type_in_a_defenum_variant_field_is_refused() {
    refused_as_unknown_named_type("bogus_in_defenum_field");
}

/// ★ THE ISOLATING ROW. `normalize` rewrites `WatAST::Symbol` nodes ONLY, so a KEYWORD-spelled
/// `:-` form never passes through `resolve_namespaced_symbol` at all. This row therefore
/// measures the wall with the stone's mechanism entirely out of the picture: it must be red
/// before and after, and if it ever diverges from its Symbol-spelled sibling
/// (`bogus_in_head`), the two spellings have stopped agreeing.
#[test]
fn the_keyword_spelling_is_refused_by_the_same_wall() {
    refused_as_unknown_named_type("bogus_keyword_spelling");
    assert_eq!(
        check("bogus_keyword_spelling"),
        check("bogus_in_head"),
        "the Symbol and Keyword spellings of one shape must reach the same verdict"
    );
}

/// CONTROL — the four names of the census, in real `:-` use, including a nested parametric.
/// Green now; green after. A stone that refuses these has broken the corpus.
#[test]
fn the_four_census_names_are_accepted_in_real_use() {
    assert_eq!(check("control_the_four_names"), 0);
}

/// ⛔⛔ CONTROL — THE TRAP. `:wat::type::Infer` is a type-position MARKER, not a declared type;
/// `TypeEnv::contains` answers **false** for it, and it has 46 occurrences in the corpus. Any
/// implementation that validates a `:-` argument against the type registry refuses this file.
/// Green now, and it is the reason the stone cannot be "ask the type store instead."
#[test]
fn the_infer_marker_is_not_a_registered_type_and_must_stay_accepted() {
    assert_eq!(
        check("control_infer_marker"),
        0,
        ":wat::type::Infer is a marker (src/types.rs:74), not a TypeEnv member"
    );
}

/// ★★★ THE LOAD-BEARING GUARD — and the only row here proven to fail under a plausible wrong
/// stone. It RUNS the program: the over-broad implementation (skip the whole `:-` subtree)
/// leaves `wat.core/str` un-rewritten among the constructor's VALUE arguments, which
/// `--check` reports as EXIT 0 and the runtime reports as `UnboundSymbol`.
#[test]
fn value_arguments_after_the_type_vector_are_still_normalized_and_the_program_runs() {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/resolve")
        .join(
            "probe_arc255_the_type_position_has_its_own_authority__\
             control_values_after_the_type_vector.wat",
        );
    assert!(path.exists(), "fixture missing: {}", path.display());
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code().unwrap_or(-1),
        0,
        "program must RUN, not merely check.\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    // A Vector, not a HashSet: the rendering is order-deterministic, so the WHOLE value is
    // pinned rather than probed with `contains`. `println` EDN-encodes `show`'s String, so the
    // inner quotes arrive escaped.
    assert_eq!(
        stdout, "\"[\\\"1\\\", \\\"b\\\"]\"\n",
        "the value arguments must survive normalization and the program must print them"
    );
}
