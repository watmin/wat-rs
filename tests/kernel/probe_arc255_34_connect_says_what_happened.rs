//! Stone 255.34 — ConnectOutcome says what happened.
//!
//! Pre-stone words were captured on the draw binary (before this stone's
//! `src/` change) and are quoted in the SCORE. Post-stone this probe prints
//! `Closed` for a dropped thread listener and for a dead process listener.

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
fn a_dropped_listener_is_closed_on_both_loci() {
    let (rc, stdout, stderr) = run("tests/kernel/probe_arc255_34_connect_says_what_happened.wat");
    assert_eq!(rc, 0, "stderr:\n{stderr}\nstdout:\n{stdout}");
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(
        lines,
        [
            "\"thread-dropped Closed connect: rendezvous send failed — listener was dropped (no listener)\"",
            "\"process-dead Closed connect abstract UDS: Connection refused (os error 111)\"",
        ],
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
