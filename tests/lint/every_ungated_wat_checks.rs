//! Arc 255 — every UNGATED tracked `.wat` file must TYPE-CHECK, not merely parse.
//!
//! `every_tracked_wat_parses` (`tests/lint/every_tracked_wat_parses.rs`) asks a strictly weaker
//! question — does the reader accept it — over the WHOLE corpus. `every_wat_scripts_file_loads`
//! (`tests/lint/wat_scripts_fixes_load.rs`) type-checks, but only under `wat-scripts/`. Neither
//! wall reaches `examples/`, `crates/*/`, `benches/`, or `wat-migrate/` — and three files rotted
//! there in three unrelated ways (arc 255 crawl) with nothing to catch it. This is that wall.
//!
//! Scope, DERIVED (not hand-listed): every git-tracked `*.wat` whose path does not start with
//! `wat/`, `wat-scripts/`, `tests/`, `wat-tests/`, or `docs/`. Those five are already covered —
//! the first two by `every_wat_scripts_file_loads`, `tests/`/`wat-tests/` by their own `.rs`
//! drivers (several of which hold DELIBERATELY-bad fixtures this wall must not walk), `docs/` is
//! record, not code. Everything else is fair game, whatever currently lives there or gets added
//! later — an allowlist of exemptions is a permanent excuse (STOP-3 in the recorded brief).
//!
//! Uses the exact mechanism `wat --check` uses (`src/distribution/mod.rs`'s `check_only` arm):
//! `startup_from_source` (parse + type-check + freeze) under `FsLoader`, the disk loader, so a
//! file's relative `load-file!` resolves against its own directory the way a real check would.
//!
//! ⛔ NON-VACUITY IS MANDATORY. If the walked set is ever empty (e.g. every ungated corpus gets
//! deleted or re-gated), this must go RED and NAME the count — a wall that cannot fail is a claim,
//! not a check.

use std::process::Command;
use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::loader::FsLoader;

const GATED_PREFIXES: &[&str] = &["wat/", "wat-scripts/", "tests/", "wat-tests/", "docs/"];

#[test]
fn every_ungated_wat_file_checks() {
    let root = env!("CARGO_MANIFEST_DIR");
    let listing = Command::new("git")
        .args(["-C", root, "ls-files", "*.wat"])
        .output()
        .expect("git ls-files");
    assert!(listing.status.success(), "git ls-files must succeed");

    let paths: Vec<String> = String::from_utf8_lossy(&listing.stdout)
        .lines()
        .map(str::to_string)
        .filter(|rel| !GATED_PREFIXES.iter().any(|p| rel.starts_with(p)))
        .collect();

    assert!(
        !paths.is_empty(),
        "derived scope (tracked *.wat outside {:?}) is EMPTY — this wall is vacuous. Either the \
         ungated corpus was deleted (re-gate or delete this test deliberately) or the exclusion \
         prefixes widened to swallow it; got 0 files.",
        GATED_PREFIXES
    );

    let mut failures: Vec<String> = Vec::new();
    for rel in &paths {
        let full = std::path::Path::new(root).join(rel);
        let src = match std::fs::read_to_string(&full) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("  {rel}\n      could not read: {e}"));
                continue;
            }
        };
        // Same call `wat --check` makes: parse + type-check + freeze, FsLoader so relative
        // load-file! resolves against the file's own directory.
        if let Err(e) = startup_from_source(&src, Some(rel.as_str()), Arc::new(FsLoader)) {
            failures.push(format!("  {rel}\n      {e:?}"));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} ungated *.wat file(s) do not type-check (scope: tracked *.wat outside {:?}):\n{}",
        failures.len(),
        paths.len(),
        GATED_PREFIXES,
        failures.join("\n")
    );
}
