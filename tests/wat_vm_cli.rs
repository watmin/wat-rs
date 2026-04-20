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
                     (stdin  :rust::std::io::Stdin)
                     (stdout :rust::std::io::Stdout)
                     (stderr :rust::std::io::Stderr)
                     -> :())
  (:wat::core::match (:wat::io::read-line stdin)
    ((Some line) (:wat::io::write stdout line))
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
/// it — needs the `:wat::algebra::cosine` primitive and lives in its
/// own CLI test (added separately).
const PROGRAMS_ARE_ATOMS_PROGRAM: &str = r#"
(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

(:wat::core::define (:user::main
                     (stdin  :rust::std::io::Stdin)
                     (stdout :rust::std::io::Stdout)
                     (stderr :rust::std::io::Stderr)
                     -> :())
  (:wat::core::let*
    (((program :wat::WatAST)
       (:wat::core::quote
         (:wat::core::match (:wat::io::read-line stdin)
           ((Some line) (:wat::io::write stdout line))
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
///    noise floor. `(:wat::algebra::cosine program-atom bound)` returns
///    a small scalar — binarized via `>` against noise-floor yields
///    "None". The printed "None" IS the proof.
/// 4. `(:wat::algebra::Bind bound key-atom)` — MAP self-inverse at the
///    vector level: `bind(bind(k,p), k) ≈ p` on non-zero positions.
///    `(:wat::algebra::cosine program-atom recovered)` returns a large
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
                     (stdin  :rust::std::io::Stdin)
                     (stdout :rust::std::io::Stdout)
                     (stderr :rust::std::io::Stderr)
                     -> :())
  (:wat::core::let*
    (((program :wat::WatAST)
       (:wat::core::quote
         (:wat::core::match (:wat::io::read-line stdin)
           ((Some line) (:wat::io::write stdout line))
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
       (:wat::algebra::cosine program-atom bound))
     ((_ :())
       (:wat::io::write stdout
         (:wat::core::if
           (:wat::core::> bound-score (:wat::config::noise-floor))
           "Some\n"
           "None\n")))

     ;; Self-inverse: bind(bind(k, p), k) recovers p at the vector level.
     ((recovered :holon::HolonAST)
       (:wat::algebra::Bind bound key-atom))

     ;; Vector-level proof #2: program-atom's signal is BACK in recovered.
     ((recov-score :f64)
       (:wat::algebra::cosine program-atom recovered))
     ((_ :())
       (:wat::io::write stdout
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
    // First arg typed :i64 instead of :rust::std::io::Stdin.
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define (:user::main
                             (stdin  :i64)
                             (stdout :rust::std::io::Stdout)
                             (stderr :rust::std::io::Stderr)
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
                             (stdin  :rust::std::io::Stdin)
                             (stdout :rust::std::io::Stdout)
                             (stderr :rust::std::io::Stderr)
                             -> :())
          (:wat::core::let (((first :()) (:wat::io::write stdout "hello ")))
            (:wat::io::write stdout "world")))
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
/// the baked `wat/std/Subtract.wat` file. Exercises BOTH branches of
/// the presence-vs-noise-floor discriminator so the test is honest:
///
/// - `presence(a, Subtract(a, b))` → `above` — Subtract(a, b) keeps
///   ~half of a's bits intact (the positions where a and b disagree);
///   cosine lands around 0.7, well above the 5σ floor (≈ 0.156 at
///   d=1024).
/// - `presence(c, Subtract(a, b))` → `below` — c is an independent
///   random atom uncorrelated with either a or b; cosine is near
///   zero, well below the floor.
///
/// Deterministic under the fixed seed. Output is exactly
/// "above\nbelow".
#[test]
fn stdlib_subtract_macro_expands_in_user_program() {
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:my::test::verdict
                             (score :f64)
                             -> :String)
          (:wat::core::if
            (:wat::core::> score (:wat::config::noise-floor))
            "above\n"
            "below\n"))

        (:wat::core::define (:user::main
                             (stdin  :rust::std::io::Stdin)
                             (stdout :rust::std::io::Stdout)
                             (stderr :rust::std::io::Stderr)
                             -> :())
          (:wat::core::let*
            (((a :holon::HolonAST) (:wat::algebra::Atom "alice"))
             ((b :holon::HolonAST) (:wat::algebra::Atom "bob"))
             ((c :holon::HolonAST) (:wat::algebra::Atom "charlie"))
             ((diff :holon::HolonAST) (:wat::std::Subtract a b))
             ((self-score :f64) (:wat::algebra::cosine a diff))
             ((other-score :f64) (:wat::algebra::cosine c diff))
             ((_ :()) (:wat::io::write stdout (:my::test::verdict self-score))))
            (:wat::io::write stdout (:my::test::verdict other-score))))
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
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout, "above\nbelow\n",
        "Subtract(a,b): presence(a,diff) should be above floor and presence(c,diff) below; got {:?}",
        stdout
    );
}

/// Circular expansion — hour 23 and hour 0 are adjacent on the
/// unit circle; hour 0 and hour 12 are antipodal. The stdlib macro
/// expands to a Blend over two basis atoms with cos/sin weights;
/// presence measurement against the reference encoding verifies
/// both the near-neighbor AND the far-neighbor paths fire as the
/// circular geometry predicts.
///
/// Expansion details (defined in wat/std/Circular.wat):
///   (Circular v p) →
///     (let* ((frac   (/ v p))
///            (two-pi (* 2.0 (pi)))
///            (theta  (* two-pi frac)))
///       (Blend (Atom :cos-basis) (Atom :sin-basis) (cos theta) (sin theta)))
///
/// Assertions exercise both branches of the noise-floor comparator.
#[test]
fn stdlib_circular_macro_near_and_far() {
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:my::test::verdict
                             (score :f64)
                             -> :String)
          (:wat::core::if
            (:wat::core::> score (:wat::config::noise-floor))
            "above\n"
            "below\n"))

        (:wat::core::define (:user::main
                             (stdin  :rust::std::io::Stdin)
                             (stdout :rust::std::io::Stdout)
                             (stderr :rust::std::io::Stderr)
                             -> :())
          (:wat::core::let*
            ;; Period is 24 (hours). h0 is midnight; h23 is an hour away;
            ;; h12 is noon, antipodal to h0 on the circle.
            (((h0  :holon::HolonAST) (:wat::std::Circular  0.0 24.0))
             ((h23 :holon::HolonAST) (:wat::std::Circular 23.0 24.0))
             ((h12 :holon::HolonAST) (:wat::std::Circular 12.0 24.0))
             ((near-score :f64) (:wat::algebra::cosine h0 h23))
             ((far-score  :f64) (:wat::algebra::cosine h0 h12))
             ((_ :()) (:wat::io::write stdout (:my::test::verdict near-score))))
            (:wat::io::write stdout (:my::test::verdict far-score))))
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
    let stdout = String::from_utf8(output.stdout).unwrap();
    // h0↔h23 are adjacent on the circle → cosine well above the 5σ floor.
    // h0↔h12 are antipodal → cosine is ~negative, below the floor.
    assert_eq!(
        stdout, "above\nbelow\n",
        "Circular(0,24)/Circular(23,24) should be near (above); Circular(0,24)/Circular(12,24) should be far (below); got {:?}",
        stdout
    );
}

/// Reject + Project — the Gram-Schmidt duo. Reject(x,y) carries
/// x's component ORTHOGONAL to y; Project(x,y) carries x's component
/// ALONG y. The geometry is the load-bearing invariant for the DDoS
/// sidecar's anomaly detection (Challenge 010, F1=1.000).
///
/// Test logic:
///   - presence(y, Reject(x, y))  → below floor (by construction)
///   - presence(y, Project(x, y)) → above floor (projection preserves
///     direction along y)
///
/// Exercises both macros AND both branches of the noise-floor
/// discriminator in one program. Assertion is exact.
#[test]
fn stdlib_reject_project_gram_schmidt_duo() {
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:my::test::verdict
                             (score :f64)
                             -> :String)
          (:wat::core::if
            (:wat::core::> score (:wat::config::noise-floor))
            "above\n"
            "below\n"))

        (:wat::core::define (:user::main
                             (stdin  :rust::std::io::Stdin)
                             (stdout :rust::std::io::Stdout)
                             (stderr :rust::std::io::Stderr)
                             -> :())
          (:wat::core::let*
            (((x :holon::HolonAST) (:wat::algebra::Atom "x"))
             ((y :holon::HolonAST) (:wat::algebra::Atom "y"))
             ((residual :holon::HolonAST) (:wat::std::Reject x y))
             ((shadow :holon::HolonAST) (:wat::std::Project x y))
             ((rej-score :f64) (:wat::algebra::cosine y residual))
             ((proj-score :f64) (:wat::algebra::cosine y shadow))
             ((_ :()) (:wat::io::write stdout (:my::test::verdict proj-score))))
            (:wat::io::write stdout (:my::test::verdict rej-score))))
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
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Project(x,y) preserves y's direction → above floor.
    // Reject(x,y) strips it  → below floor.
    assert_eq!(
        stdout, "above\nbelow\n",
        "Project(x,y) should align with y; Reject(x,y) should be orthogonal; got {:?}",
        stdout
    );
}

/// Sequential encoding is STRICT identity: two lists with the same
/// items in different order produce vectors that are orthogonal at
/// the noise-floor level. This is the load-bearing property of the
/// bind-chain expansion (058-009 reframe).
///
/// Test exercises both branches of the discriminator in one program:
///   - presence(Sequential[a,b,c], Sequential[a,b,c]) → above floor
///     (identical compound, cosine ≈ 1.0)
///   - presence(Sequential[a,b,c], Sequential[a,c,b]) → below floor
///     (different compound — the ordering is meaningful)
#[test]
fn stdlib_sequential_is_order_sensitive() {
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:my::test::verdict
                             (score :f64)
                             -> :String)
          (:wat::core::if
            (:wat::core::> score (:wat::config::noise-floor))
            "above\n"
            "below\n"))

        (:wat::core::define (:user::main
                             (stdin  :rust::std::io::Stdin)
                             (stdout :rust::std::io::Stdout)
                             (stderr :rust::std::io::Stderr)
                             -> :())
          (:wat::core::let*
            (((a :holon::HolonAST) (:wat::algebra::Atom "a"))
             ((b :holon::HolonAST) (:wat::algebra::Atom "b"))
             ((c :holon::HolonAST) (:wat::algebra::Atom "c"))
             ((abc :holon::HolonAST) (:wat::std::Sequential (:wat::core::list :holon::HolonAST a b c)))
             ((acb :holon::HolonAST) (:wat::std::Sequential (:wat::core::list :holon::HolonAST a c b)))
             ((same :f64) (:wat::algebra::cosine abc abc))
             ((reorder :f64) (:wat::algebra::cosine abc acb))
             ((_ :()) (:wat::io::write stdout (:my::test::verdict same))))
            (:wat::io::write stdout (:my::test::verdict reorder))))
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
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout, "above\nbelow\n",
        "Sequential(a,b,c) ≈ itself (above); Sequential(a,b,c) vs Sequential(a,c,b) differ (below); got {:?}",
        stdout
    );
}

/// Console stdlib — the Path-B hello-world. End-to-end proof that:
///   - stdlib-define registration works (Console + Console/loop +
///     Console/send are wat-source defines registered at startup)
///   - the tuple constructor produces a destructurable pair
///   - HandlePool's claim-or-panic cycle runs to completion
///   - spawn/join route a wat function across threads
///   - select + remove-at drop disconnected receivers cleanly
///
/// **Shutdown shape.** `console`'s binding must go out of scope
/// BEFORE the join, or the Arc it holds keeps the underlying
/// crossbeam sender alive and the driver's select sees no disconnect.
/// The nested let* below splits: the inner let* owns `console`;
/// when its body returns, the inner env drops, the sender's Arc
/// hits zero, the paired receiver in the driver sees :None,
/// Console/loop removes it, the rxs list empties, the driver
/// thread returns Unit, and the outer `(join driver)` unblocks.
/// Matches the lab's `drop(handles); driver.join()` pattern — the
/// cascade runs the shutdown.
///
/// Expected stdout exactly: "hello via Console".
#[test]
fn stdlib_console_hello_world() {
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main
                             (stdin  :rust::std::io::Stdin)
                             (stdout :rust::std::io::Stdout)
                             (stderr :rust::std::io::Stderr)
                             -> :())
          (:wat::core::let*
            (;; Build Console over BOTH stdio streams. One writer.
             ;; After this point, the good program ignores stdout /
             ;; stderr bindings — Console is the sole gateway.
             ((pool console-driver)
              (:wat::std::program::Console stdout stderr 1))
             ;; Phase 1 — do the Console work in an INNER scope so
             ;; the client handle drops before we reach the join.
             ((_ :())
              (:wat::core::let*
                (((console :rust::crossbeam_channel::Sender<(i64,String)>)
                  (:wat::kernel::HandlePool::pop pool))
                 ((_2 :()) (:wat::kernel::HandlePool::finish pool)))
                (:wat::std::program::Console/out console "hello via Console"))))
            ;; Phase 2 — inner scope done, console's Arc released,
            ;; Console/loop sees its rx disconnect and exits.
            (:wat::kernel::join console-driver)))
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat-vm");
    let _ = std::fs::remove_file(&path);
    assert!(
        output.status.success(),
        "wat-vm exit {:?}, stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "hello via Console", "got: {:?}", stdout);
}

/// Console with N>1 clients — the multi-writer gateway pattern.
/// Three worker functions each get their own Console client handle
/// and write a distinct message. Main joins all three workers inside
/// an inner scope; when the inner scope ends, every client handle's
/// Arc drops, Console/loop sees its rxs disconnect one-by-one via
/// select+remove-at, the list empties, the driver thread exits, and
/// the outer `(join console-driver)` unblocks.
///
/// Ordering: the three workers run in parallel; their writes arrive
/// at Console in whatever order the scheduler picks. The test sorts
/// the output lines and compares to the sorted expected set.
#[test]
fn stdlib_console_multi_writer() {
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define
          (:my::worker
            (console :rust::crossbeam_channel::Sender<(i64,String)>)
            (msg :String)
            -> :())
          (:wat::std::program::Console/out console msg))

        (:wat::core::define (:user::main
                             (stdin  :rust::std::io::Stdin)
                             (stdout :rust::std::io::Stdout)
                             (stderr :rust::std::io::Stderr)
                             -> :())
          (:wat::core::let*
            (((pool console-driver)
              (:wat::std::program::Console stdout stderr 3))
             ;; Inner scope owns the three handles AND the worker
             ;; program handles. When its body finishes, the inner
             ;; env drops, every handle Arc releases its last ref,
             ;; and Console/loop cascades shut.
             ((_ :())
              (:wat::core::let*
                (((h0 :rust::crossbeam_channel::Sender<(i64,String)>)
                  (:wat::kernel::HandlePool::pop pool))
                 ((h1 :rust::crossbeam_channel::Sender<(i64,String)>)
                  (:wat::kernel::HandlePool::pop pool))
                 ((h2 :rust::crossbeam_channel::Sender<(i64,String)>)
                  (:wat::kernel::HandlePool::pop pool))
                 ((_0 :()) (:wat::kernel::HandlePool::finish pool))
                 ((w0 :wat::kernel::ProgramHandle<()>)
                  (:wat::kernel::spawn :my::worker h0 "alpha\n"))
                 ((w1 :wat::kernel::ProgramHandle<()>)
                  (:wat::kernel::spawn :my::worker h1 "bravo\n"))
                 ((w2 :wat::kernel::ProgramHandle<()>)
                  (:wat::kernel::spawn :my::worker h2 "charlie\n"))
                 ((_1 :()) (:wat::kernel::join w0))
                 ((_2 :()) (:wat::kernel::join w1)))
                (:wat::kernel::join w2))))
            (:wat::kernel::join console-driver)))
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat-vm");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat-vm");
    let _ = std::fs::remove_file(&path);
    assert!(
        output.status.success(),
        "wat-vm exit {:?}, stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut lines: Vec<&str> = stdout.lines().collect();
    lines.sort();
    assert_eq!(
        lines,
        vec!["alpha", "bravo", "charlie"],
        "expected sorted {{alpha, bravo, charlie}}; got {:?}",
        stdout
    );
}

