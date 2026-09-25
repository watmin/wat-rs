//! Stone 255.32 — the emitted ServiceEvent.Lost arm reaps and recurses.
//!
//! Pre-stone the same expansion contained assertion-failed!. The arm is
//! unreachable at runtime: poll' does not build Lost. This row reads the
//! macro's output.

use std::path::Path;
use std::process::{Command, Stdio};

fn run(rel: &str) -> (i32, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(bin)
        .arg(rel)
        .current_dir(manifest)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {rel}: {e}"));
    (
        out.status.code().unwrap_or(1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn the_emitted_lost_arm_reaps_and_does_not_raise() {
    let (rc, stdout, stderr) = run("tests/services/probe_arc255_32_lost_arm_expansion.wat");
    assert_eq!(rc, 0, "stderr:\n{stderr}\nstdout:\n{stdout}");
    assert_eq!(
        stdout.trim(),
        r#""ServiceEvent.Lost {:idx idx :cause _cause} (:p32.echo/serve self l (:wat.seq/remove-at selectables idx) next-id state)] [:wat.spawn/""#,
        "stderr:\n{stderr}"
    );
}
