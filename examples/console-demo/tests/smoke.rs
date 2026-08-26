//! Arc 255 stone — root-pull smoke test for console-demo. This is
//! the "run gate" half of the two-wall design: a `--check` pass
//! proves the wat type-checks, but cannot prove the demo doesn't
//! die at runtime on `:wat::kernel::eprintln` (wat's PANIC
//! channel). Spawning the built binary and asserting a clean exit
//! plus empty stderr is the only thing that catches that.
//!
//! Mirrors `examples/with-loader/tests/smoke.rs` in shape.

use std::process::Command;

#[test]
fn console_demo_prints_five_events_and_exits_clean() {
    let bin = env!("CARGO_BIN_EXE_console-demo");
    let output = Command::new(bin)
        .output()
        .expect("spawn console-demo binary");

    assert!(
        output.status.success(),
        "binary exited non-zero: status={:?} stdout={:?} stderr={:?}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = "\
#demo.Event/Buy [100.5 7]
#demo.Event/Sell [102.25 3 \"stop-loss\"]
#demo.Event/Buy [99.0 12]
#demo.Event/CircuitBreak [\"spike-volume\"]
#demo.Event/CircuitBreak [\"exchange-disconnected\"]
";
    assert_eq!(
        stdout.as_ref(),
        expected,
        "unexpected stdout (stderr: {:?})",
        String::from_utf8_lossy(&output.stderr),
    );

    // The assertion that a checker cannot make: nothing was ever
    // written to stderr. `eprintln` is wat's panic channel — any
    // byte here would mean the demo died mid-run.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty(),
        "expected empty stderr, got: {:?}",
        stderr,
    );
}