/// Trigram(a,b,c,d) = Bundle([Sequential(a,b,c), Sequential(b,c,d)]).
/// Presence of the first trigram's Sequential encoding against the
/// full Trigram should be above the noise floor — it's a participant
/// in the bundle. Presence of an UNRELATED atom should be below.
///
/// Exercises the full stdlib chain: Trigram → Ngram → map over window
/// → Sequential → foldl+map-with-index+Permute+Bind.
#[test]
fn stdlib_trigram_bundles_sequential_windows() {
    let program = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:my::test::verdict
                             (score :f64)
                             -> :String)
          (:wat::core::if
            (:wat::core::> score (:wat::config::noise-floor))
            "above\n"
            "below\n"))

        (:wat::core::define (:user::main
                             (stdin  :rust::std::io::Stdin)
                             (stdout :rust::std::io::Stdout)
                             (stderr :rust::std::io::Stderr)
                             -> :())
          (:wat::core::let*
            (((a :holon::HolonAST) (:wat::algebra::Atom "a"))
             ((b :holon::HolonAST) (:wat::algebra::Atom "b"))
             ((c :holon::HolonAST) (:wat::algebra::Atom "c"))
             ((d :holon::HolonAST) (:wat::algebra::Atom "d"))
             ((z :holon::HolonAST) (:wat::algebra::Atom "unrelated-z"))
             ((window-1 :holon::HolonAST)
              (:wat::std::Sequential (:wat::core::list :holon::HolonAST a b c)))
             ;; Trigram expands to a Bundle, which now returns
             ;; :Result<holon::HolonAST, wat::algebra::CapacityExceeded>.
             ;; Unwrap explicitly — the Err arm is unreachable here
             ;; (4 atoms at d=1024 is well under budget=32) but the
             ;; type system requires us to acknowledge it.
             ((full :holon::HolonAST)
              (:wat::core::match
                (:wat::std::Trigram (:wat::core::list :holon::HolonAST a b c d))
                ((Ok h) h)
                ((Err _) a)))
             ((participant :f64) (:wat::algebra::cosine window-1 full))
             ((outsider :f64) (:wat::algebra::cosine z full))
             ((_ :()) (:wat::io::write stdout (:my::test::verdict participant))))
            (:wat::io::write stdout (:my::test::verdict outsider))))
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
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout, "above\nbelow\n",
        "Sequential(a,b,c) should be present in Trigram(a,b,c,d) (above); unrelated atom not (below); got {:?}",
        stdout
    );
}
