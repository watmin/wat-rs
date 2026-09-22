//! THE FINDINGS BOARD — what `the-little-wat` found, measured at OUR head, one row per finding.
//!
//! `~/work/holon/the-little-wat` worked Friedman's *Little* books and a dozen more suites through
//! wat **to find where wat breaks**: 172 distinct findings, each with a repro among 364 probes,
//! every diagnostic quoted verbatim. Its own rule is *"nothing is claimed from wat-rs's docs;
//! every 'wat can't' is a probe that ran."*
//!
//! ⛔ **But it is a SNAPSHOT** — stamped `2026-09-15, wat-rs a3218644d` — and its entries carry no
//! status field. Its only automated OPEN/FIXED signal, `tools/recheck.sh`, covers 11 of the 172,
//! lives in a repo another session is actively pushing, and is blind in at least one place we can
//! name (see F-083 below). **Nothing over there can tell a cured finding from a live one at our
//! HEAD.** This board is that instrument, on our side of the fence, where our floor runs it.
//!
//! ## What a row asserts, and what it deliberately does not
//!
//! A row is a fixture plus the `(check rc, run rc)` pair measured at HEAD, plus — where exit codes
//! cannot see the defect — the stdout that carries it. It records **what is true today**. It
//! rules NOTHING about what should be true: nearly every cure on this board is a small
//! language-design decision, not a mechanical fix (their own relay files F-045 under *Fix* —
//! refuse `first` on an empty collection at the checker — AND under *Extend* — make `first`/`rest`
//! total, like `last`. Those are opposite languages.) The board exists so that whoever makes that
//! ruling can see the ruling land.
//!
//! **A red here is therefore not automatically a regression.** Read the shape:
//!
//! - a `LENIENT` or `STRICT` row going red usually means the finding was CURED — go close it in
//!   `the-little-wat/FINDINGS.md`, and update the row to the cured verdict;
//! - **F-093 is the exception.** It is already cured (`bc93125aa`, arc 8c); its row is a
//!   REGRESSION PIN, and a red there means the clj-spelled call site stopped being type-checked.
//!
//! ## ⛔ THE DRIVER IS THE BINARY, TWICE — AND THAT IS A CONTRACT DECISION, NOT A DETAIL
//!
//! `tests/lint/every_wat_bad_fixture_actually_fails.rs` records that the binary and the in-process
//! `startup_from_file` give **OPPOSITE verdicts on the same files**, and that a prior strike was
//! withdrawn for choosing wrong. Every finding here is a claim about what a user experiences
//! running `wat foo.wat`, and `startup_from_file` **cannot express the second half of the pair at
//! all** — it does not evaluate. Ported onto it, the whole LENIENT class (checker accepts, runtime
//! kills) would read as ABSENT. The helpers below are the banked probe's
//! (`tests/diagnostics/probe_ex003_silent_failure_pair.rs`, `68e7de0d3`), unchanged except for the
//! fixture directory and stem prefix; that probe's header carries the full reasoning.
//!
//! ## Why these fixtures are `.wat` and never `.wat.bad`
//!
//! `.wat.bad` is a claim that a file fails at **STARTUP**, enforced through `startup_from_file`.
//! Three of these eight start up clean and die at EVAL, and one (F-083) never fails at all. `.wat`
//! is the honest extension, and `tests/` is excluded from `every_ungated_wat_checks`
//! (`GATED_PREFIXES`), so a fixture that dies at eval is safe to keep here — verified, not assumed.
//!
//! ## Relationship to the banked probe — breadth, not duplication
//!
//! `probe_ex003_silent_failure_pair.rs` stays and is untouched. It is the DEPTH example: it alone
//! asserts F-031's death happens for F-031's *own* reason. This board is the BREADTH. Its
//! `expect_stderr` column is a deliberately weaker instrument — a targeted fragment of a large,
//! span-bearing EDN blob whose file paths and line/col numbers make byte-equality impossible.
//!
//! ## What this board CANNOT see
//!
//! - **A wrong value behind a zero exit.** F-083 is the specimen and the warning: `(0, 0)` with
//!   nothing pinned is a row that can never fail. [`no_row_can_be_vacuous`] makes that shape
//!   unrepresentable, by an assertion over the table rather than by convention.
//! - **Findings that are not program-level.** F-085 (a doc claim), F-087 (86 of 203 doc names),
//!   F-089 (the CLI surface), F-096 (a read-ratio benchmark) are three OTHER instruments. They are
//!   rejected from this board, not deferred into it.
//! - **Any count.** There is no pinned row length. A pinned length is right for a quarantine that
//!   must SHRINK; this board must GROW, and a count would cap coverage downward while looking like
//!   rigour. Adding a finding is adding a row.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// What kind of failure a row records — and it is load-bearing, not a label: each variant
/// constrains the `(check, run)` pair it may sit beside (see [`every_shape_agrees_with_its_pair`]),
/// so a row relabelled without re-measuring goes red.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// The checker ACCEPTS and the runtime KILLS — the silent-failure class, the one their author
    /// says cost most, *"because a model trusts a green run."*
    Lenient,
    /// The checker REFUSES, so the binary never evaluates. The user is told, at check time.
    Strict,
    /// Both halves exit clean and the ANSWER is wrong. Exit codes are blind to this; only the
    /// pinned stdout sees it.
    WrongAnswer,
}

