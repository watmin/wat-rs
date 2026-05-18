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
/// - canonical [] -> :nil signature (arc 170)
/// - kernel readln / println EDN-only contract (arc 170 slice 1f-ι)
/// - crossbeam channel wiring
/// - stdio bridge threads
/// - clean shutdown
///
/// Arc 170 migration: signature drops IOReader/IOWriter params; argv
/// is ambient; stdin is read via `(:wat::kernel::readln -> :String)`
/// which expects EDN-encoded input on the wire (quoted string);
/// stdout is written via `(:wat::kernel::println ...)` which emits
/// the EDN-encoded form (quoted string) followed by a newline.
/// Rust scaffolding sends EDN-quoted `"watmin"` on stdin and asserts
/// the EDN-quoted form on stdout.
const ECHO_PROGRAM: &str = r#"

(:wat::core::define (:user::main -> :wat::core::nil)
  (:wat::core::let
    [line (:wat::kernel::readln -> :wat::core::String)]
    (:wat::kernel::println line)))
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

    // Pipe EDN-encoded "watmin" to child stdin (arc 170 slice 1f-ι
    // EDN-only contract: readln -> :String expects a quoted EDN String
    // on the wire, i.e. `"watmin"\n` with literal double-quotes).
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"\"watmin\"\n")
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
    // println re-EDN-encodes the String value: output is `"watmin"\n`
    // (with literal double-quotes, per slice 1f-ι contract).
    assert_eq!(stdout, "\"watmin\"\n", "stdout mismatch: {:?}", stdout);
}

/// Programs-are-atoms hello-world (structural side). Demonstrates the
/// structural wrap/unwrap round-trip:
///
/// 1. `(:wat::core::quote ...)` captures a println expression as a
///    `:wat::WatAST` without firing its side effects.
/// 2. `(:wat::holon::Atom program)` wraps the WatAST as an Atom
///    holon — the program is now a typed box in the algebra.
/// 3. `(:wat::holon::to-watast program-atom)` extracts the payload back
///    as a `:wat::WatAST`. Structural field read; exact; no cosine.
/// 4. `(:wat::eval-ast! reveal)` executes the program under
///    constrained eval.
///
/// This proves the STRUCTURAL side of programs-as-atoms: `(Atom x) →
/// to-watast → x` is lossless, exact, and carries arbitrary
/// wat programs as data.
///
/// Arc 170 migration: outer main uses canonical [] -> :nil signature;
/// inner quoted program uses (:wat::kernel::println "wat-atoms") —
/// the println call is the load-bearing expression captured as data
/// and re-executed via eval-ast!. No stdin required.
const PROGRAMS_ARE_ATOMS_PROGRAM: &str = r#"

(:wat::core::define (:user::main -> :wat::core::nil)
  (:wat::core::let
    [program
       (:wat::core::quote
         (:wat::kernel::println "wat-atoms"))
     program-atom
       (:wat::holon::Atom program)
     ;; arc 057 Story-2 recovery: program-atom is now a structural
     ;; HolonAST (the form lowered onto the algebra grid). to-watast
     ;; lifts it back to a runnable WatAST; eval-ast! fires it.
     reveal
       (:wat::holon::to-watast program-atom)]
    ;; eval-ast! returns :Result<wat::holon::HolonAST, EvalError> per
    ;; the 2026-04-20 INSCRIPTION. Match both arms to preserve main's
    ;; declared return type of :(). Err arm is unreachable here
    ;; (the quoted program is well-formed and non-mutating).
    (:wat::core::match (:wat::eval-ast! reveal) -> :wat::core::nil
      ((Ok _) ())
      ((Err _) ()))))
"#;

