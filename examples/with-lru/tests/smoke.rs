//! End-to-end smoke test for the with-lru-example binary.
//!
//! Spawns the just-built binary as a real subprocess, feeds it
//! empty stdin, asserts stdout contains `hit` and exit code 0.
//! This is the arc 013 proof's outermost layer: if this test
//! passes, the full external-wat-crate pipeline works from a
//! consumer's perspective — Cargo resolves wat + wat-lru, the
//! macro expands to `compose_and_run`, the runtime composes +
//! freezes + runs, stdio bridges through real OS streams, and
//! the LRU surface reaches user code through the dep registrar.
//!
//! Pattern mirrors `wat-rs/tests/wat_cli.rs`.

use std::process::{Command, Stdio};

#[test]
fn with_lru_example_prints_hit() {
    let bin = env!("CARGO_BIN_EXE_with-lru-example");
    let output = Command::new(bin)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn with-lru-example");

    assert!(
        output.status.success(),
        "expected exit 0; got {:?}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    assert_eq!(
        stdout.trim(),
        "hit",
        "expected stdout `hit`; got {:?}; stderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}
