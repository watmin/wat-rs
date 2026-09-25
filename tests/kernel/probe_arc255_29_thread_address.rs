//! Stone 255.29 — a thread address is data.
//!
//! Pre-stone words were captured on the draw binary (`860add63b`, before this
//! stone's `src/` change) and are quoted in the SCORE. Post-stone:
//! - a Shared address in a pure record loads;
//! - `address-wire?` prints `false` then `true`;
//! - a thread address sent to a process child prints `Rejected` with the
//!   minting-process sentence, and the parent's echoed dial is `Connected`.

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
    let sentence = "a thread address is dialable only inside its minting process";
    let suffix = format!("{sentence}\" \"Connected\"]");
    let Some((head, tail)) = line.split_once(&suffix) else {
        panic!("stdout:\n{stdout}\nstderr:\n{stderr}");
    };
    assert_eq!(tail, "");
    assert_eq!(line.chars().next(), Some('['));
    let prefix = "\"Rejected: thread address minted by process ";
    let Some(rest) = head[1..].strip_prefix(prefix) else {
        panic!("stdout:\n{stdout}\nstderr:\n{stderr}");
    };
    let Some((minter, dialer_part)) = rest.split_once(" dialed from process ") else {
        panic!("stdout:\n{stdout}\nstderr:\n{stderr}");
    };
    let Some(dialer) = dialer_part.strip_suffix(" — ") else {
        panic!("stdout:\n{stdout}\nstderr:\n{stderr}");
    };
    let minter_pid: i32 = minter.parse().expect(minter);
    let dialer_pid: i32 = dialer.parse().expect(dialer);
    assert_ne!(minter_pid, dialer_pid);
}