#[test]
fn programs_are_atoms_hello_world() {
    let path = write_temp(PROGRAMS_ARE_ATOMS_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat");

    let output = child.wait_with_output().expect("wait");
    let _ = std::fs::remove_file(&path);

    assert!(
        output.status.success(),
        "wat exit {:?}, stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    // println EDN-encodes the String "wat-atoms" → `"wat-atoms"\n`
    // (arc 170 slice 1f-ι EDN-only contract).
    assert_eq!(
        stdout, "\"wat-atoms\"\n",
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
/// 1. `(:wat::core::quote ...)` captures a println expression as a
///    `:wat::WatAST`.
/// 2. `(:wat::holon::Atom program)` wraps it as an Atom holon.
/// 3. `(:wat::holon::Bind key-atom program-atom)` composes the Atom
///    with a key — the resulting vector is ROUGHLY ORTHOGONAL to
///    program-atom. `presence?` returns false → "absent" printed.
///    The "absent" IS the proof the signal was bound away.
/// 4. `(:wat::holon::Bind bound key-atom)` — MAP self-inverse:
///    `bind(bind(k,p), k) ≈ p`. `presence?` returns true → "present"
///    printed. The "present" is the proof the algebra recovered signal.
/// 5. `to-watast` + `eval-ast!` fires the quoted println program.
///
/// Arc 170 migration: outer main uses canonical [] -> :nil; inner
/// quoted program uses `(:wat::kernel::println "wat-atoms")` instead
/// of the retired IOReader/IOWriter stdin-echo path. Presence proof
/// prints "absent"/"present" via println (EDN-encoded Strings).
/// Observable stdout: `"absent"\n"present"\n"wat-atoms"\n`.
const PRESENCE_PROOF_PROGRAM: &str = r#"

(:wat::core::define (:user::main -> :wat::core::nil)
  (:wat::core::let
    [program
       (:wat::core::quote
         (:wat::kernel::println "wat-atoms"))
     program-atom
       (:wat::holon::Atom program)
     key-atom
       (:wat::holon::Atom "hello-world")

     ;; Compose: program-atom bound under key-atom.
     bound
       (:wat::holon::Bind key-atom program-atom)

     ;; Substrate proof #1: program-atom's signal is GONE from bound.
     ;; Arc 037 slice 3: presence? does the honest per-d threshold
     ;; comparison internally. absent = not present.
     _
       (:wat::kernel::println
         (:wat::core::if
           (:wat::holon::presence? program-atom bound)
           -> :wat::core::String
           "present"
           "absent"))

     ;; Self-inverse: bind(bind(k, p), k) recovers p at the vector level.
     recovered
       (:wat::holon::Bind bound key-atom)

     ;; Substrate proof #2: program-atom's signal is BACK in recovered.
     _
       (:wat::kernel::println
         (:wat::core::if
           (:wat::holon::presence? program-atom recovered)
           -> :wat::core::String
           "present"
           "absent"))

     ;; arc 057 Story-2 recovery: program-atom is the structurally
     ;; lowered HolonAST. to-watast lifts it back to a runnable WatAST.
     ;; The presence measurements above proved the vector dynamics;
     ;; this line runs the actual program.
     reveal
       (:wat::holon::to-watast program-atom)]
    ;; eval-ast! returns :Result<wat::holon::HolonAST, EvalError> per
    ;; the 2026-04-20 INSCRIPTION.
    (:wat::core::match (:wat::eval-ast! reveal) -> :wat::core::nil
      ((Ok _) ())
      ((Err _) ()))))
"#;

#[test]
fn presence_proof_hello_world() {
    let path = write_temp(PRESENCE_PROOF_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat");

    let output = child.wait_with_output().expect("wait");
    let _ = std::fs::remove_file(&path);

    assert!(
        output.status.success(),
        "wat exit {:?}, stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    // println EDN-encodes each String with quotes (arc 170 slice 1f-ι):
    //   "absent"\n  — program-atom signal NOT in bound (proof #1)
    //   "present"\n — program-atom signal recovered (proof #2)
    //   "wat-atoms"\n — eval-ast! fires the quoted println program
    assert_eq!(
        stdout, "\"absent\"\n\"present\"\n\"wat-atoms\"\n",
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

// wrong_arity_user_main_rejected — DELETED (arc 170 migration).
//
// Pre-arc-170: `:user::main` required a 3-arg (IOReader/IOWriter×2)
// or 4-arg signature; declaring zero params fired EXIT_MAIN_SIGNATURE=4.
//
// Post-arc-170: `(:user::main -> :wat::core::nil)` with zero params IS
// the canonical shape (arc 170 REALIZATIONS pass 7 + pass 10). The
// scenario this test exercised — "zero params is wrong" — is now inverted:
// zero params is CORRECT. Deleting the test avoids asserting the
// opposite of the substrate's contract. The canonical shape is proven
// by `t1_canonical_nil_main_freezes` in `tests/wat_arc170_program_contracts.rs`.

#[test]
fn wrong_arg_type_user_main_rejected() {
    // Any non-canonical :user::main signature fires BareLegacyMainSignature
    // at startup (arc 170 slice 1e). Under arc 170, the 3-arg
    // IOReader/IOWriter×2 shape is a check error surfaced by the walker;
    // EXIT_STARTUP_ERROR=3 (not EXIT_MAIN_SIGNATURE=4) because the
    // BareLegacy diagnostic fires at type-check time during startup.
    // The exact param type (i64 vs IOReader) is irrelevant — the whole
    // shape is retired.
    let program = r#"
        (:wat::core::define (:user::main
                             (stdin  :wat::core::i64)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :wat::core::nil)
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

    // Arc 170: BareLegacyMainSignature fires at type-check → EXIT_STARTUP_ERROR=3.
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(":user::main") || stderr.contains("legacy") || stderr.contains("canonical"),
        "stderr should mention the legacy main signature; got: {}",
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
    // Arc 211b — child's stderr now carries the structured #wat.kernel/ProcessPanics
    // EDN envelope (slice 1i) wrapping a StartupError variant. The substrate's
    // panic-as-EDN doctrine (arc 211b) supersedes the pre-211 "startup:" text prefix.
    assert!(
        stderr.contains("#wat.kernel/ProcessPanics") && stderr.contains("StartupError"),
        "stderr should contain structured ProcessPanics envelope with StartupError variant; got: {}",
        stderr
    );
}

#[test]
fn program_writes_multiple_times_to_stdout() {
    // :user::main calls println twice; stdout accumulates both writes.
    // Arc 170 migration: canonical [] -> :nil signature; IOWriter/print
    // retired in favour of (:wat::kernel::println ...). Each println
    // emits one EDN-encoded line. Two calls → two EDN lines on stdout.
    // Rust assertion updated for arc 170 slice 1f-ι EDN-only contract:
    // println of a String value emits the EDN-quoted form with newline.
    let program = r#"
        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::do
            (:wat::kernel::println "hello")
            (:wat::kernel::println "world")
            :wat::core::nil))
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
    // Each println EDN-encodes its String argument:
    //   "hello"\n — first call
    //   "world"\n — second call
    assert_eq!(stdout, "\"hello\"\n\"world\"\n", "got: {:?}", stdout);
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
            ;; Arc 170 migration: canonical [] -> :nil signatures;
            ;; IOWriter/println → (:wat::kernel::println ...);
            ;; demo::loop no longer needs a stdout param — println
            ;; routes through the ambient StdOutService.
            (:wat::core::define (:demo::loop -> :wat::core::nil)
              (:wat::core::if (:wat::kernel::stopped?) -> :wat::core::nil
                ()                                       ; observed stop → return clean
                (:demo::loop)))                          ; tight poll loop

            (:wat::core::define (:user::main -> :wat::core::nil)
              (:wat::core::do
                (:wat::kernel::println "READY")
                (:demo::loop)))
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
        // Arc 170 slice 1f-ι: println EDN-encodes the String value —
        // `"READY"\n` on the wire. Trim and strip surrounding EDN quotes.
        let trimmed = line.trim().trim_matches('"');
        assert_eq!(trimmed, "READY", "expected READY marker; got {:?}", line);

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

// sigterm_cascades_two_levels_via_process_group — DELETED (arc 170 migration).
//
// Pre-arc-170: this test embedded a wat program that used
// `:wat::kernel::fork-program-ast` to spawn a grandchild, then forwarded
// the grandchild's stdout via IOReader/IOWriter line-by-line. The two-level
// cascade proof depended on BOTH fork-program-ast AND the old 3-arg
// `:user::main` (stdin/stdout/stderr) in the grandchild.
//
// Post-arc-170: `fork-program-ast` is a retired primitive (fires
// BareLegacyForkProgram at type-check). The canonical replacement is
// `:wat::kernel::spawn-process worker-fn` (typed channels, no raw
// stdin/stdout pipe access from the WAT side). Migrating this test would
// require a full spawn-process grandchild — a Pattern B1 rewrite
// (typed-channel + process-group inheritance proof), not a const-string
// Pattern B2 migration. The scenario is preserved in intention: the
// arc 106 process-group cascade discipline is proven by
// `sigterm_to_cli_cascades_via_polling_contract` (depth-1) and the
// substrate's pgid mechanics are unchanged. A depth-2 spawn-process proof
// belongs in `tests/wat_arc170_program_contracts.rs` as a T17 entry.
// Deleting here; no new test added (out of B2 scope).

// ─── Arc 115 slice 1 — `wat --check` mode ────────────────────────────────

const ARC115_GOOD_PROGRAM: &str = r#"
(:wat::core::define
  (:user::main -> :wat::core::nil)
  ())
"#;

const ARC115_BAD_PROGRAM: &str = r#"
(:wat::core::define
  (:user::main -> :wat::core::nil)
  (:wat::kernel::send no-such-thing 42))
"#;

#[test]
fn check_mode_exits_zero_on_good_program() {
    let path = write_temp(ARC115_GOOD_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg("--check")
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    let _ = std::fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(0));
    assert!(
        output.stdout.is_empty(),
        "stdout should be empty in default mode; got: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn check_mode_exits_nonzero_on_bad_program() {
    let path = write_temp(ARC115_BAD_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg("--check")
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    let _ = std::fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Default mode → text Display via stderr.
    assert!(
        stderr.contains("type-check error"),
        "stderr should contain type-check error; got: {}",
        stderr
    );
}

#[test]
fn check_output_edn_emits_record_per_diagnostic() {
    let path = write_temp(ARC115_BAD_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .args(["--check", "--check-output", "edn"])
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    let _ = std::fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    // ARC115_BAD_PROGRAM produces 2 type-check errors: one
    // CommCallOutOfPosition + one ReturnTypeMismatch. Each surfaces
    // as one EDN record on its own line.
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "expected 2 EDN records (one per CheckError); got {}: {}",
        lines.len(),
        stdout
    );
    assert!(
        lines[0].starts_with("#wat.diag/CommCallOutOfPosition"),
        "first line should be CommCallOutOfPosition tag; got: {}",
        lines[0]
    );
    assert!(
        lines[1].starts_with("#wat.diag/ReturnTypeMismatch"),
        "second line should be ReturnTypeMismatch tag; got: {}",
        lines[1]
    );
    // Structured fields preserved verbatim — not text-wrapped.
    assert!(lines[0].contains(":callee \":wat::kernel::send\""));
    assert!(lines[1].contains(":function \":user::main\""));
}

#[test]
fn check_output_json_emits_record_per_diagnostic() {
    let path = write_temp(ARC115_BAD_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .args(["--check", "--check-output", "json"])
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    let _ = std::fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "expected 2 JSON records (one per CheckError); got {}: {}",
        lines.len(),
        stdout
    );
    assert!(
        lines[0].contains("\"kind\":\"CommCallOutOfPosition\""),
        "first line should have kind=CommCallOutOfPosition; got: {}",
        lines[0]
    );
    assert!(
        lines[1].contains("\"kind\":\"ReturnTypeMismatch\""),
        "second line should have kind=ReturnTypeMismatch; got: {}",
        lines[1]
    );
    assert!(lines[0].contains("\"callee\":\":wat::kernel::send\""));
    assert!(lines[1].contains("\"function\":\":user::main\""));
}

#[test]
fn check_output_without_check_flag_is_usage_error() {
    let path = write_temp(ARC115_GOOD_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .args(["--check-output", "edn"])
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    let _ = std::fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(64)); // EX_USAGE
}
