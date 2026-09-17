//! The boot census, end to end — excursus 001 `boot-names-where-its-time-goes`.
//!
//! ⛔ WHY THIS TEST EXISTS, and why it cannot live in `src/`. The census
//! (`src/freeze/census.rs`) has unit tests for its wording, its parsing and its accumulator, and
//! every one of those passes on an instrument that is wired to NOTHING. The claim this stone
//! actually makes — *"arming `WAT_BOOT_CENSUS=files` names which of the 55 baked manifest entries
//! boot spends its time in"* — is a claim about the real pipeline, and the only way to falsify it
//! is to arm the census, run a real freeze, and read the table back. `report` writes to the real
//! fd 2 where a test cannot see it, hence `rendered_report`.
//!
//! Precedent: `ring_census_counts_every_ring_it_hands_out` (`src/comms/process.rs`) — a census
//! that is not demonstrated to COUNT is decoration.
//!
//! ⚠ PER-TEST PROCESS ISOLATION IS LOAD-BEARING HERE. The census mode is a process-global
//! `OnceLock` (deliberately: a diagnostic that changes mid-run is a different bug), so this test
//! must be the first thing in its process to resolve it. nextest forks a process per test —
//! `.config/nextest.toml`'s own header states that contract — which is the floor. The
//! `force_mode` assertion below is what makes a violation a RED instead of a silent skip.

use std::time::Duration;

use wat::freeze::census::{force_mode, rendered_report, totals, Mode};

/// ⭑ The armed census must attribute real boot time to real manifest entries — by name.
#[test]
fn an_armed_boot_census_names_the_manifest_entries_it_spent_time_in() {
    assert!(
        force_mode(Mode::Files),
        "the census mode was already resolved in this process, so this test did not measure an \
         armed boot. It requires per-test process isolation (nextest forks per test); under a \
         shared-process runner it cannot be trusted and must not be read as a pass."
    );

    // A real freeze of a real program: the one-line probe the excursus measured by hand.
    let world = wat::freeze::startup_from_file("wat-scripts/probes/arc-170/probe-trivial.wat")
        .expect("the trivial probe must freeze");
    assert!(
        world.symbols().get(":user::main").is_some(),
        "the probe declares :user::main — if it did not, this boot froze the wrong thing"
    );

    let (accounted, files, guards) = totals();

    // 1. It COUNTED. A table of zeroes would satisfy every wording assertion in the unit tests.
    assert!(
        accounted > Duration::from_millis(10),
        "an armed census over a real freeze must account for real time; got {accounted:?}"
    );

    // 2. It attributed to FILES, and to essentially all of them. The manifest is 55 entries plus
    //    the user's own entry file; a curve that names three of them is not a curve. The bound is
    //    deliberately loose (the manifest grows) and one-sided (fewer is the failure).
    assert!(
        files >= 50,
        "the per-manifest-entry curve must cover the manifest, not a corner of it: {files} \
         distinct files attributed"
    );
    assert!(
        guards >= files as u64,
        "every attributed file needs at least one guard pair: {guards} pairs / {files} files"
    );

    let t = rendered_report();

    // 3. The phases named in the report are the ones the loader ACTUALLY has — the DESIGN's guess
    //    (parse/expand/check/freeze) is NOT the set, and a future refactor that renames a seam
    //    must break this rather than silently print a table missing the pass it renamed.
    for phase in [
        "3a   stdlib-parse",
        "4    stdlib-expand",
        "5    stdlib-types-register",
        "6    stdlib-defines-register",
        "8f   check:body-infer(ALL fns)",
        "9    frozen-world-freeze",
    ] {
        assert!(t.contains(phase), "phase {phase:?} absent from the report:\n{t}");
    }

    // 4. Real manifest entries, by path. `wat/core.wat` is manifest position 1 and can never not
    //    be loaded; `wat/service.wat` carries the `defservice` machinery this stone was drawn to
    //    weigh.
    for entry in ["wat/core.wat", "wat/service.wat", "wat/test.wat"] {
        assert!(t.contains(entry), "manifest entry {entry:?} unnamed in the report:\n{t}");
    }

    // 5. ⛔ The instrument must state its own size next to its output — otherwise a reader has no
    //    way to bound how much of the numbers above are the measurer.
    // rune:lint(loose-assert) — a REAL boot's report is a ~60-row table whose every ms varies per
    // run, so no whole-value `assert_eq!` exists to write. This is a targeted presence over a
    // large variable output; the report's exact WORDING is pinned byte-for-byte by the goldens in
    // `src/freeze/census.rs::tests` (`EXPECTED_FILES_REPORT`), which is where the strict form
    // belongs. Same reason for the two sites below.
    assert!(
        t.contains("file_guard"),
        "the report must state its own guard count:\n{t}"
    );

    // 6. And it must reconcile: the accounted total, the in-pipeline remainder and the
    //    pre-pipeline slice are three separate printed facts, not one number.
    for anchor in [
        "ACCOUNTED (sum of leaves)",
        "PIPELINE WALL",
        "unaccounted, in-pipeline",
        "SINCE PROCESS BOOT",
    ] {
        assert!(t.contains(anchor), "reconciliation line {anchor:?} missing:\n{t}");
    }

    // 7. ⛔ And no reading may be invisible: a phase recorded outside `PHASE_ORDER` would be
    //    excluded from every row and silently falsify the accounted total. The report shouts in
    //    that case; on a healthy run that shout must be ABSENT.
    // rune:lint(loose-assert) — a targeted ABSENCE over the same large per-run-variable output:
    // the claim is that this shout is not present, which no equality against a whole value can
    // express. The shout's own wording is pinned in `EXPECTED_STRAY_PHASE_REPORT`.
    assert!(
        !t.contains("ABSENT from PHASE_ORDER"),
        "a real boot recorded a phase the report cannot place — instrument bug:\n{t}"
    );
}
