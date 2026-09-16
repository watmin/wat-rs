//! 2a4d — the stdlib door reads a STEP's stdlib files as ONE world.
//!
//! Finding 23: batch 4a declared `FireOutcome` / `InsertOutcome` / `CompileOutcome` in
//! `wat/rete.wat` and USED them in `wat/fmt.wat`, `wat/grep.wat`, `wat/query.wat` and
//! `wat/rete/oracle/{explain,fire}.wat` — all changed in the SAME step. The door, asked
//! ONE FILE AT A TIME against the baked snapshot, answered every user file from the OLD
//! (or absent) enum; `match-arm` left the list-form arms UNRESOLVED, the merged stdlib
//! could not load, and the leftovers were KEY-FIRST'd BY HAND (the R21 breach).
//!
//! THE FIXTURE — two stdlib files in ONE set: the declarer re-declares a stdlib enum with
//! a NEW variant (a CHANGED enum), the user matches that new variant in list form. Handed
//! to `match-arm` as one set they must convert KEY-FIRST, with no UNRESOLVED, no
//! UNREGISTERABLE and no hand touch. The field names `tag` / `count` exist in no snapshot
//! and in no other file, so nothing but the sibling declaration can supply them.
//!
//! RED under the mutation that asks the door per file (`:wat::fix::enum-fields-in`
//! ignoring the world).
//!
//! The set is handed over as repo-relative `wat/…` paths with the codemod run FROM the
//! set's directory — `scripts/replay/convert.sh`'s exact shape, and what
//! `:wat::fix::stdlib-source-path?` keys on.
//!
//! The two fixture halves are co-located `.pre` files (never inlined — see
//! `tests/lint/no_inlined_wat_in_tests.rs`; `.pre` because they are migration INPUT
//! carrying the pre-migration spelling, exactly as `wat-scripts/fixes/replay/*/before.pre`
//! does, not corpus a loader should walk).

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const DECLARER: &str = "wat/probe2a4d-declarer.wat";
const USER: &str = "wat/probe2a4d-user.wat";

/// The set handed to a codemod on stdin, built from chars so the literal is not an
/// inlined EDN string (`tests/lint/no_inlined_edn.rs` — restructure, never a rune);
/// `every_recorded_migration_replays.rs`'s `default_path_vector` is the same shape.
fn path_vector(paths: &[&str]) -> String {
    let mut s = String::new();
    s.push('[');
    for (i, p) in paths.iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        s.push('"');
        s.push_str(p);
        s.push('"');
    }
    s.push(']');
    s.push('\n');
    s
}

#[test]
fn the_stdlib_door_reads_a_set_as_one_world() {
    let tmp = std::env::temp_dir().join(format!("wat-2a4d-one-world-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join("wat")).expect("mkdir wat/ in the set");
    fs::copy(
        manifest().join("tests/cli/stdlib_door_one_world__declarer.pre"),
        tmp.join(DECLARER),
    )
    .expect("copy the declarer half");
    fs::copy(
        manifest().join("tests/cli/stdlib_door_one_world__user.pre"),
        tmp.join(USER),
    )
    .expect("copy the user half");

    let codemod = manifest().join("wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat");
    let mut child = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(&codemod)
        .current_dir(&tmp)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat on the codemod");
    let set = path_vector(&[DECLARER, USER]);
    child
        .stdin
        .as_mut()
        .expect("codemod stdin")
        .write_all(set.as_bytes())
        .expect("hand the set over");
    drop(child.stdin.take());
    let out = child.wait_with_output().expect("wait for the codemod");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let converted = fs::read_to_string(tmp.join(USER)).unwrap_or_default();
    let _ = fs::remove_dir_all(&tmp);

    assert!(
        out.status.success(),
        "codemod rc={}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
        out.status
    );
    // rune:lint(loose-assert) — a targeted ABSENCE over the codemod's whole report; the
    // report carries per-file lines whose paths vary with the temp dir, so there is no
    // deterministic structured value to pin.
    assert!(
        !stdout.contains("UNRESOLVED"),
        "the set's own declaration must resolve for its sibling — no arm may be reported \
         UNRESOLVED\n--- stdout ---\n{stdout}"
    );
    // rune:lint(loose-assert) — same targeted absence over the same varying report.
    assert!(
        !stdout.contains("UNREGISTERABLE"),
        "the union of the set's stdlib files must register through the door\n--- stdout ---\n{stdout}"
    );
    // The converted half is deterministic — it carries no temp path — so it is pinned
    // WHOLE against a co-located golden, never a substring (`tests/lint/no_loose_string_assert.rs`).
    // The arm head keeps the `::` spelling on purpose: `match-arm` converts the arm's SHAPE,
    // and the separator flip is a later chain step (`variant-separator-to-dot`).
    assert_eq!(
        converted,
        include_str!("stdlib_door_one_world__user.post"),
        "the arm must be KEY-FIRST with the field names only the SIBLING file declares\n\
         --- stdout ---\n{stdout}"
    );
}
