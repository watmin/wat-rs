//! A PARITY SCRIPT NOBODY RUNS IS NOT A GATE — it is a file that looks like one.
//!
//! `PERF-ARC` states this arc's closing condition as *"differential-tested bit-for-bit against the
//! wat oracle AND benched at or past Clara"*. The oracle half was gated in nextest. **The Clara
//! half was invoked by no job at all**: the scripts need a JDK the runner did not have, so a
//! Clara-parity regression merged fully green. `run-all.sh`'s own header records that having
//! already happened once, with four axes dead for days.
//!
//! Wiring CI once does not close that. The wiring is a line in a YAML file, and the failure this
//! gate exists for is the line going away — or, exactly as happened here, a NEW parity script
//! landing that nobody thinks to wire. `check-query-compat.sh` was found while fixing the first
//! problem: a working three-way gate (Clara == oracle == native, 24 rows across three query
//! families) referenced by ZERO files in the tree.
//!
//! ## Discovery, not a list
//!
//! The directory is WALKED. A list cannot notice what was never added to it — which is the whole
//! defect. Same doctrine as `wat_scripts_grid_axes_live`'s axis discovery and
//! `gen_doc_surface_matches`'s verb surface: the disk is the population, and the gate reconciles
//! against it.
//!
//! ## What counts as "invoked"
//!
//! Named by the CI workflow, or named by a Rust test. Both are real invocation paths and both are
//! gated: `check-spec-native.sh` is driven from `tests/rete/wat_scripts_grid_axes_live.rs` rather
//! than from CI directly, and that is fine — the question this asks is whether SOMETHING runs it,
//! not which thing.

use std::path::{Path, PathBuf};

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Every `check-*.sh` under the grid — the parity scripts, discovered.
fn parity_scripts() -> Vec<String> {
    let dir = repo("wat-scripts/perf/grid");
    let mut out: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|e| e.expect("dir entry").file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("check-") && n.ends_with(".sh"))
        .collect();
    out.sort();
    out
}

/// Scripts whose invocation is SUPERSEDED, each with the gate that replaced it.
///
/// Not a general escape: the reason names a REAL gate that runs the same comparison, and if that
/// gate is ever removed this row is a lie a reader can check in one grep. Asserted for exact
/// equality against the orphan set below, so a row that stops being needed goes red too — the
/// `SIZED_AXES` doctrine, where the array IS the population.
const SUPERSEDED: &[(&str, &str)] = &[(
    "check-spec-native.sh",
    "spec vs native on the where-family — implemented NATIVELY by      `tests/rete/wat_scripts_grid_axes_live.rs::spec_equals_native_on_every_where_family`, which      rewrites each axis to `fire-rules$oracle` in-process and diffs the rows. Wiring the shell      twin into CI as well would run the same comparison twice and call the duplication coverage.",
)];

/// Text of every place that could legitimately invoke one, COMMENTS STRIPPED.
///
/// ⚠ THE COMMENT-STRIPPING IS THE WHOLE GATE, and it was learned by mutation. The first version of
/// this file concatenated raw file text — and passed with the invocation deliberately deleted from
/// `ci.yml`, because THIS FILE names the script in its own doc comment. A gate that reads its own
/// prose as evidence certifies nothing. `check-spec-native.sh` was passing for the same reason:
/// its only mention anywhere is a `//!` line in the grid test.
fn invocation_surface() -> String {
    fn strip_comments(text: &str, markers: &[&str]) -> String {
        text.lines()
            .map(|l| {
                let mut cut = l.len();
                for m in markers {
                    if let Some(at) = l.find(m) {
                        cut = cut.min(at);
                    }
                }
                &l[..cut]
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    let yml = std::fs::read_to_string(repo(".github/workflows/ci.yml"))
        .expect("read .github/workflows/ci.yml — CI config is part of the gate surface");
    // YAML comments are `#`; a `#` inside a `run:` line would only ever be a shell comment, which
    // is equally not an invocation.
    let mut blob = strip_comments(&yml, &["#"]);

    let mut stack = vec![repo("tests")];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.file_name().is_some_and(|n| n == "every_parity_script_is_invoked.rs") {
                // THIS FILE IS EXCLUDED, and that is load-bearing. It names every script twice
                // over — in the doctrine above and, worse, as a STRING LITERAL in the SUPERSEDED
                // table, which comment-stripping cannot remove. Including it made the gate read
                // its own exemption row as proof the script was invoked. A gate may not be its own
                // evidence; both self-satisfying paths were found by MUTATION (deleting a real
                // invocation and watching the test stay green), never by reading the code.
                continue;
            } else if p.extension().is_some_and(|x| x == "rs") {
                if let Ok(t) = std::fs::read_to_string(&p) {
                    blob.push_str(&strip_comments(&t, &["//"]));
                    blob.push('\n');
                }
            }
        }
    }
    blob
}

#[test]
fn every_parity_script_is_invoked_by_ci_or_a_test() {
    let scripts = parity_scripts();
    assert!(
        scripts.len() >= 3,
        "found only {} parity script(s) under wat-scripts/perf/grid — the glob went blind, which \
         would make this gate pass vacuously; fix the walk rather than the assertion: {scripts:?}",
        scripts.len()
    );

    let surface = invocation_surface();
    let orphans: Vec<String> = scripts
        .iter()
        .filter(|s| !surface.contains(s.as_str()))
        .cloned()
        .collect();

    // The superseded set is asserted EXACTLY equal to the orphan set: an unrun script missing from
    // the table is red, and a table row for a script that IS run is red too — a stale excuse
    // outliving its reason is what `excusare` hunts.
    let mut expected: Vec<String> = SUPERSEDED.iter().map(|(n, _)| (*n).to_string()).collect();
    expected.sort();
    let mut got = orphans.clone();
    got.sort();
    assert_eq!(
        got, expected,
        "the parity scripts nothing invokes do not match the SUPERSEDED table in \
         tests/lint/every_parity_script_is_invoked.rs.\n\
         \n\
         A script here and not in the table is invoked by NOTHING — not by ci.yml, not by any test \
         — and a parity script nobody runs is not a gate, it is a file that looks like one. A row \
         in the table for a script that IS invoked is a stale excuse; delete the row.\n\
         \n\
         Mentions inside COMMENTS do not count, deliberately: this gate once passed on its own \
         doc comment."
    );

    assert!(
        orphans.is_empty() || !SUPERSEDED.is_empty(),
        "{} parity script(s) are invoked by NOTHING — not by .github/workflows/ci.yml, not by any \
         test under tests/: {:?}\n\
         \n\
         A parity script nobody runs is not a gate, it is a file that looks like one. This arc's \
         closing condition IS Clara agreement, and it went unchecked in CI for the whole arc \
         because the scripts needed a JDK the runner lacked. Wire it into the parity job in \
         ci.yml, or delete the script — a third option, leaving it to look like coverage, is the \
         one this gate exists to remove.",
        orphans.len(),
        orphans
    );
}
