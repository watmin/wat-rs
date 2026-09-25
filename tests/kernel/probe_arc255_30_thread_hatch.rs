//! Stone 255.30 — the thread hatch is closed. Every comm carries data.
//!
//! Pre-stone words (draw binary, before this stone's checker change): all four
//! rows rc=0. The struct row printed nothing. The pure row printed `7`. The
//! defservice printed `"echo:hi"`. The kwargs pool printed
//! `["echo:a" "echo:b" "echo:c"]`.

use std::path::Path;
use std::process::{Command, Stdio};

use wat::check::error::CheckErrorKind;
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

const WALL: &str = "a comm carries only pure data — type :p30::S is not \
    pure (§7 purity wall). A resource belongs in :ephemeral state, never on a channel. \
    Redesign I/O as records, scalars, or pure enums \
    (no Sender, Receiver, or handle fields).";

#[test]
fn a_struct_on_a_thread_peer_is_refused() {
    let result = startup_from_file("tests/kernel/probe_arc255_30_struct_on_thread_peer.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::fn" && reason == WALL
    );
}

#[test]
fn a_pure_payload_on_a_thread_peer_round_trips() {
    let (rc, stdout, stderr) = run("tests/kernel/probe_arc255_30_pure_on_thread_peer.wat");
    assert_eq!(rc, 0, "stderr:\n{stderr}\nstdout:\n{stdout}");
    assert_eq!(stdout.trim(), "7");
}

#[test]
fn a_thread_locus_defservice_runs() {
    let (rc, stdout, stderr) = run("tests/kernel/probe_arc255_30_thread_defservice.wat");
    assert_eq!(rc, 0, "stderr:\n{stderr}\nstdout:\n{stdout}");
    assert_eq!(stdout.trim(), "\"echo:hi\"");
}

#[test]
fn the_bracket_thread_pool_with_kwargs_runs() {
    let (rc, stdout, stderr) = run("tests/kernel/probe_arc255_30_bracket_kwargs_pool.wat");
    assert_eq!(rc, 0, "stderr:\n{stderr}\nstdout:\n{stdout}");
    let line = stdout.trim();
    assert_eq!(line.chars().next(), Some('['));
    assert_eq!(line.chars().next_back(), Some(']'));
    assert_eq!(&line[1..line.len() - 1], "\"echo:a\" \"echo:b\" \"echo:c\"");
}
