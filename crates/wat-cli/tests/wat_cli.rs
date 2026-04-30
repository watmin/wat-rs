//! End-to-end integration tests for the `wat` binary.
//!
//! Each test spawns a real subprocess via [`std::process::Command`],
//! feeds real OS stdin, reads real OS stdout/stderr, and asserts on
//! both output and exit code. Uses `env!("CARGO_BIN_EXE_wat")` so
//! Cargo points us at the just-built binary.

use std::io::Write;
use std::process::{Command, Stdio};

/// Helper: write `contents` to a uniquely-named temp file and return
/// its path. Caller is responsible for cleaning up.
fn write_temp(contents: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "wat-test-{}-{}.wat",
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
/// hello-world of the wat. Exercises:
/// - signature enforcement (3 args)
/// - kernel send
/// - kernel recv (one-line stdin semantic)
/// - crossbeam channel wiring
/// - stdio bridge threads
/// - clean shutdown
const ECHO_PROGRAM: &str = r#"

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
    ((Some line) (:wat::io::IOWriter/print stdout line))
    (:None ())))
"#;

#[test]
fn echo_program_reads_stdin_writes_stdout() {
    let path = write_temp(ECHO_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat");

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
        "wat exit {:?}, stderr: {}",
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
/// 2. `(:wat::holon::Atom program)` wraps the WatAST as an Atom
///    holon — the program is now a typed box in the algebra.
/// 3. `(:wat::core::atom-value program-atom)` extracts the payload back
///    as a `:wat::WatAST`. Structural field read; exact; no cosine.
/// 4. `(:wat::eval-ast! reveal)` executes the program under
///    constrained eval.
///
/// This proves the STRUCTURAL side of programs-as-atoms: `(Atom x) →
/// (atom-value ...) → x` is lossless, exact, and carries arbitrary
/// wat programs as data.
///
/// The VECTOR side of the proof — measuring that `Bind(k, program-atom)`
/// obscures the program at the vector level and self-inverse recovers
/// it — needs the `:wat::holon::cosine` primitive and lives in its
/// own CLI test (added separately).
const PROGRAMS_ARE_ATOMS_PROGRAM: &str = r#"

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::core::let*
    (((program :wat::WatAST)
       (:wat::core::quote
         (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
           ((Some line) (:wat::io::IOWriter/print stdout line))
           (:None ()))))
     ((program-atom :wat::holon::HolonAST)
       (:wat::holon::Atom program))
     ;; arc 057 Story-2 recovery: program-atom is now a structural
     ;; HolonAST (the form lowered onto the algebra grid). to-watast
     ;; lifts it back to a runnable WatAST; eval-ast! fires it.
     ((reveal :wat::WatAST)
       (:wat::holon::to-watast program-atom)))
    ;; eval-ast! returns :Result<wat::holon::HolonAST, EvalError> per
    ;; the 2026-04-20 INSCRIPTION. Match both arms to preserve main's
    ;; declared return type of :(). Err arm is unreachable here
    ;; (the quoted program is well-formed and non-mutating).
    (:wat::core::match (:wat::eval-ast! reveal) -> :()
      ((Ok _) ())
      ((Err _) ()))))
"#;