/// One finding's measured verdict.
///
/// ⛔ NAMED FIELDS, NOT A TUPLE, AND THAT IS NOT A STYLE PREFERENCE. This stone's brief sketched
/// the row as a 7-tuple; `clippy::type_complexity` REFUSES that at `-D clippy::all`, and the
/// accessors it forces (`r.2`, `r.5`) are exactly the shape in which a column is read for the
/// wrong one. `check`/`run` are two `i32`s side by side; `stderr_fragment`/`stdout` two
/// `Option<&str>`s side by side. A tuple lets either pair swap silently.
struct Row {
    /// The finding's id in `the-little-wat/FINDINGS.md`.
    id: &'static str,
    /// The fixture stem — the file is `little_wat_findings_board__<stem>.wat`, beside this one.
    fixture: &'static str,
    /// Exit code of `wat --check <fixture>`.
    check: i32,
    /// Exit code of `wat <fixture>`.
    run: i32,
    /// A fragment of the run's stderr naming the DEFECT's own words, so a death for some unrelated
    /// reason still reds. `None` where the program does not die.
    stderr_fragment: Option<&'static str>,
    /// The run's stdout, pinned exactly. Required — see [`no_row_can_be_vacuous`] — wherever the
    /// pair is `(0, 0)`, because there the value is the only evidence there is.
    stdout: Option<&'static str>,
    /// The class this row belongs to; checked against the pair by [`every_shape_agrees_with_its_pair`].
    shape: Shape,
}

