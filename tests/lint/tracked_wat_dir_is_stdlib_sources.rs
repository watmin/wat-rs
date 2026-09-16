//! 2a4b — tracked `wat/**/*.wat` is EXACTLY the stdlib load list.
//!
//! `stdlib-source-path?` cannot ask the running binary's list (at #22 `wat/gen.wat`
//! is new). The tracked files under `wat/` are the include_str! home. If a file
//! is added under `wat/` without a `STDLIB_FILES` row, or a row names a path that
//! is not tracked, this gate goes red.
//!
//! The load list is asked of the running substrate — `(:wat::stdlib::sources)`, via the
//! co-located fixture — never scanned out of `src/load/stdlib.rs`'s text: a text scan
//! counts a commented-out row as baked, so the gate would pass while the file is gone.

use std::process::Command;
use wat::freeze::call_beside_value;
use wat::runtime::Value;

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
    let val = call_beside_value(file!(), ":user::stdlib-source-paths")
        .expect("(:wat::stdlib::sources) should evaluate");
    let Value::Vec(items) = val else {
        panic!("expected a Vector of paths; got {val:?}");
    };
    let mut out: Vec<String> = items
        .iter()
        .map(|item| match item {
            Value::String(s) => s.to_string(),
            other => panic!("expected a path String; got {other:?}"),
        })
        .collect();
    out.sort();
    out
}

#[test]
fn tracked_wat_dir_is_exactly_stdlib_sources() {
    let tracked = tracked_wat_paths();
    let loaded = stdlib_paths();
    // NON-VACUITY: both sides of the set-equality below must independently prove they found
    // something, or an empty tracked set and an empty loaded set would compare equal and pass.
    assert!(
        !tracked.is_empty(),
        "git ls-files wat/ returned nothing — this gate is measuring nothing"
    );
    assert!(!loaded.is_empty(), "(:wat::stdlib::sources) returned no files");
    assert_eq!(
        tracked, loaded,
        "tracked wat/**/*.wat and the baked stdlib's paths must be the same set.\n\
         only-tracked: {:?}\n\
         only-loaded: {:?}",
        tracked.iter().filter(|p| !loaded.contains(p)).collect::<Vec<_>>(),
        loaded.iter().filter(|p| !tracked.contains(p)).collect::<Vec<_>>(),
    );
}
