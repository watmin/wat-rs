//! Gate — every hunt/measurement TOOL under `wat-scripts/hunt/` must prove itself.
//!
//! WHY. `wat-scripts/` tooling is not compiled, not linted, and not loaded by any
//! other test — the same blind spot `wat_scripts_fixes_load.rs` was written to close
//! for `.wat` files ("a stale exemplar that no longer runs is a graveyard that reads
//! like live code"). A measuring tool is worse than a stale exemplar, because it does
//! not merely rot quietly: it emits NUMBERS, and those numbers steer work.
//!
//! This is not hypothetical. `fn-census.py` exists because the exemplar hunt's target
//! table was measured by hand and was wrong TWICE, in opposite directions — first
//! `fn`-line-to-end-of-file (swallowing the `#[cfg(test)] mod tests` below, reporting
//! 388/451/590-line bodies that are really 87/35/72), then starting AT the `fn` line
//! (so a function whose `///` block sits above it read as 0% comment). The first take
//! steered the campaign at the WRONG functions for three sessions. The second shipped
//! inside the tool built to fix the first.
//!
//! So the tool carries `--selftest`: fixtures for each of those incidents plus the
//! brace-in-string / brace-in-comment / char-literal / trait-signature cases its
//! scanner has to get right. This gate runs it, so the proof is on the floor rather
//! than in someone's memory of having checked once.
//!
//! ADDING A TOOL HERE. Give it a `--selftest` that exits non-zero on failure, then add
//! it to `TOOLS`. A tool with no self-test does not belong in `hunt/` — it belongs in a
//! scratch directory, where nobody will mistake its output for evidence.
//!
//! Run: cargo test --release -p wat --test lint hunt_tooling

use std::path::Path;
use std::process::Command;

/// (script under `wat-scripts/hunt/`, the flag that runs its self-test).
const TOOLS: &[(&str, &str)] = &[("fn-census.py", "--selftest")];

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn every_hunt_tool_passes_its_own_selftest() {
    for (script, flag) in TOOLS {
        let path = repo_root().join("wat-scripts/hunt").join(script);
        assert!(path.is_file(), "{script}: not found at {}", path.display());

        let out = Command::new("python3")
            .arg(&path)
            .arg(flag)
            .current_dir(repo_root())
            .output()
            .unwrap_or_else(|e| panic!("{script}: could not run python3: {e}"));

        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            out.status.success(),
            "{script} {flag} FAILED (exit {:?}).\n\
             The measurer disagrees with the incidents that produced it — trust the\n\
             self-test, not the tool, until this is green again.\n\
             --- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
            out.status.code()
        );
    }
}

/// The self-test must be able to FAIL. A gate that cannot go red is decoration, and
/// this whole tool exists because an unverified measurement was trusted.
///
/// The mutation is applied to a COPY in a temp dir; the real script is never touched.
#[test]
fn the_selftest_can_actually_fail() {
    let src = repo_root().join("wat-scripts/hunt/fn-census.py");
    let body = std::fs::read_to_string(&src).expect("read fn-census.py");

    // Reintroduce the take-2 defect: stop walking back over docs/attributes, so an
    // item's `///` block stops counting as part of it.
    let needle = "        while j >= 0 and LEAD_RE.match(lines[j]) and not in_skip(j):";
    assert!(
        body.contains(needle),
        "the mutation target moved; this gate is no longer proving anything — \
         re-point it at the backward doc-walk in fn-census.py"
    );
    let mutant = body.replace(needle, "        while False and j >= 0 and LEAD_RE.match(lines[j]) and not in_skip(j):");

    let dir = std::env::temp_dir().join(format!("fn-census-mutant-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let mutant_path = dir.join("fn-census-mutant.py");
    std::fs::write(&mutant_path, mutant).expect("write mutant");

    let out = Command::new("python3")
        .arg(&mutant_path)
        .arg("--selftest")
        .output()
        .expect("run mutant");
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        !out.status.success(),
        "a mutated fn-census.py — one that ignores doc comments above a fn, the exact \
         bug this tool shipped with — still PASSED its own self-test. The self-test is \
         not testing what it claims."
    );
}

// ── The grid's own reconciliation, lifted onto the floor ─────────────────────
//
// `run-all.sh` already does the hard half of this and says why, in a comment worth
// keeping: four axes once "sat DEAD for days — they died at rule-compile the hour law
// A armed and every gate stayed green, because the loader gate only PARSES and nothing
// ran this script." So it now DISCOVERS axes from disk and refuses to run when one has
// no size rung — "a gate that discovers beats one that lists."
//
// The gap is not that check; it is WHEN it happens. It fires only when a human runs the
// grid by hand, and the grid is in no CI job (the Clara half needs a JDK the runner does
// not have). So the reconciliation runs on the floor here instead — no JDK, no Clara, no
// grid run: pure consistency of the harness against itself.
//
// Two things `run-all.sh` does NOT check are checked here as well:
//   - a generator whose `(ns …)` or `:axis "…"` disagrees with its filename. `run-axis.sh`
//     invokes `-m "$AXIS"` and greps the emitted `:axis`, so a mismatch is a confusing
//     runtime failure rather than a clear one.
//   - a rung present in LADDER but absent from ORDER. ORDER is the DEFAULT sweep, so such
//     an axis is silently never run — the exact silence the discovery block was added for,
//     one array over.

fn grid_dir() -> std::path::PathBuf {
    repo_root().join("wat-scripts/perf/grid")
}

/// Axis stems on disk: a `<axis>.wat` WITH a `gen-<axis>.sh` twin. That pairing is what
/// `run-axis.sh` requires and is what distinguishes a perf axis from the `where-*`
/// expressivity corpus (static `.clj` twins, its own runner — different question).
fn discovered_axes() -> Vec<String> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(grid_dir()).expect("read grid dir").flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("wat") {
            continue;
        }
        let stem = p.file_stem().unwrap().to_string_lossy().to_string();
        if grid_dir().join(format!("gen-{stem}.sh")).is_file() {
            out.push(stem);
        }
    }
    out.sort();
    out
}

