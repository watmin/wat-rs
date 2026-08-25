//! Arc 278 — every tracked `*.wat` file must READ.
//!
//! ⛔ Why this exists, and it is a real incident. Two tracked `.wat` files sat unreadable for
//! **months** — `docs/arc/2026/05/130-…/complected-2026-05-02/{substrate,test}.wat`, written
//! before arc 109 annihilated the angle bracket and un-lexable ever since. Nothing noticed,
//! because `wat/grep.wat` swallowed a parse failure into an empty fact base: every corpus-wide
//! census silently dropped them and reported success. That was F1 of
//! `278/NOTE-wat-grep-is-defective-three-findings.md`, and the fix made wat-grep LOUD.
//!
//! But loud-when-someone-runs-a-census is still a check nobody is obliged to run. This is the
//! wall: the class cannot regrow, because the floor holds it.
//!
//! **Parse only, not type-check.** `every_wat_scripts_file_loads` already type-checks everything
//! under `wat-scripts/` on the live runtime; this asks a strictly weaker question of a strictly
//! larger set — *does the reader accept it at all* — over every tracked `.wat` in the repo.
//!
//! A file that is deliberately not valid wat is named `.wat.bad` (the existing convention;
//! see `tests/collection/*.wat.bad`) and is therefore not tracked as `*.wat` at all. That is the
//! escape hatch, and it is a rename rather than an exemption list — nothing here to keep in sync.

use std::process::Command;

#[test]
fn every_tracked_wat_file_parses() {
    let root = env!("CARGO_MANIFEST_DIR");
    let listing = Command::new("git")
        .args(["-C", root, "ls-files", "*.wat"])
        .output()
        .expect("git ls-files");
    assert!(listing.status.success(), "git ls-files must succeed");
    let paths: Vec<String> = String::from_utf8_lossy(&listing.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    assert!(paths.len() > 1000, "expected the whole .wat corpus; got {}", paths.len());

    let mut unreadable: Vec<String> = Vec::new();
    for rel in &paths {
        let full = std::path::Path::new(root).join(rel);
        let src = match std::fs::read_to_string(&full) {
            Ok(s) => s,
            Err(e) => {
                unreadable.push(format!("{rel} — could not read: {e}"));
                continue;
            }
        };
        if let Err(e) = wat_reader::parser::parse_all_with_file(&src, rel) {
            unreadable.push(format!("{rel} — {e:?}"));
        }
    }

    assert!(
        unreadable.is_empty(),
        "{} tracked *.wat file(s) do not parse. A .wat the reader cannot read is invisible to \
         every corpus tool that walks the tree — which is how two of them survived months. \
         Fix the file, or rename it `.wat.bad` if being unreadable is the point (the arc-130 \
         complected/ calibration set is the worked example).\n  {}",
        unreadable.len(),
        unreadable.join("\n  ")
    );
}