#[test]
fn programs_are_atoms_hello_world() {
    let path = write_temp(PROGRAMS_ARE_ATOMS_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat");

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
        "wat exit {:?}, stderr: {}",
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
/// 2. `(:wat::holon::Atom program)` wraps it as an Atom holon.
/// 3. `(:wat::holon::Bind key-atom program-atom)` composes the Atom
///    with a key, producing a Bind tree whose encoded vector is
///    ROUGHLY ORTHOGONAL to the program-atom's vector. Below the 5σ
///    noise floor. `(:wat::holon::cosine program-atom bound)` returns
///    a small scalar — binarized via `>` against noise-floor yields
///    "None". The printed "None" IS the proof.
/// 4. `(:wat::holon::Bind bound key-atom)` — MAP self-inverse at the
///    vector level: `bind(bind(k,p), k) ≈ p` on non-zero positions.
///    `(:wat::holon::cosine program-atom recovered)` returns a large
///    scalar — binarizes to "Some". The printed "Some" is the proof
///    the algebra recovered the signal.
/// 5. `(:wat::core::atom-value program-atom)` extracts the WatAST
///    payload structurally — the caller's reference has been in scope
///    all along. `(:wat::eval-ast! reveal)` fires the echo.
///
/// Observable stdout: `None\nSome\nwatmin`. The presence measurements
/// at lines 1 and 2 are the load-bearing proof; the echo at line 3 is
/// the eval confirming the program survived.
const PRESENCE_PROOF_PROGRAM: &str = r#"

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::core::let*
    (((program :wat::WatAST)
       (:wat::core::quote
         (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
           ((Some line) (:wat::io::IOWriter/print stdout line))
           (:None ()))))
     ((program-atom :wat::holon::HolonAST)
       (:wat::holon::Atom program))
     ((key-atom :wat::holon::HolonAST)
       (:wat::holon::Atom "hello-world"))

     ;; Compose: program-atom bound under key-atom.
     ((bound :wat::holon::HolonAST)
       (:wat::holon::Bind key-atom program-atom))

     ;; Substrate proof #1: program-atom's signal is GONE from bound.
     ;; Arc 037 slice 3: presence? does the honest per-d threshold
     ;; comparison internally — the router picks d per operand,
     ;; presence-floor is computed as presence-sigma / sqrt(d) at the
     ;; picked d. Users no longer hand-roll `cosine vs noise-floor`.
     ((_ :())
       (:wat::io::IOWriter/print stdout
         (:wat::core::if
           (:wat::holon::presence? program-atom bound)
           -> :String
           "Some\n"
           "None\n")))

     ;; Self-inverse: bind(bind(k, p), k) recovers p at the vector level.
     ((recovered :wat::holon::HolonAST)
       (:wat::holon::Bind bound key-atom))

     ;; Substrate proof #2: program-atom's signal is BACK in recovered.
     ((_ :())
       (:wat::io::IOWriter/print stdout
         (:wat::core::if
           (:wat::holon::presence? program-atom recovered)
           -> :String
           "Some\n"
           "None\n")))

     ;; arc 057 Story-2 recovery: program-atom is the structurally
     ;; lowered HolonAST (Bundle of Symbol/lit leaves). to-watast lifts
     ;; it back to a runnable WatAST. The presence measurements above
     ;; proved the vector dynamics; this line runs the actual program.
     ((reveal :wat::WatAST)
       (:wat::holon::to-watast program-atom)))
    ;; eval-ast! now returns :Result<wat::holon::HolonAST, EvalError> per
    ;; the 2026-04-20 INSCRIPTION. Match both arms to preserve main's
    ;; declared return type of :(). Err arm is unreachable here —
    ;; the quoted echo program is well-formed and non-mutating.
    (:wat::core::match (:wat::eval-ast! reveal) -> :()
      ((Ok _) ())
      ((Err _) ()))))
"#;

#[test]
fn presence_proof_hello_world() {
    let path = write_temp(PRESENCE_PROOF_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat");

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
        "wat exit {:?}, stderr: {}",
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
    // halts the child with EXIT_MAIN_SIGNATURE (4).  Arc 104 cli
    // forks the entry and propagates the child's exit code; the
    // signature check moved from cli → child branch, so the code
    // is now 4 (was 3 pre-arc-104, when cli ran user code in-thread).
    let program = r#"
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat");
    let _ = std::fs::remove_file(&path);

    let code = output.status.code();
    assert_eq!(code, Some(4), "expected exit 4; got {:?}", code);
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
    // (wat requires 3 args). EXIT_MAIN_SIGNATURE=4 from the child.
    let program = r#"
        (:wat::core::define (:user::main -> :()) ())
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat");
    let _ = std::fs::remove_file(&path);

    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("parameters"),
        "stderr should mention parameters; got: {}",
        stderr
    );
}

#[test]
fn wrong_arg_type_user_main_rejected() {
    // First arg typed :i64 instead of :wat::io::IOReader.
    // EXIT_MAIN_SIGNATURE=4 from the child.
    let program = r#"
        (:wat::core::define (:user::main
                             (stdin  :i64)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :())
          ())
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat");
    let _ = std::fs::remove_file(&path);

    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("parameter #1") || stderr.contains("stdin"),
        "stderr should identify stdin; got: {}",
        stderr
    );
}

