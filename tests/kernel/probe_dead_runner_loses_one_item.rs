//! a-dead-runner-loses-one-item-not-the-run — control by mutation.
//!
//! Runtime: 4 items, 2 thread runners; worker 0 panics on its first item.
//! Re-dispatch must still return `[0 2 4 6]`.
//! Source pin: Closed/Lost in collect-loop call `collect-requeue`. Delete that
//! and this file goes red — the predecessor control failed that bar.

use std::path::Path;

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

fn bracket_code() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("wat/bracket.wat");
    let src = std::fs::read_to_string(p).expect("wat/bracket.wat must be readable");
    src.lines()
        .map(|l| match l.find(";;") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn run_vec(path: &str) -> Vec<i64> {
    let world = startup_from_file(path).expect("startup should succeed");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {path:?}"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute eval")
    {
        Value::Vec(v) => v
            .iter()
            .map(|tv| match tv {
                Value::i64(n) => *n,
                other => panic!("non-i64 element: {other:?}"),
            })
            .collect(),
        other => panic!("expected Vector; got {other:?}"),
    }
}

/// ⭐ Closed/Lost re-dispatch is actually in the collect-loop arms.
/// Delete `collect-requeue` from those arms and this reddens.
#[test]
fn closed_and_lost_requeue_the_held_item() {
    let code = bracket_code();
    let closed = code
        .split(":wat::spawn::ServiceEvent::Closed")
        .nth(1)
        .expect("collect-loop has no Closed arm");
    let closed_body = &closed[..closed.len().min(800)];
    assert!(
        // rune:lint(loose-assert) — presence of the re-dispatch helper in the Closed
        // arm body. The claim is "this call is here", not a value equality.
        closed_body.contains("collect-requeue"),
        "Closed arm no longer re-dispatches via collect-requeue — the mutation this \
         control exists to catch"
    );
    let lost = code
        .split(":wat::spawn::ServiceEvent::Lost")
        .nth(1)
        .expect("collect-loop has no Lost arm");
    let lost_body = &lost[..lost.len().min(800)];
    assert!(
        // rune:lint(loose-assert) — presence of the re-dispatch helper in the Lost
        // arm body. The claim is "this call is here", not a value equality.
        lost_body.contains("collect-requeue"),
        "Lost arm no longer re-dispatches via collect-requeue — the mutation this \
         control exists to catch"
    );
}

/// All 4 results returned after worker 0 dies holding item 0.
#[test]
fn killed_runner_item_is_redelivered() {
    let got = run_vec("tests/kernel/probe_dead_runner_loses_one_item.wat");
    assert_eq!(got, vec![0, 2, 4, 6]);
}

/// Non-vacuity: the kill actually fired (worker 0 eprinted before dying).
#[test]
fn dead_runner_fault_fired() {
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = std::process::Command::new(bin)
        .arg("tests/kernel/probe_dead_runner_loses_one_item.wat")
        .output()
        .expect("spawn wat");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "map should survive the dead runner; stdout={stdout} stderr={stderr}"
    );
    assert_eq!(stdout.trim().trim_matches('"'), "0,2,4,6");
    assert!(
        // rune:lint(loose-assert) — the kill sentinel is a presence check on
        // captured stderr of a subprocess, not a value under test.
        stderr.contains("dead-runner-fired"),
        "worker 0 never died; stderr={stderr}"
    );
}
