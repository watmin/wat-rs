//! 2a4b — tracked `wat/**/*.wat` is EXACTLY the stdlib load list.
//!
//! `stdlib-source-path?` cannot ask the running binary's list (at #22 `wat/gen.wat`
//! is new). The tracked files under `wat/` are the include_str! home. If a file
//! is added under `wat/` without a `STDLIB_FILES` row, or a row names a path that
//! is not tracked, this gate goes red.

use std::process::Command;

fn tracked_wat_paths() -> Vec<String> {
    let out = Command::new("git")
        .args(["ls-files", "--", "wat/*.wat", "wat/**/*.wat"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git ls-files wat/");
    assert!(out.status.success(), "git ls-files failed: {out:?}");
    let mut v: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    v.sort();
    v
}

fn stdlib_paths() -> Vec<String> {
    const STDLIB_RS: &str = include_str!("../../src/load/stdlib.rs");
    let mut out = Vec::new();
    for (i, _) in STDLIB_RS.match_indices("path: \"") {
        let after = &STDLIB_RS[i + "path: \"".len()..];
        let Some(close) = after.find('"') else { continue };
        let p = after[..close].to_string();
        if p.starts_with("wat/") {
            out.push(p);
        }
    }
    out.sort();
    out.dedup();
    out
}

#[test]
fn tracked_wat_dir_is_exactly_stdlib_sources() {
    let tracked = tracked_wat_paths();
    let loaded = stdlib_paths();
    assert!(
        !tracked.is_empty(),
        "git ls-files wat/ returned nothing — this gate is measuring nothing"
    );
    assert!(
        !loaded.is_empty(),
        "STDLIB_FILES scan found no path: entries"
    );
    assert_eq!(
        tracked, loaded,
        "tracked wat/**/*.wat and STDLIB_FILES paths must be the same set.\n\
         only-tracked: {:?}\n\
         only-loaded: {:?}",
        tracked.iter().filter(|p| !loaded.contains(p)).collect::<Vec<_>>(),
        loaded.iter().filter(|p| !tracked.contains(p)).collect::<Vec<_>>(),
    );
}
