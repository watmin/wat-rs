//! Gate — EVERY `.wat` under `wat-scripts/` must LOAD (parse + type-check) on the CURRENT runtime.
//!
//! `wat-scripts/` holds the kept refactor-tooling (`fixes/`), the showpiece demos, and library
//! helpers. None of it is frozen into the binary (unlike `wat/*.wat`, which fails the build on
//! drift) and none of it is loaded by any other test — so a substrate contract change (e.g. `first`
//! going Option->element under arc-047) rotted these scripts silently while the measured stdlib
//! usages got updated. A stale exemplar that no longer runs is a graveyard that reads like live
//! code (it trapped a prior session). This gate closes that blind spot: ALL wat must remain correct,
//! always — a `wat-scripts/` file that no longer type-checks goes RED here, so the rot cannot hide.
//!
//! `startup_from_source` + `FsLoader` (the disk loader cargo-wat uses) parses + type-checks the
//! whole world (all defns, including `:user::main`'s body, and any disk `load-file!` dependencies)
//! without running `main` — exactly the check that catches a dead idiom (`Option/expect`-over-`first`)
//! or a broken declaration form (the retired `:wat::core::Record::def`). It must use the SAME loader the
//! scripts run under (NOT `InMemoryLoader`), or it lies about scripts with relative `load-file!`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::FsLoader;

fn collect_wat(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let p = entry.expect("dir entry").path();
        if p.is_dir() {
            collect_wat(&p, out);
        } else if p.extension().is_some_and(|x| x == "wat") {
            out.push(p);
        }
    }
}

#[test]
fn every_wat_scripts_file_loads_on_the_current_runtime() {
    let mut entries = Vec::new();
    collect_wat(Path::new("wat-scripts"), &mut entries);
    entries.sort();

    // NON-VACUITY: a walk that comes back empty asserts nothing over nothing and reports PASS, and
    // every verdict downstream inherits that silence. The floor sits well under the
    // 445 .wat file(s) this walk finds today — driven 2026-09-01, and the count comes
    // from `tests/lint/every_walking_gate_declares_non_vacuity.rs`, never from prose — so it
    // catches a walk gone blind — a moved root, a renamed directory — without rotting as the
    // tree grows.
    assert!(
        entries.len() > 200,
        "the wat-scripts load walk found only {} .wat file(s) — it is not \
         reaching the tree it claims to guard, so its green means nothing",
        entries.len()
    );

    let mut failures = Vec::new();
    for path in &entries {
        let rel = path.to_str().expect("utf8 path");
        // FsLoader (the disk loader cargo-wat uses) — NOT InMemoryLoader — so a script's relative
        // `(:wat::load-file! "../lib/…")` resolves against its own dir exactly as it does when run.
        // The gate must measure the way the script actually runs, or it lies (false positives).
        let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        if let Err(e) = startup_from_source(&src, Some(rel), Arc::new(FsLoader)) {
            failures.push(format!("  {rel}\n      {e:?}"));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} wat-scripts/ files do not load on the current runtime (rotted):\n{}",
        failures.len(),
        entries.len(),
        failures.join("\n")
    );
}
