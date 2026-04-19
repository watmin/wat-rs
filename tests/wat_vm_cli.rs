//! End-to-end integration tests for the `wat-vm` binary.
//!
//! Each test spawns a real subprocess via [`std::process::Command`],
//! feeds real OS stdin, reads real OS stdout/stderr, and asserts on
//! both output and exit code. Uses `env!("CARGO_BIN_EXE_wat-vm")` so
//! Cargo points us at the just-built binary.

use std::io::Write;
use std::process::{Command, Stdio};

/// Helper: write `contents` to a uniquely-named temp file and return
/// its path. Caller is responsible for cleaning up.
fn write_temp(contents: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "wat-vm-test-{}-{}.wat",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    let mut f = std::fs::File::create(&path).expect("create temp");
    f.write_all(contents.as_bytes()).expect("write");
    path
}

/// Minimal `:user::main` that echoes stdin to stdout — the
/// hello-world of the wat-vm. Exercises:
/// - signature enforcement (3 args)
/// - kernel send
/// - kernel recv (one-line stdin semantic)
/// - crossbeam channel wiring
/// - stdio bridge threads
/// - clean shutdown
const ECHO_PROGRAM: &str = r#"
(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

(:wat::core::define (:user::main
                     (stdin  :crossbeam_channel::Receiver<String>)
                     (stdout :crossbeam_channel::Sender<String>)
                     (stderr :crossbeam_channel::Sender<String>)
                     -> :())
  (:wat::kernel::send stdout (:wat::kernel::recv stdin)))
"#;

#[test]
fn echo_program_reads_stdin_writes_stdout() {
    let path = write_temp(ECHO_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat-vm");

    // Pipe "watmin\n" to child stdin.
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"watmin\n")
        .unwrap();
    // Close stdin so child sees EOF after its one-line read.
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    let _ = std::fs::remove_file(&path);

    assert!(
        output.status.success(),
        "wat-vm exit {:?}, stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "watmin", "stdout mismatch: {:?}", stdout);
}

#[test]
fn missing_user_main_rejected() {
    // Valid setup but no :user::main defined — signature enforcement
    // should halt with exit 3.
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat-vm");
    let _ = std::fs::remove_file(&path);

    let code = output.status.code();
    assert_eq!(code, Some(3), "expected exit 3; got {:?}", code);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(":user::main"),
        "stderr must mention :user::main; got: {}",
        stderr
    );
}

#[test]
fn wrong_arity_user_main_rejected() {
    // :user::main declared with zero args — signature check rejects
    // (wat-vm requires 3 args).
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define (:user::main -> :()) ())
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat-vm");
    let _ = std::fs::remove_file(&path);

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("parameters"),
        "stderr should mention parameters; got: {}",
        stderr
    );
}

#[test]
fn wrong_arg_type_user_main_rejected() {
    // First arg typed :i64 instead of :crossbeam_channel::Receiver<String>.
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define (:user::main
                             (stdin  :i64)
                             (stdout :crossbeam_channel::Sender<String>)
                             (stderr :crossbeam_channel::Sender<String>)
                             -> :())
          ())
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat-vm");
    let _ = std::fs::remove_file(&path);

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("parameter #1") || stderr.contains("stdin"),
        "stderr should identify stdin; got: {}",
        stderr
    );
}

#[test]
fn usage_error_no_argv() {
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let output = Command::new(bin).stdin(Stdio::null()).output().expect("spawn");
    assert_eq!(output.status.code(), Some(64));
}

#[test]
fn missing_entry_file_is_ex_noinput() {
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let output = Command::new(bin)
        .arg("/nonexistent/wat-vm-test-missing.wat")
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    assert_eq!(output.status.code(), Some(66));
}

#[test]
fn startup_error_bubbles_up_as_exit_1() {
    // Missing :wat::config::set-dims! — startup halts.
    let program = r#"
        (:wat::algebra::Atom 42)
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    let _ = std::fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("wat-vm: startup:"));
}

#[test]
fn program_writes_multiple_times_to_stdout() {
    // :user::main calls send twice; stdout accumulates both writes.
    // The sequence is expressed as a let where the first send binds
    // the sacrificial `first` local (its Unit result is discarded);
    // the let body is the second send, whose Unit result is the
    // function's return value (matches the `-> :()` signature).
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define (:user::main
                             (stdin  :crossbeam_channel::Receiver<String>)
                             (stdout :crossbeam_channel::Sender<String>)
                             (stderr :crossbeam_channel::Sender<String>)
                             -> :())
          (:wat::core::let (((first :()) (:wat::kernel::send stdout "hello ")))
            (:wat::kernel::send stdout "world")))
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    let _ = std::fs::remove_file(&path);

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "hello world", "got: {:?}", stdout);
}
