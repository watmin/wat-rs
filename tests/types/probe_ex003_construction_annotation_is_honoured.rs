//! Probe (excursus 003, stone N) — **A CONSTRUCTION SITE'S EXPLICIT TYPE ARGUMENT IS HONOURED OR
//! REFUSED, NEVER IGNORED** (the-little-wat F-107 part 1).
//!
//! Before this stone, `(:T :- [A…] :field v …)` parsed and did nothing. The macro expander
//! (`src/macros/expand.rs`, the macro-dispatch arms) peeled the `:- [..]` off and handed the
//! kwargs companion only the field pairs, so `kwargs-construct` never saw it:
//!
//! | construction | `--check` before |
//! |---|---|
//! | `(:p::Ops :seed (:p::E.A {}) :step …)` | `parameter #2 expects [:p::E.A :-> :p::E.A]; got [:p::E :-> :p::E]` |
//! | same with `:- [:p::E]` | **identical** |
//! | same with `:- [:wat::core::i64]` — deliberately WRONG | **identical** |
//! | `(:t::Box :- [:wat::core::i64] :x "str")` in `main` | **checked clean, ran rc 0, stored a String** |
//!
//! The generic ENUM variant path dropped it too, at a different site: the call arm peeled and
//! parsed the spec, then `infer_enum_map_ctor` was never handed it —
//! `(:t::Opt.Some :- [:wat::core::i64 :wat::core::String :no::Such] {:v 1})` checked clean.
//!
//! ## The cure these fixtures pin
//!
//! The expander hands a macro its args RAW; the companion splices the spec into
//! `(:wat::core::kwargs-construct :T :- [A…] …)`; `infer_kwargs_construct_check` carries it onto
//! the synthetic prime call `(:T' :- [A…] …)`, whose call-position door already binds
//! (`instantiate_with_args`) and refuses too many / a non-generic `:T`. The enum path takes the
//! same rule. The runtime and the rete consumers of `kwargs-construct` peel it through the one
//! door (`types::peel_param_spec`) — a value carries no type parameters.
//!
//! ⚠ **Fewer arguments than declared is LEGAL**, by the existing call-position rule
//! (`STONE-guard-the-peel-point`, `check.rs`: `>` not `!=` — inference completes a partial
//! application, and `:- []` is the expressed empty binder). This probe pins the empty binder as
//! accepted; "wrong arity" here means TOO MANY.
//!
//! ## Known-open, pinned not cured
//!
//! `no_annotation_part2_open` — with NO annotation a bare variant literal still binds `T` to the
//! variant `:p::E.A` and the error blames `step` (F-107 parts 2 and 4; part 2 is F-019's
//! construction-site case). Its golden moves when that lands.
//!
//! ## Mutation (each site driven RED, one at a time, then restored byte-identical)
//!
//! | mutation | RED |
//! |---|---|
//! | expander strips again (`rest_after_marker`, both arms) | the 7 struct/record cells: `reported_case`, the 5 refusals, the run-time twin |
//! | checker drops `spec_nodes` from the synthetic prime call | the same 7 |
//! | `infer_enum_map_ctor` handed `None` | the 4 enum refusals |
//! | too-many message reports `k` (the prime) again | `too_many`, `non_generic` |
//! | runtime `eval_kwargs_construct` stops peeling | `reported_case`, `honoured_runs` |
//! | rete `lower_construct` stops peeling | both rete cells |
//! | rete `:then` validator stops peeling | `rete_then_nested` |
//!
//! Under the expander mutation `honoured_runs` stays green, as it should: its values agree with
//! their annotations, and its direct `kwargs-construct` line is not a macro call.
//!
//! Goldens are captured, never hand-authored:
//! `UPDATE_EDN=1 cargo nextest run --release -E 'test(/construction_annotation_is_honoured/)'`.

use std::path::PathBuf;
use std::process::{Command, Stdio};

const PREFIX: &str = "probe_ex003_construction_annotation_is_honoured";

fn fixture(file: &str) -> PathBuf {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("{PREFIX}__{file}"));
    assert!(p.exists(), "fixture missing: {}", p.display());
    p
}