/// The board. Measured on this branch's build of `main` @ `600abe8c3`, binary freshly built, each
/// fixture run twice by hand before this table was written. Adding a finding is adding a row.
const BOARD: &[Row] = &[
    Row {
        id: "F-031",
        fixture: "f031_length_on_string",
        check: 0,
        run: 1,
        stderr_fragment: Some(":wat::core::length: expected"),
        stdout: None,
        shape: Shape::Lenient,
    },
    Row {
        id: "F-045",
        fixture: "f045_first_of_empty_vector",
        check: 0,
        run: 1,
        stderr_fragment: Some(":wat::core::first: sequence has 0 element(s); no element at index 0"),
        stdout: None,
        shape: Shape::Lenient,
    },
    Row {
        id: "F-090",
        fixture: "f090_rational_to_f64_on_collapse",
        check: 0,
        run: 1,
        stderr_fragment: Some("expected rational, got wat::core::bigint"),
        stdout: None,
        shape: Shape::Lenient,
    },
    Row {
        id: "F-058",
        fixture: "f058_persistentmap_nested_bracketed_type",
        check: 1,
        run: 3,
        stderr_fragment: Some("bracketed type must be a type keyword"),
        stdout: None,
        shape: Shape::Strict,
    },
    Row {
        id: "F-080",
        fixture: "f080_filterv_on_persistentvector",
        check: 1,
        run: 3,
        stderr_fragment: Some("no clause of `:wat::core::filterv` matches arity 2"),
        stdout: None,
        shape: Shape::Strict,
    },
    Row {
        id: "F-088",
        fixture: "f088_stream_collect_absent",
        check: 1,
        run: 3,
        stderr_fragment: Some("not a builtin, not a registered function"),
        stdout: None,
        shape: Shape::Strict,
    },
    // ⛔ CURED, and on the board precisely because it is. Their snapshot recorded this running to
    // exit 0 with an unchecked clj-spelled call; `bc93125aa` (arc 8c) closed it. The row is the
    // regression pin — a red here means the cure came undone, not that a finding was fixed.
    Row {
        id: "F-093",
        fixture: "f093_clj_spelled_wrong_typed_arg",
        check: 1,
        run: 3,
        stderr_fragment: Some(":bad::ident: parameter #1 expects :wat::core::i64; got :wat::core::String"),
        stdout: None,
        shape: Shape::Strict,
    },
    // ⛔ THE ROW EXIT CODES CANNOT SEE. Two puts of one key, then `len` — the correct answer is
    // `1` and wat prints `0`, cleanly, twice green. The pinned stdout is the ONLY assertion here
    // with any content; without it this row would be unfailable. See `no_row_can_be_vacuous`.
    Row {
        id: "F-083",
        fixture: "f083_holographic_lru_reput",
        check: 0,
        run: 0,
        stderr_fragment: None,
        stdout: Some("0"),
        shape: Shape::WrongAnswer,
    },
];

/// Copied from `tests/diagnostics/probe_ex003_silent_failure_pair.rs` (`68e7de0d3`) — only the
/// directory and the stem prefix differ. Do not re-derive the driver decision; read that header.
fn fixture(case: &str) -> PathBuf {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/lint")
        .join(format!("little_wat_findings_board__{case}.wat"));
    assert!(p.exists(), "fixture missing: {}", p.display());
    p
}

