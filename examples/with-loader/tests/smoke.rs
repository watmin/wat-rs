//! Arc 017 slice 1 — end-to-end smoke test for `wat::main! {
//! loader: "wat" }`. Spawns the built binary, confirms:
//! (a) it exits cleanly (ScopedLoader resolved, `(load!)` fetched
//! helper.wat, `:user::main` invoked, printed);
//! (b) stdout is exactly `hello, wat-loaded\n`.

use std::process::Command;

#[test]
fn with_loader_example_loads_helper_and_prints_greeting() {
    let bin = env!("CARGO_BIN_EXE_with-loader-example");
    let output = Command::new(bin)
        .output()
        .expect("spawn with-loader-example binary");

    assert!(
        output.status.success(),
        "binary exited non-zero: status={:?} stdout={:?} stderr={:?}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.as_ref(),
        "hello, wat-loaded\n",
        "unexpected stdout (stderr: {:?})",
        String::from_utf8_lossy(&output.stderr),
    );
}