fn run_all_sh() -> String {
    std::fs::read_to_string(grid_dir().join("run-all.sh")).expect("read run-all.sh")
}

/// Keys of the `LADDER` associative array — lines shaped `  [axis]="..."`.
fn ladder_keys(src: &str) -> Vec<String> {
    src.lines()
        .filter_map(|l| {
            let t = l.trim_start();
            let rest = t.strip_prefix('[')?;
            let (key, after) = rest.split_once(']')?;
            after.starts_with('=').then(|| key.to_string())
        })
        .collect()
}

fn order_entries(src: &str) -> Vec<String> {
    let line = src
        .lines()
        .find(|l| l.trim_start().starts_with("ORDER=("))
        .expect("run-all.sh has no ORDER=( … ) line");
    let inner = line.split_once('(').unwrap().1.rsplit_once(')').unwrap().0;
    inner.split_whitespace().map(|s| s.to_string()).collect()
}

#[test]
fn every_grid_axis_on_disk_has_a_size_rung_and_is_swept_by_default() {
    let src = run_all_sh();
    let ladder = ladder_keys(&src);
    let order = order_entries(&src);
    let axes = discovered_axes();
    // NON-VACUITY: the axes are DISCOVERED from disk by the `.wat` + `gen-*.sh` pairing, so a
    // moved grid dir or a broken pairing rule would leave this gate sweeping an empty list and
    // reporting every ladder rung consistent. 11 axes are found today (driven 2026-09-01).
    assert!(!axes.is_empty(), "no perf axes discovered — the pairing rule or the dir moved");

    for axis in &axes {
        assert!(
            ladder.contains(axis),
            "grid axis `{axis}` has {axis}.wat + gen-{axis}.sh but NO LADDER rung in \
             run-all.sh — it would be swept by nobody. Choose sizes deliberately; a grid \
             run at different sizes is not comparable to the one before it."
        );
        assert!(
            order.contains(axis),
            "grid axis `{axis}` has a LADDER rung but is missing from ORDER in run-all.sh \
             — ORDER is the DEFAULT sweep, so `run-all.sh` with no arguments never runs it."
        );
    }
    for rung in &ladder {
        assert!(
            axes.contains(rung),
            "run-all.sh has a LADDER rung for `{rung}` but the grid has no {rung}.wat + \
             gen-{rung}.sh pair — a size ladder for an axis that does not exist."
        );
    }
}

#[test]
fn every_grid_generator_names_the_axis_its_filename_claims() {
    for axis in discovered_axes() {
        let gen = grid_dir().join(format!("gen-{axis}.sh"));
        let body = std::fs::read_to_string(&gen).expect("read generator");

        // Split the open-paren off as a CHAR before matching the head, so no string
        // literal here opens with `(` — `no_inlined_edn` bans EDN-esque literals in
        // tests, and is explicit that a literal which merely LOOKS like EDN is not a
        // rune candidate: restructure instead.
        let ns = body
            .lines()
            .find_map(|l| {
                let head = l.trim_start().strip_prefix('(')?.strip_prefix("ns ")?;
                Some(
                    head.split(|c: char| c.is_whitespace() || c == ')')
                        .next()
                        .unwrap_or("")
                        .to_string(),
                )
            })
            .unwrap_or_else(|| panic!("gen-{axis}.sh emits no namespace form"));
        assert_eq!(
            ns, axis,
            "gen-{axis}.sh declares namespace `{ns}` but run-axis.sh invokes `-m {axis}` — the \
             run dies with FileNotFoundException instead of a clear error."
        );

        let marker = ":axis \\\"";
        let ax = body
            .split_once(marker)
            .unwrap_or_else(|| panic!("gen-{axis}.sh emits no `:axis \\\"…\\\"` in its #grid/Result"))
            .1;
        let ax = &ax[..ax.find('\\').unwrap_or(ax.len())];
        assert_eq!(
            ax, axis,
            "gen-{axis}.sh emits :axis \"{ax}\" — run-axis.sh greps the emitted :axis to pair \
             the Clara result with the wat one, so a mismatch reads as a missing Clara run."
        );
    }
}
