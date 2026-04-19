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
  (:wat::core::match (:wat::kernel::recv stdin)
    ((Some line) (:wat::kernel::send stdout line))
    (:None ())))
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

/// Programs-are-atoms hello-world (structural side). Same observable
/// behavior as `ECHO_PROGRAM`, but the echo expression takes a detour
/// through the algebra's structural wrap/unwrap:
///
/// 1. `(:wat::core::quote ...)` captures the send/recv expression as a
///    `:wat::WatAST` without firing its side effects.
/// 2. `(:wat::algebra::Atom program)` wraps the WatAST as an Atom
///    holon — the program is now a typed box in the algebra.
/// 3. `(:wat::core::atom-value program-atom)` extracts the payload back
///    as a `:wat::WatAST`. Structural field read; exact; no cosine.
/// 4. `(:wat::core::eval-ast! reveal)` executes the program under
///    constrained eval.
///
/// This proves the STRUCTURAL side of programs-as-atoms: `(Atom x) →
/// (atom-value ...) → x` is lossless, exact, and carries arbitrary
/// wat programs as data.
///
/// The VECTOR side of the proof — measuring that `Bind(k, program-atom)`
/// obscures the program at the vector level and self-inverse recovers
/// it — needs the `:wat::core::presence` primitive and lives in its
/// own CLI test (added separately).
const PROGRAMS_ARE_ATOMS_PROGRAM: &str = r#"
(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

(:wat::core::define (:user::main
                     (stdin  :crossbeam_channel::Receiver<String>)
                     (stdout :crossbeam_channel::Sender<String>)
                     (stderr :crossbeam_channel::Sender<String>)
                     -> :())
  (:wat::core::let*
    (((program :wat::WatAST)
       (:wat::core::quote
         (:wat::core::match (:wat::kernel::recv stdin)
           ((Some line) (:wat::kernel::send stdout line))
           (:None ()))))
     ((program-atom :holon::HolonAST)
       (:wat::algebra::Atom program))
     ((reveal :wat::WatAST)
       (:wat::core::atom-value program-atom)))
    (:wat::core::eval-ast! reveal)))
"#;

#[test]
fn programs_are_atoms_hello_world() {
    let path = write_temp(PROGRAMS_ARE_ATOMS_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat-vm");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"watmin\n")
        .unwrap();
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
    assert_eq!(
        stdout, "watmin",
        "programs-are-atoms roundtrip failed — stdout: {:?}",
        stdout
    );
}

/// Programs-are-atoms hello-world (vector side, with presence proof).
///
/// Extends the structural hello-world with a VECTOR-level demonstration
/// that MAP's bind / unbind self-inverse is observable through presence
/// measurement:
///
/// 1. `(:wat::core::quote ...)` captures the send/recv expression as a
///    `:wat::WatAST`.
/// 2. `(:wat::algebra::Atom program)` wraps it as an Atom holon.
/// 3. `(:wat::algebra::Bind key-atom program-atom)` composes the Atom
///    with a key, producing a Bind tree whose encoded vector is
///    ROUGHLY ORTHOGONAL to the program-atom's vector. Below the 5σ
///    noise floor. `(:wat::core::presence program-atom bound)` returns
///    a small scalar — binarized via `>` against noise-floor yields
///    "None". The printed "None" IS the proof.
/// 4. `(:wat::algebra::Bind bound key-atom)` — MAP self-inverse at the
///    vector level: `bind(bind(k,p), k) ≈ p` on non-zero positions.
///    `(:wat::core::presence program-atom recovered)` returns a large
///    scalar — binarizes to "Some". The printed "Some" is the proof
///    the algebra recovered the signal.
/// 5. `(:wat::core::atom-value program-atom)` extracts the WatAST
///    payload structurally — the caller's reference has been in scope
///    all along. `(:wat::core::eval-ast! reveal)` fires the echo.
///
/// Observable stdout: `None\nSome\nwatmin`. The presence measurements
/// at lines 1 and 2 are the load-bearing proof; the echo at line 3 is
/// the eval confirming the program survived.
const PRESENCE_PROOF_PROGRAM: &str = r#"
(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

(:wat::core::define (:user::main
                     (stdin  :crossbeam_channel::Receiver<String>)
                     (stdout :crossbeam_channel::Sender<String>)
                     (stderr :crossbeam_channel::Sender<String>)
                     -> :())
  (:wat::core::let*
    (((program :wat::WatAST)
       (:wat::core::quote
         (:wat::core::match (:wat::kernel::recv stdin)
           ((Some line) (:wat::kernel::send stdout line))
           (:None ()))))
     ((program-atom :holon::HolonAST)
       (:wat::algebra::Atom program))
     ((key-atom :holon::HolonAST)
       (:wat::algebra::Atom "hello-world"))

     ;; Compose: program-atom bound under key-atom.
     ((bound :holon::HolonAST)
       (:wat::algebra::Bind key-atom program-atom))

     ;; Vector-level proof #1: program-atom's signal is GONE from bound.
     ((bound-score :f64)
       (:wat::core::presence program-atom bound))
     ((_ :())
       (:wat::kernel::send stdout
         (:wat::core::if
           (:wat::core::> bound-score (:wat::config::noise-floor))
           "Some\n"
           "None\n")))

     ;; Self-inverse: bind(bind(k, p), k) recovers p at the vector level.
     ((recovered :holon::HolonAST)
       (:wat::algebra::Bind bound key-atom))

     ;; Vector-level proof #2: program-atom's signal is BACK in recovered.
     ((recov-score :f64)
       (:wat::core::presence program-atom recovered))
     ((_ :())
       (:wat::kernel::send stdout
         (:wat::core::if
           (:wat::core::> recov-score (:wat::config::noise-floor))
           "Some\n"
           "None\n")))

     ;; Structural path: extract the WatAST from the in-scope program-atom
     ;; and run it. The presence measurements above proved the vector
     ;; dynamics; this line runs the actual program.
     ((reveal :wat::WatAST)
       (:wat::core::atom-value program-atom)))
    (:wat::core::eval-ast! reveal)))
"#;

#[test]
fn presence_proof_hello_world() {
    let path = write_temp(PRESENCE_PROOF_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat-vm");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"watmin\n")
        .unwrap();
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
    assert_eq!(
        stdout, "None\nSome\nwatmin",
        "presence proof mismatch — stdout: {:?}",
        stdout
    );
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

/// Stdlib macros are live at startup without an explicit load! —
/// `(:wat::std::Subtract x y)` expands to `(Blend x y 1 -1)` through
/// the baked `wat/std/Subtract.wat` file. Proves the stdlib loader
/// registers defmacros ahead of user code.
#[test]
fn stdlib_subtract_macro_expands_in_user_program() {
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main
                             (stdin  :crossbeam_channel::Receiver<String>)
                             (stdout :crossbeam_channel::Sender<String>)
                             (stderr :crossbeam_channel::Sender<String>)
                             -> :())
          ;; Call Subtract on two atoms. Expansion commits to Blend
          ;; with literal weights 1 / -1. The result is a holon;
          ;; measure presence of the first atom against the result and
          ;; report whether it crossed the noise floor.
          (:wat::core::let*
            (((a :holon::HolonAST) (:wat::algebra::Atom "alice"))
             ((b :holon::HolonAST) (:wat::algebra::Atom "bob"))
             ((diff :holon::HolonAST) (:wat::std::Subtract a b))
             ((score :f64) (:wat::core::presence a diff)))
            (:wat::kernel::send stdout
              (:wat::core::if
                (:wat::core::> score (:wat::config::noise-floor))
                "above"
                "below"))))
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    let _ = std::fs::remove_file(&path);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // The specific above/below answer depends on Blend's geometry;
    // what matters for this test is that the macro EXPANDED without
    // error and the program ran cleanly. Either outcome is a pass.
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout == "above" || stdout == "below",
        "expected `above` or `below`; got {:?}",
        stdout
    );
}