/// One invocation of the binary. Returns `(exit code, stdout, stderr)`.
fn drive(check_only: bool, file: &str) -> (i32, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wat"));
    if check_only {
        cmd.arg("--check");
    }
    let out = cmd
        .arg(fixture(file))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
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

/// A program that must CHECK and RUN: exit 0, exact stdout, empty stderr.
fn assert_runs(case: &str, expected_stdout: &str) {
    let file = format!("{case}.wat");
    assert_eq!(
        drive(true, &file),
        (0, String::new(), String::new()),
        "`{case}` must check clean"
    );
    assert_eq!(
        drive(false, &file),
        (0, expected_stdout.to_string(), String::new()),
        "`{case}` must run to exactly this stdout with nothing on stderr"
    );
}

/// A program the checker must REFUSE: `--check` exits 1, stdout empty, stderr is the whole
/// `CheckErrors` face pinned as an EDN golden — which error fires, where, and what it names.
macro_rules! refusal_cell {
    ($name:ident, $case:literal) => {
        #[test]
        fn $name() {
            let (rc, stdout, stderr) = drive(true, concat!($case, ".wat.bad"));
            assert_eq!(
                (rc, stdout.as_str()),
                (1, ""),
                "`{}` must be refused by --check (exit 1). stderr:\n{stderr}",
                $case
            );
            wat::assert_edn_matches_file!(
                stderr,
                concat!(
                    "probe_ex003_construction_annotation_is_honoured__",
                    $case,
                    "_check.edn"
                ),
                "the refusal changed — check it still names the annotated mismatch at the field"
            );
        }
    };
}

// ── honoured ────────────────────────────────────────────────────────────────────────────────

/// F-107's reported case: `:- [:p::E]` pins T before `seed`'s variant literal can narrow it.
#[test]
fn the_reported_case_checks_and_runs() {
    assert_runs("reported_case", "\"A\"\n");
}

/// A struct, the empty binder, a two-param record with kwargs out of order, and the
/// `kwargs-construct` verb written directly — each checks and runs with its values intact.
#[test]
fn right_annotations_check_and_run() {
    assert_runs("honoured_runs", "7\n\"empty-binder\"\n\"one\"\n2\n9\n");
}

/// The generic enum variant path: a unit and a tagged variant, annotated.
#[test]
fn enum_variant_annotations_check_and_run() {
    assert_runs("enum_honoured", "1\n");
}

/// Rete `expr_ir` lowering of a rete defn body `(:rn::Rate :- [] :count k)`.
#[test]
fn a_rete_fn_body_construction_with_a_spec_still_lowers() {
    assert_runs("rete_fn_body", "5\n");
}

/// Rete `:then` validation of a nested `(:rt::Inner :- [] :n ?x)` operand.
#[test]
fn a_rete_then_nested_construction_with_a_spec_still_validates() {
    assert_runs("rete_then_nested", "1\n");
}

// ── refused ─────────────────────────────────────────────────────────────────────────────────

refusal_cell!(a_wrong_annotation_is_refused_at_the_field, "wrong_annotation");
refusal_cell!(a_wrong_value_under_an_annotation_is_refused, "box_wrong_value");
refusal_cell!(too_many_type_arguments_are_refused, "too_many");
refusal_cell!(an_annotation_on_a_non_generic_type_is_refused, "non_generic");
refusal_cell!(a_record_refusal_lands_on_the_right_field, "record_wrong_value");
refusal_cell!(an_enum_variant_wrong_value_is_refused, "enum_wrong_value");
refusal_cell!(an_enum_unit_variant_wrong_annotation_is_refused, "enum_unit_wrong");
refusal_cell!(too_many_enum_type_arguments_are_refused, "enum_too_many");
refusal_cell!(an_annotation_on_a_non_generic_enum_is_refused, "enum_non_generic");

// KNOWN-OPEN (F-107 part 2 / F-019): no annotation still narrows T to the variant.
refusal_cell!(no_annotation_still_narrows_to_the_variant_known_open, "no_annotation_part2_open");

/// The run-time half of `box_wrong_value`: it used to RUN and print `"str"`. Now startup refuses
/// it before `main` — nonzero exit, and the String never reaches stdout.
#[test]
fn a_wrong_value_under_an_annotation_no_longer_runs() {
    let (rc, stdout, _stderr) = drive(false, "box_wrong_value.wat.bad");
    assert_eq!(
        (rc, stdout.as_str()),
        (3, ""),
        "startup must refuse the program (exit 3) and nothing may reach stdout"
    );
}
