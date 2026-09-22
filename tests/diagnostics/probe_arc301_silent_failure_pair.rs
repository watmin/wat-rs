//! Probe (arc 301) — THE SILENT-FAILURE PAIR IS MACHINE-READABLE.
//!
//! The sibling repo `the-little-wat` worked 97 book chapters through wat to find where it
//! breaks, and its `FINDINGS.md` names the class that cost its author most: a program the
//! **checker accepts** and the **runtime kills**. Its own words: *"the silent failures,
//! because a model trusts a green run."*
//!
//! This probe banks the measurement that makes that class a GATE rather than a narrative:
//! the verdict of a finding in this class is the PAIR of exit codes
//!
//! ```text
//!   wat --check <file>   ->  rc 0   the checker accepts
//!   wat         <file>   ->  rc 1   the runtime rejects
//! ```
//!
//! and nothing else has to be parsed to read it. Measured on `main` @ `600abe8c3`.
//!
//! ## ⛔ THE DRIVER IS THE CONTRACT DECISION, NOT A DETAIL
//!
//! `tests/lint/every_wat_bad_fixture_actually_fails.rs` records that the binary and the
//! in-process driver `startup_from_file` give **OPPOSITE verdicts** on the same files, and
//! that a first draft of that strike was withdrawn for using the wrong one. This probe uses
//! **the binary, twice**, deliberately: every the-little-wat finding is a claim about what a
//! user experiences running `wat foo.wat`, and `startup_from_file` cannot express the second
//! half of the pair at all — it does not evaluate, so the "dies at runtime" half is invisible
//! to it. A port of these findings onto the in-process driver would silently measure only the
//! check half and report the class as absent.
//!
//! ## What this probe does NOT claim
//!
//! It banks ONE finding (F-031) as the worked reference. It does not survey the class, does
//! not assert a count, and does not claim the pair is the right instrument for the findings
//! that are about documentation (F-085), the CLI surface (F-089) or performance (F-096) —
//! those are different instruments and are affirmatively out of this probe's scope.
//!
//! ⛔ **This test goes RED the day F-031 is fixed, and that is the point.** It pins a KNOWN
//! DEFECT. A red here is not a regression: it is the signal to go close the finding, update
//! this assertion to the cured verdict, and relay the closure.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn fixture(case: &str) -> PathBuf {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/diagnostics")
        .join(format!("probe_arc301_silent_failure_pair__{case}.wat"));
    assert!(p.exists(), "fixture missing: {}", p.display());
    p
}

fn rc(case: &str, check_only: bool) -> i32 {
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
    out.status.code().unwrap_or(-1)
}

fn stderr_of(case: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(fixture(case))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// F-031 — `(length "abc")`: the checker accepts, the runtime kills.
///
/// This is the pair, and it is the whole probe.
#[test]
fn f031_length_on_a_string_is_accepted_by_the_checker_and_killed_by_the_runtime() {
    let case = "f031_length_on_string";
    assert_eq!(
        rc(case, true),
        0,
        "F-031 CURED at the checker: `wat --check` now refuses `(length \"abc\")`. \
         This is the signal to close the finding — update this assertion to the cured verdict."
    );
    assert_ne!(
        rc(case, false),
        0,
        "F-031 CURED at the runtime: `(length \"abc\")` now runs cleanly. \
         Go read the finding: either `length` gained a String clause, or the specimen rotted."
    );
}

/// The second half of the pair must fail for F-031's OWN reason, not for some other one.
///
/// Without this, any startup failure at all — a retired spelling, a missing main — would
/// satisfy `assert_ne!(rc, 0)` above and the probe would pass while measuring nothing.
/// `[[a-negative-fixture-can-fail-for-the-wrong-reason]]`.
#[test]
fn f031_the_runtime_death_is_a_length_type_mismatch_not_an_unrelated_failure() {
    let err = stderr_of("f031_length_on_string");
    assert!(
        err.contains("RuntimeError"),
        "expected a RuntimeError; the program failed some other way:\n{err}"
    );
    assert!(
        err.contains(":wat::core::length"),
        "expected the death to name :wat::core::length; got:\n{err}"
    );
}