/// One invocation of the binary: `wat --check <f>` when `check_only`, else `wat <f>`.
/// Returns `(exit code, stdout, stderr)`. Same `Command` construction as the banked probe's
/// `rc()`/`stderr_of()`, widened to return all three because F-083's whole content is a stdout.
fn invoke(case: &str, check_only: bool) -> (i32, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wat"));
    if check_only {
        cmd.arg("--check");
    }
    let out = cmd
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

/// Every row still reads the verdict it was measured at.
#[test]
fn every_row_holds_its_measured_verdict() {
    // NON-VACUITY: the population is this file's own const table, so no walk can silently reach
    // zero — but an edit can, and an empty BOARD would make the loop below assert nothing over
    // nothing and report PASS. Deliberately NOT a pinned length (this board must grow; see the
    // header): emptiness is the failure mode, a changed count is not.
    assert!(
        !BOARD.is_empty(),
        "BOARD is empty — this gate now measures nothing. Restore the rows or delete the gate; \
         an empty board that reports PASS is worse than no board."
    );

    for row in BOARD {
        let id = row.id;
        let (check_rc, _, _) = invoke(row.fixture, true);
        assert_eq!(
            check_rc, row.check,
            "{id}: `wat --check` now exits {check_rc}, not {}. The CHECKER's verdict on this \
             finding moved. Go read the finding in the-little-wat/FINDINGS.md: either it was cured \
             (update this row to the new pair and close it there) or the fixture rotted.",
            row.check
        );

        let (run_rc, out, err) = invoke(row.fixture, false);
        assert_eq!(
            run_rc, row.run,
            "{id}: `wat` now exits {run_rc}, not {}. The RUNTIME's verdict on this finding moved. \
             Same two possibilities, same first move: read the finding, do not adjust the number.\n\
             --- stdout ---\n{out}\n--- stderr ---\n{err}",
            row.run
        );

        if let Some(frag) = row.stderr_fragment {
            // A fragment, not byte-equality: the diagnostic is a span-bearing EDN blob carrying
            // this file's own path and line/col numbers, so an exact match would pin the fixture's
            // line breaks rather than the defect. The fragment names the DEFECT's own words, so a
            // death for some unrelated reason (a retired spelling, a missing main) still reds —
            // `[[a-negative-fixture-can-fail-for-the-wrong-reason]]`.
            let names_its_own_reason = err.contains(frag);
            assert!(
                names_its_own_reason,
                "{id}: the failure no longer names its own reason. Expected to find:\n  {frag}\n\
                 The program failed some OTHER way, which means this row is measuring something \
                 else. Full stderr:\n{err}"
            );
        }

        if let Some(expected) = row.stdout {
            assert_eq!(
                out.trim(),
                expected,
                "{id}: the printed ANSWER changed. This row exists because exit codes cannot see \
                 this finding — the value IS the measurement. If the answer is now correct, the \
                 finding is cured: close it and retire this row."
            );
        }
    }
}

/// ⛔ A ROW ASSERTING `(0, 0)` WITH NOTHING PINNED CAN NEVER FAIL.
///
/// It is coverage-shaped and measures nothing: both halves exit clean, so both `assert_eq!`s above
/// are satisfied by any program at all — including one that no longer reproduces the finding.
/// F-083 is the live subject: its whole content is a wrong VALUE behind two green exits.
///
/// ⚠ **This test has a live subject only while a `(0, 0)` row is on the board.** F-083 is that
/// subject today. If F-083 is ever cured and retired and no other `(0, 0)` row has arrived, this
/// test becomes vacuous ITSELF and must be re-derived or removed — named here so it is a decision
/// later rather than a surprise.
#[test]
fn no_row_can_be_vacuous() {
    let mut subjects = 0usize;
    for row in BOARD {
        if row.check == 0 && row.run == 0 {
            subjects += 1;
            assert!(
                row.stdout.is_some(),
                "{} asserts (0, 0) and pins no stdout — a row that CANNOT FAIL. Both exit codes \
                 are clean, so this row would stay green after the finding was cured, after the \
                 fixture rotted, and after the program was replaced by an empty main. Pin the \
                 value the finding is about, or take the row off the board.",
                row.id
            );
        }
    }
    // NON-VACUITY: with no (0, 0) row the loop above asserts nothing, and this test would report
    // PASS while proving the rule against no subject at all. Say so out loud (see this test's doc
    // comment) rather than discovering it the day F-083 is cured.
    assert!(
        subjects > 0,
        "no (0, 0) row is on the board, so this rule has no subject and this test is now vacuous \
         itself. Re-derive it against the rows that exist, or remove it — do not leave it green."
    );
}

/// A row's [`Shape`] must agree with the pair beside it, so a label cannot drift off the
/// measurement it names. Relabelling without re-measuring is the cheap mistake this catches.
#[test]
fn every_shape_agrees_with_its_pair() {
    for row in BOARD {
        let id = row.id;
        let (check, run) = (row.check, row.run);
        match row.shape {
            Shape::Lenient => assert!(
                check == 0 && run != 0,
                "{id} is labelled LENIENT (checker accepts, runtime kills) but reads ({check}, \
                 {run}). Re-measure and relabel — the label is a claim about the class."
            ),
            Shape::Strict => assert!(
                check != 0,
                "{id} is labelled STRICT (the checker refuses) but `--check` reads {check}. A \
                 STRICT finding never reaches the runtime."
            ),
            Shape::WrongAnswer => assert!(
                check == 0 && run == 0 && row.stdout.is_some(),
                "{id} is labelled WRONG-ANSWER but reads ({check}, {run}) with stdout pinned = \
                 {}. That label means BOTH halves exit clean and the printed value is the whole \
                 evidence.",
                row.stdout.is_some()
            ),
        }
    }
}
