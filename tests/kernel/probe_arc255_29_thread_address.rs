//! Stone 255.29 — a thread address is data.
//!
//! Pre-stone words were captured on the draw binary (`860add63b`, before this
//! stone's `src/` change) and are quoted in the SCORE. Post-stone:
//! - a Shared address in a pure record loads;
//! - `address-wire?` prints `false` then `true`;
//! - a thread address sent to a process child prints `Undialable` with the
//!   inert-wire sentence, and the parent's echoed dial is the same sentence.

use std::path::Path;
use std::process::{Command, Stdio};

use wat::freeze::startup_from_file;

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
fn a_shared_address_in_a_pure_record_is_accepted() {
    startup_from_file("tests/kernel/probe_arc255_29_shared_in_pure_record.wat")
        .expect("Shared address in a pure record must load");
}

#[test]
fn address_wire_stays_false_for_a_thread_address() {
    let (rc, stdout, stderr) = run("tests/kernel/probe_arc255_29_address_wire.wat");
    assert_eq!(rc, 0, "stderr:\n{stderr}\nstdout:\n{stdout}");
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines, ["false", "true"], "stdout:\n{stdout}\nstderr:\n{stderr}");
}

#[test]
fn a_thread_address_sent_to_a_process_child_is_rejected() {
    let (rc, stdout, stderr) = run("tests/kernel/probe_arc255_29_thread_address_to_process.wat");
    assert_eq!(rc, 0, "stderr:\n{stderr}\nstdout:\n{stdout}");
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 1, "stdout:\n{stdout}\nstderr:\n{stderr}");
    let line = lines[0];
    let sentence =
        "Undialable: a thread address is dialable only through the live value; one that crossed a wire is inert.";
    let mut expected = String::new();
    expected.push('[');
    expected.push('"');
    expected.push_str(sentence);
    expected.push('"');
    expected.push(' ');
    expected.push('"');
    expected.push_str(sentence);
    expected.push('"');
    expected.push(']');
    assert_eq!(line, expected, "stdout:\n{stdout}\nstderr:\n{stderr}");
}