#[test]
fn usage_error_no_argv() {
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin).stdin(Stdio::null()).output().expect("spawn");
    assert_eq!(output.status.code(), Some(64));
}

#[test]
fn missing_entry_file_is_ex_noinput() {
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg("/nonexistent/wat-test-missing.wat")
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    assert_eq!(output.status.code(), Some(66));
}

#[test]
fn startup_error_bubbles_up_as_exit_3() {
    // Arc 037 retired required-ness for dims/capacity-mode. A remaining
    // startup failure surface: malformed config setter (bad type) still
    // halts startup. Arc 104 cli forks the entry; startup happens IN
    // THE CHILD now, so the failure exits the child with
    // EXIT_STARTUP_ERROR=3 (was 1 pre-arc-104, when cli ran startup
    // in-thread). set-capacity-mode! takes a keyword; passing a string
    // triggers ConfigError::BadType.
    let program = r#"
        (:wat::config::set-capacity-mode! "oops")
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    let _ = std::fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Child's "startup: ..." prefix (no "wat:" — that was a pre-arc-104
    // cli prefix; the proxy forwards the child's stderr verbatim).
    assert!(
        stderr.contains("startup:"),
        "stderr should contain 'startup:'; got: {}",
        stderr
    );
}

#[test]
fn program_writes_multiple_times_to_stdout() {
    // :user::main calls send twice; stdout accumulates both writes.
    // The sequence is expressed as a let where the first send binds
    // the sacrificial `first` local (its Unit result is discarded);
    // the let body is the second send, whose Unit result is the
    // function's return value (matches the `-> :()` signature).
    let program = r#"
        (:wat::core::define (:user::main
                             (stdin  :wat::io::IOReader)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :())
          (:wat::core::let (((first :()) (:wat::io::IOWriter/print stdout "hello ")))
            (:wat::io::IOWriter/print stdout "world")))
    "#;
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat");
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

#[test]
fn sigterm_to_cli_cascades_via_polling_contract() {
    // Arc 106 — the wat-native polling contract through fork. The
    // contract this test exercises:
    //
    //   1. cli installs wat signal handlers at startup; child
    //      installs the same handlers post-fork (substrate, not
    //      SIG_DFL — arc 106 replaced the SIG_DFL reset block in
    //      `child_branch_from_source` with `install_substrate_signal_handlers`).
    //   2. The child becomes its own process group leader via
    //      `setpgid(0, 0)` post-fork. The cli's CHILD_PGID atomic
    //      tracks this group.
    //   3. SIGTERM arrives at cli's handler → flips KERNEL_STOPPED
    //      in cli's memory + `killpg(CHILD_PGID, SIGTERM)` to
    //      broadcast.
    //   4. Kernel delivers SIGTERM to every group member; each
    //      child's wat handler flips its own KERNEL_STOPPED.
    //   5. Wat program polls `(:wat::kernel::stopped?)` → observes
    //      true → returns cleanly. :user::main returns ().
    //   6. Child _exits 0. Cli's waitpid returns WIFEXITED with
    //      code 0. Cli exits 0.
    //
    // Lock-step via stdout marker. The program prints "READY" once
    // it's about to enter the polling loop — by then the cli has
    // forked it, set CHILD_PGID, the child has setpgid'd into its
    // own group, installed handlers, loaded the program. Test reads
    // stdout until READY, THEN sends SIGTERM. No sleep; the wire IS
    // the synchronization.
    //
    // The test runs in a forked subprocess via `wat::fork::run_in_fork`
    // for hermetic isolation — fresh signal-handler state, no SIGCHLD
    // residue from earlier tests in this binary. Same isolation
    // pattern `tests/wat_harness_deps.rs` uses against OnceLock
    // contention.
    wat::fork::run_in_fork(|| {
        let program = r#"
            (:wat::core::define (:demo::loop
                                 (stdout :wat::io::IOWriter)
                                 -> :())
              (:wat::core::if (:wat::kernel::stopped?) -> :()
                ()                                       ; observed stop → return clean
                (:demo::loop stdout)))                   ; tight poll loop

            (:wat::core::define (:user::main
                                 (stdin  :wat::io::IOReader)
                                 (stdout :wat::io::IOWriter)
                                 (stderr :wat::io::IOWriter)
                                 -> :())
              (:wat::core::let*
                (((_ :()) (:wat::io::IOWriter/println stdout "READY")))
                (:demo::loop stdout)))
        "#;
        let path = write_temp(program);
        let bin = env!("CARGO_BIN_EXE_wat");
        let mut child = Command::new(bin)
            .arg(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn wat");

        // Lock-step: read stdout until we see READY. By the time the
        // child wat process has println'd READY, every cascade
        // prerequisite is settled — cli has fork()ed + set CHILD_PGID,
        // child has setpgid'd, child has installed wat handlers, child
        // has loaded program, child is in the polling loop. SIGTERM
        // is now safe to deliver; no race window.
        use std::io::{BufRead, BufReader};
        let stdout = child.stdout.take().expect("child stdout");
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).expect("read READY");
        assert_eq!(line.trim(), "READY", "expected READY marker; got {:?}", line);

        // Send SIGTERM to wat-cli. The handler flips KERNEL_STOPPED
        // in cli + killpg(CHILD_PGID, SIGTERM) cascades to every
        // process in the group. Child's wat handler flips its own
        // KERNEL_STOPPED; child polls; child exits clean.
        let cli_pid = child.id() as libc::pid_t;
        unsafe {
            libc::kill(cli_pid, libc::SIGTERM);
        }

        let status = child.wait().expect("wait wat-cli");
        let code = status.code();
        let _ = std::fs::remove_file(&path);

        // Polling contract: child exits 0 (clean shutdown via
        // observed stop flag). NOT 143 (which would mean the child
        // was killed by SIGTERM's default action — pre-arc-106
        // contract). NOT None (which would mean the cli process
        // itself was killed by signal before forwarding — impossible
        // post-arc-106 because the cli's wat handler runs on signal,
        // doesn't terminate the cli).
        assert_eq!(
            code,
            Some(0),
            "polling contract: cli should exit 0 after child observes stopped? \
             and returns clean; got {:?}",
            code
        );
    });
}

#[test]
fn sigterm_cascades_two_levels_via_process_group() {
    // Arc 106 slice 3 — the cascade-depth proof.
    //
    // The cli's direct child IS its own process group leader (slice 1's
    // setpgid). When the wat program in that child calls
    // `:wat::kernel::fork-program-ast` to spawn a grandchild, the
    // grandchild inherits the parent's pgid by POSIX default — no
    // setpgid call in the grandchild's `child_branch` path. Result: cli
    // + child + grandchild are all in the same process group; the cli's
    // `killpg(CHILD_PGID, sig)` cascades to every member in one syscall.
    //
    // This test verifies the cascade reaches depth 2:
    //   1. cli forks parent (depth 1 — child_branch_from_source: setpgid).
    //   2. Parent wat program forks grandchild (depth 2 — child_branch:
    //      no setpgid, inherits parent's pgid).
    //   3. Both processes poll `(:wat::kernel::stopped?)`.
    //   4. Both print READY markers; parent forwards grandchild's READY
    //      to its own stdout for the test to observe.
    //   5. Test sends SIGTERM to cli.
    //   6. cli's handler: flips KERNEL_STOPPED + killpg → kernel
    //      delivers SIGTERM to BOTH parent and grandchild.
    //   7. Each handler flips its own KERNEL_STOPPED.
    //   8. Both poll loops observe stopped → return clean.
    //   9. Grandchild exits 0 → its stdout closes → parent's
    //      forward-loop sees :None → returns.
    //   10. Parent calls wait-child → reaps grandchild → exits 0.
    //   11. cli's waitpid sees WIFEXITED 0 → cli exits 0.
    //
    // If pgid inheritance broke (grandchild in its own group), step 6
    // would only signal the parent; grandchild keeps running; parent's
    // forward-loop never returns; test hangs. The test's clean exit IS
    // the cascade proof.
    wat::fork::run_in_fork(|| {
        let program = r#"
            ;; Parent: forks grandchild via fork-program-ast, then
            ;; forwards grandchild's stdout to its own stdout. When
            ;; SIGTERM cascades through the group, grandchild observes
            ;; stopped, returns, closes its stdout — parent's
            ;; read-line returns :None — parent exits its forward
            ;; loop, reaps grandchild via wait-child, returns ().

            (:wat::core::define
              (:demo::forward-loop
                (rx :wat::io::IOReader)
                (out :wat::io::IOWriter)
                -> :())
              (:wat::core::match (:wat::io::IOReader/read-line rx) -> :()
                (:None ())
                ((Some line)
                  (:wat::core::let*
                    (((_ :()) (:wat::io::IOWriter/println out line)))
                    (:demo::forward-loop rx out)))))

            (:wat::core::define (:user::main
                                 (stdin  :wat::io::IOReader)
                                 (stdout :wat::io::IOWriter)
                                 (stderr :wat::io::IOWriter)
                                 -> :())
              (:wat::core::let*
                (((_ :()) (:wat::io::IOWriter/println stdout "PARENT READY"))
                 ;; Grandchild source: prints GRANDCHILD READY then
                 ;; polls stopped?. Pgid inherited from parent (no
                 ;; setpgid in child_branch); cascade reaches it via
                 ;; the cli's killpg.
                 ((child :wat::kernel::ForkedChild<(),()>)
                  (:wat::kernel::fork-program-ast
                    (:wat::test::program
                      (:wat::core::define (:demo::poll-loop -> :())
                        (:wat::core::if (:wat::kernel::stopped?) -> :()
                          ()
                          (:demo::poll-loop)))
                      (:wat::core::define (:user::main
                                           (gstdin  :wat::io::IOReader)
                                           (gstdout :wat::io::IOWriter)
                                           (gstderr :wat::io::IOWriter)
                                           -> :())
                        (:wat::core::let*
                          (((_ :()) (:wat::io::IOWriter/println gstdout "GRANDCHILD READY")))
                          (:demo::poll-loop))))))
                 ((rx :wat::io::IOReader)
                  (:wat::kernel::ForkedChild/stdout child))
                 ((handle :wat::kernel::ChildHandle)
                  (:wat::kernel::ForkedChild/handle child))
                 ((_ :()) (:demo::forward-loop rx stdout))
                 ((_exit :i64) (:wat::kernel::wait-child handle)))
                ()))
        "#;
        let path = write_temp(program);
        let bin = env!("CARGO_BIN_EXE_wat");
        let mut child = Command::new(bin)
            .arg(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn wat");

        // Lock-step: read both READY markers. Order is deterministic —
        // parent prints PARENT READY before forking, then forks +
        // forwards grandchild's stdout. Once the parent's read-line
        // returns the GRANDCHILD READY line, the parent forwards it.
        // Test reads two lines.
        use std::io::{BufRead, BufReader};
        let stdout = child.stdout.take().expect("child stdout");
        let mut reader = BufReader::new(stdout);
        let mut parent_line = String::new();
        let mut grandchild_line = String::new();
        reader.read_line(&mut parent_line).expect("read parent READY");
        reader
            .read_line(&mut grandchild_line)
            .expect("read grandchild READY");
        assert_eq!(parent_line.trim(), "PARENT READY");
        assert_eq!(grandchild_line.trim(), "GRANDCHILD READY");

        // Both processes are in the polling loop. SIGTERM the cli;
        // cascade reaches both via process group.
        let cli_pid = child.id() as libc::pid_t;
        unsafe {
            libc::kill(cli_pid, libc::SIGTERM);
        }

        let status = child.wait().expect("wait wat-cli");
        let code = status.code();
        let _ = std::fs::remove_file(&path);

        // The grandchild observed stopped via cascade → exited 0;
        // parent's forward-loop saw :None → returned; parent reaped
        // grandchild → exited 0; cli waitpid → 0. The cascade is the
        // load-bearing claim; cli's exit 0 is the proof.
        assert_eq!(
            code,
            Some(0),
            "cascade proof: cli should exit 0 — both parent AND grandchild \
             observed stopped? via process-group cascade and returned clean; got {:?}",
            code
        );
    });
}



