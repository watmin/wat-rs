//! Integration coverage for `wat test <path>` (arc 007 slice 4).
//!
//! Spawns the built `wat` binary with `test` subcommand and asserts
//! on exit code + stdout. Belt-and-suspenders for the wat-tests/
//! convention — a failing deftest would surface here as a non-zero
//! exit from the CLI.

use std::process::Command;

/// Path to the cargo-built `wat` binary. env! resolves at compile time.
const WAT_BIN: &str = env!("CARGO_BIN_EXE_wat");

fn wat_test(path: &str) -> (std::process::ExitStatus, String, String) {
    let out = Command::new(WAT_BIN)
        .arg("test")
        .arg(path)
        .output()
        .expect("spawn wat");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (out.status, stdout, stderr)
}

// ─── happy path — wat-tests/ directory ──────────────────────────────────

#[test]
fn wat_test_on_wat_tests_dir_passes() {
    let crate_root = env!("CARGO_MANIFEST_DIR");
    let wat_tests_dir = format!("{}/wat-tests", crate_root);
    let (status, stdout, stderr) = wat_test(&wat_tests_dir);
    assert!(
        status.success(),
        "wat test wat-tests/ should exit 0.\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("test result: ok"),
        "expected cargo-style ok summary; got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("running "),
        "expected \"running N tests\" banner; got:\n{}",
        stdout
    );
}

// ─── happy path — single-file invocation ────────────────────────────────

#[test]
fn wat_test_on_single_file_passes() {
    let crate_root = env!("CARGO_MANIFEST_DIR");
    let harness = format!("{}/wat-tests/std/test.wat", crate_root);
    let (status, stdout, _) = wat_test(&harness);
    assert!(status.success(), "single-file invocation should exit 0:\n{}", stdout);
    assert!(stdout.contains("test result: ok"));
}

// ─── failing test surfaces through exit code ────────────────────────────

#[test]
fn wat_test_failing_deftest_exits_nonzero() {
    // Write a one-off .wat file to a tempdir with a failing deftest
    // and assert the CLI reports FAILED and exits non-zero.
    let tmp = std::env::temp_dir().join(format!(
        "wat-test-fail-{}-{}.wat",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let src = r##"(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::test::deftest :failing::test-should-fail :error 1024
  ()
  (:wat::test::assert-eq 1 2))
"##;
    std::fs::write(&tmp, src).expect("write tempfile");

    let (status, stdout, stderr) = wat_test(&tmp.to_string_lossy());
    let _ = std::fs::remove_file(&tmp);

    assert!(
        !status.success(),
        "failing test should exit non-zero:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("FAILED"),
        "expected FAILED marker in output:\n{}",
        stdout
    );
    assert!(
        stdout.contains("assert-eq failed"),
        "expected failure message from assert-eq:\n{}",
        stdout
    );
}

// ─── empty path / no .wat files ─────────────────────────────────────────

#[test]
fn wat_test_empty_dir_exits_no_tests() {
    let tmp = std::env::temp_dir().join(format!(
        "wat-test-empty-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&tmp).expect("create tempdir");

    let (status, _stdout, stderr) = wat_test(&tmp.to_string_lossy());
    let _ = std::fs::remove_dir(&tmp);

    assert!(!status.success(), "empty dir should exit non-zero");
    assert!(
        stderr.contains("no .wat files"),
        "expected 'no .wat files' diagnostic; got stderr:\n{}",
        stderr
    );
}

// ─── usage error — missing path ─────────────────────────────────────────

#[test]
fn wat_test_missing_path_reports_usage() {
    let out = Command::new(WAT_BIN)
        .arg("test")
        .output()
        .expect("spawn wat");
    assert!(!out.status.success(), "no path → usage error");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("usage:"), "expected usage message; got {}", stderr);
}
