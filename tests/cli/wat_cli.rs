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
/// is ambient; stdin is read via `(:wat::kernel::readln)`
/// which expects EDN-encoded input on the wire (quoted string);
/// stdout is written via `(:wat::kernel::println ...)` which emits
/// the EDN-encoded form (quoted string) followed by a newline.
/// Rust scaffolding sends EDN-quoted `"watmin"` on stdin and asserts
/// the EDN-quoted form on stdout.
const ECHO_PROGRAM: &str = include_str!("wat_cli__echo_program.wat");

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
const PROGRAMS_ARE_ATOMS_PROGRAM: &str = include_str!("wat_cli__programs_are_atoms.wat");

#[test]
fn programs_are_atoms_hello_world() {
    let path = write_temp(PROGRAMS_ARE_ATOMS_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let child = Command::new(bin)
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
const PRESENCE_PROOF_PROGRAM: &str = include_str!("wat_cli__presence_proof.wat");

#[test]
fn presence_proof_hello_world() {
    let path = write_temp(PRESENCE_PROOF_PROGRAM);
    let bin = env!("CARGO_BIN_EXE_wat");
    let child = Command::new(bin)
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
    assert!( // rune:lint(loose-assert) — subprocess stderr embeds temp file path (pid + nanosecond timestamp via write_temp); full stderr is non-deterministic
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
    let program = include_str!("wat_cli__wrong_arg_type_main.wat");
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
    assert!( // rune:lint(loose-assert) — subprocess stderr embeds temp file path (pid + nanosecond timestamp via write_temp); full stderr is non-deterministic
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
    let program = include_str!("wat_cli__bad_capacity_mode.wat");
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
    assert!( // rune:lint(loose-assert) — subprocess stderr embeds temp file path (pid + nanosecond timestamp via write_temp); full stderr is non-deterministic
        stderr.contains("#wat.kernel.LociDiedError/StartupError"),
        "stderr should contain structured ProcessPanics envelope with StartupError variant; got: {}",
        stderr
    );
}

#[test]
fn freeze_time_panic_surfaces_structured_not_silent() {
    // Arc 278 no-hidden-failures (R41 EGO SVM LEX): a PANIC during freeze-time
    // evaluation of a top-level form (here Result/expect on an Err) must NOT
    // vanish. PRE-FIX it exits 1 with ZERO bytes on BOTH streams — the child's
    // outer catch_unwind (src/process/clone.rs:429) drops the payload + _exit(1),
    // and the silent panic hook (src/process/child.rs) eats the default text.
    // The runtime-panic-in-main path is loud (its own catch at
    // src/process/verbs.rs:342); the freeze call (src/process/verbs.rs:614) has
    // no catch — that asymmetry IS the defect. POST-FIX: the child catches the
    // freeze panic, emits the structured #wat.kernel/ProcessPanics envelope
    // (preserving the AssertionFailure payload), and exits EXIT_STARTUP_ERROR=3
    // — a freeze-time failure is a STARTUP failure (phase-honest; four-questions).
    // A top-level `let` is const-eval'd during freeze (a bare call expr is not);
    // its initializer Result/expect's on an eval-ast! Err (unknown verb) → panics
    // at freeze time. PRE-FIX: mute exit 1 (0 bytes both streams).
    let program = include_str!("wat_cli__freeze_time_panic.wat");
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    let _ = std::fs::remove_file(&path);
    assert_eq!(
        output.status.code(),
        Some(3),
        "freeze-time panic must exit 3 (startup failure), not mute exit 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!( // rune:lint(loose-assert) — subprocess stderr embeds temp file path (pid + nanosecond timestamp via write_temp); full stderr is non-deterministic
        stderr.contains("#wat.kernel.LociDiedError/Panic") && stderr.contains("freeze-time boom"),
        "freeze-time panic must surface the structured ProcessPanics envelope carrying its reason, not silence; got: {}",
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
    let program = include_str!("wat_cli__multiple_println.wat");
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
    // The wat-native polling contract: SIGTERM to a running `wat` reaches the
    // program as a FLAG it polls, not as a kill.
    //
    //   1. The cli installs the substrate's signal handlers at startup.
    //   2. SIGTERM arrives → the handler flips KERNEL_STOPPED. That is all it
    //      does; a handler must be async-signal-safe, so it is one atomic store.
    //   3. The wat program polls `(:wat::kernel::stopped?)`, observes true, and
    //      returns cleanly. `:user::main` returns ().
    //   4. The process exits 0.
    //
    // ⚠ This comment was REWRITTEN 2026-07-27. It used to describe a fork: the
    // cli forking the program into a child, `child_branch_from_source`
    // installing handlers post-fork, a `CHILD_PGID` + `killpg` cascade
    // broadcasting to the group. NONE of that exists. Arc 170 stopped the cli
    // forking (`f56ad55b` — `wat <file>` runs in-process), and the killpg
    // cascade was already fictional before that, because every spawned child
    // calls `setpgid(0, 0)` and so sits in its own group that a killpg on the
    // cli's group never reaches. `child_branch_from_source` and `CHILD_PGID`
    // now appear in `src/` only inside comments. A test whose doc narrates a
    // retired mechanism reads as live code and is the graveyard this arc
    // exists to burn.
    //
    // Lock-step via a stdout marker: the program prints "READY" when it is
    // about to enter the polling loop, and only then does the test send
    // SIGTERM. No sleep — the wire IS the synchronization.
    //
    // FLAKE DISPOSITION (2026-07-27): this failed ONCE under a loaded run and
    // was parked to be re-checked after the execve landed. Re-checked and NOT
    // REPRODUCIBLE — 25/25 isolated, 6/6 with the cli binary at 32 threads on
    // 14 cores, and green in every whole-floor run including two at 2x
    // over-subscription. The mechanism is gone too, but NOT for the reason
    // predicted: the guess was that exec would fix it by giving children fresh
    // KERNEL_STOPPED/handler state, and exec does do that — for spawn-process
    // children. This path has no fork at all since `f56ad55b`, so the cli
    // de-fork is what removed the race here, and it landed BEFORE the exec.
    let program = include_str!("wat_cli__sigterm_polling_loop.wat");
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // Arc 170 — was `Stdio::null()`. A test that DISCARDS the reason for its own failure is a
        // mask, and this one masked the reason it exists to prove: wat writes a structured
        // `#wat.kernel/…` diagnostic to stderr immediately before a non-zero exit, and routing it
        // to /dev/null left every failure reporting only `got Some(2)` — an exit code with no
        // cause attached. This failure has flipped state five times in one day; the reason is the
        // only thing that can close it, and it was being thrown away at the source.
        .stderr(Stdio::piped())
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

    // Drain the child's stderr BEFORE wait(): it is a pipe now, and the reason wat writes on its
    // way out is the only evidence that can close this. Read to EOF, which the child's exit gives us.
    let mut child_stderr = String::new();
    if let Some(mut e) = child.stderr.take() {
        use std::io::Read;
        let _ = e.read_to_string(&mut child_stderr);
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
         and returns clean; got {:?}\n\
         ── child stderr (the reason; empty means it died without writing one) ──\n{}",
        code,
        if child_stderr.trim().is_empty() { "<empty>" } else { child_stderr.trim() }
    );
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

const ARC115_GOOD_PROGRAM: &str = include_str!("wat_cli__check_good.wat");

const ARC115_BAD_PROGRAM: &str = include_str!("wat_cli__check_bad.wat");

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
    assert!( // rune:lint(loose-assert) — subprocess stderr embeds temp file path (pid + nanosecond timestamp via write_temp); full stderr is non-deterministic
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
    // TypeMismatch + one ReturnTypeMismatch. Each surfaces
    // as one EDN record on its own line.
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "expected 2 EDN records (one per CheckError); got {}: {}",
        lines.len(),
        stdout
    );
    // Arc 296: namespace changed from wat.diag to wat.check.
    //
    // ⚠ ORDER FLIPPED AT ARC 278 C20, AND THE FLIP IS THE CONTRACT NOW. Check errors leave
    // `check_program` sorted into SOURCE order (`check::error::sort_into_source_order`), so the
    // record that comes first is the one whose span STARTS first. Here that is the
    // ReturnTypeMismatch: its span is the WHOLE body form, and the TypeMismatch sits at the
    // argument inside it, so the body's start comes first. (No line number is quoted here on
    // purpose — the fixture's leading comment block moves them, and a cite that its own file
    // can invalidate is one nobody re-derives.) Before the sort this pair came out in
    // emission order, which was not a property of anything a reader could see — the program has
    // exactly one function, so its single-entry `HashMap` happened not to expose the hash
    // randomisation the sort was written to kill.
    assert!( // rune:lint(loose-assert) — each EDN line includes :file "..." with the temp path (pid + nanosecond timestamp via write_temp); full line is non-deterministic
        lines[0].starts_with("#wat.check/ReturnTypeMismatch"),
        "first line should be ReturnTypeMismatch tag (its span is the whole body form, which \
         starts before the argument mismatch inside it); got: {}",
        lines[0]
    );
    assert!( // rune:lint(loose-assert) — each EDN line includes :file "..." with the temp path (pid + nanosecond timestamp via write_temp); full line is non-deterministic
        lines[1].starts_with("#wat.check/TypeMismatch"),
        "second line should be TypeMismatch tag; got: {}",
        lines[1]
    );
    // Structured fields preserved verbatim — not text-wrapped.
    // :file field is prepended first; function and callee follow.
    assert!(lines[0].contains(":function \":user::main\"")); // rune:lint(loose-assert) — EDN line includes variable :file field (temp path with pid + nanosecond timestamp); targeted field check is the contract
    assert!(lines[1].contains(":callee \":wat::core::i64::+\"")); // rune:lint(loose-assert) — EDN line includes variable :file field (temp path with pid + nanosecond timestamp); targeted field check is the contract
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
    // Arc 296: JSON shape uses #tag sentinel; field keys carry EDN colon prefix.
    //
    // ⚠ ORDER FLIPPED AT ARC 278 C20 — see the EDN twin above for why. Source order, so the
    // record whose span starts first (the ReturnTypeMismatch over the whole body form) leads.
    assert!( // rune:lint(loose-assert) — each JSON line includes ":file":"..." with the temp path (pid + nanosecond timestamp via write_temp); full line is non-deterministic
        lines[0].contains("\"#tag\":\"wat.check/ReturnTypeMismatch\""),
        "first line should have #tag=wat.check/ReturnTypeMismatch; got: {}",
        lines[0]
    );
    assert!( // rune:lint(loose-assert) — each JSON line includes ":file":"..." with the temp path (pid + nanosecond timestamp via write_temp); full line is non-deterministic
        lines[1].contains("\"#tag\":\"wat.check/TypeMismatch\""),
        "second line should have #tag=wat.check/TypeMismatch; got: {}",
        lines[1]
    );
    // Keyword field keys carry leading colon in JSON (EDN keyword serialization).
    assert!(lines[0].contains("\":function\":\":user::main\"")); // rune:lint(loose-assert) — JSON line includes variable \":file\":\"...\" field (temp path with pid + nanosecond timestamp); targeted field check is the contract
    assert!(lines[1].contains("\":callee\":\":wat::core::i64::+\"")); // rune:lint(loose-assert) — JSON line includes variable \":file\":\"...\" field (temp path with pid + nanosecond timestamp); targeted field check is the contract
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

// ─── Arc 170 — argv passthrough (the arc's own purpose, finally wired) ───────
//
// Arc 170 built the ambient (`ARGV` OnceLock, `set_argv`, the
// `(:wat::runtime::argv)` verb) and wrote `distribution/mod.rs`'s promise that
// "argv[2..N] = subsequent shell args … every shell arg passes through
// unfiltered." Nothing passed through: `argv::parse` demanded EXACTLY one
// positional, a gate arc 115 (2b397cc0) installed to enforce `--check`'s
// grammar and that nobody revisited when 170 added the pipe. The valve had been
// shut since before the pipe was laid.
//
// The layout is the OS shell's, unmodified: argv[0] = path to the wat binary,
// argv[1] = path to the entry file, argv[2..] = whatever else the caller said.
// `set_argv` already receives the whole vector untouched — only the gate moved.

#[test]
fn argv_passes_shell_args_through_to_user_main() {
    let path = write_temp(include_str!("wat_cli__argv_passthrough.wat"));
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg(&path)
        .arg("--some")
        .arg("arg")
        .arg("42")
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat");
    let _ = std::fs::remove_file(&path);

    assert_eq!(
        output.status.code(),
        Some(0),
        "extra shell args must be accepted, not EX_USAGE; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // wat stdio is EDN — assert the STRUCTURE, not a substring. `println` emits
    // the Vector<String> as one EDN line.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed = wat_edn::parse_owned(stdout.trim()).expect("argv line must be EDN");
    let items = match &parsed {
        wat_edn::OwnedValue::Vector(v) => v,
        other => panic!("argv must be an EDN vector; got {other:?}"),
    };
    let strings: Vec<&str> = items
        .iter()
        .map(|v| match v {
            wat_edn::OwnedValue::String(s) => &s[..],
            other => panic!("every argv element must be a String; got {other:?}"),
        })
        .collect();

    // argv[0] is the RESOLVED binary (current_exe), which may differ from the
    // spelling cargo handed us — compare canonicalised, not verbatim.
    let expected_bin = std::fs::canonicalize(bin).expect("canonicalize bin");
    let actual_bin = std::fs::canonicalize(strings[0]).expect("canonicalize argv[0]");
    assert_eq!(actual_bin, expected_bin, "argv[0] must be the wat binary");
    assert_eq!(
        std::path::Path::new(strings[1]),
        path.as_path(),
        "argv[1] must be the entry path"
    );
    assert_eq!(
        &strings[2..],
        &["--some", "arg", "42"],
        "argv[2..] must be the caller's args, unedited"
    );
}

#[test]
fn check_mode_still_demands_exactly_one_entry() {
    // Arity belongs to the MODE, not to the parser globally. `--check` verifies
    // ONE file; handing it two is a usage error even though the run path now
    // accepts trailing args.
    // Any valid entry works — the parser rejects on arity before it reads the
    // file at all, so this reuses the passthrough fixture rather than minting a
    // second one for a body that is never parsed.
    let path = write_temp(include_str!("wat_cli__argv_passthrough.wat"));
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg("--check")
        .arg(&path)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat");
    let _ = std::fs::remove_file(&path);
    assert_eq!(
        output.status.code(),
        Some(64),
        "--check with two entries must be EX_USAGE"
    );
}

/// Arc 170 — SIGTERM must reach a program parked in a READ, not only one spinning on
/// `stopped?`.
///
/// `sigterm_to_cli_cascades_via_polling_contract` (above) proves the contract for a COMPUTE
/// loop, which reaches its own poll unaided. This proves it where the program is blocked in
/// `read-frame` — the shape of every interactive wat program: the REPL, the stdio-service
/// demo, `repl-daemon`.
///
/// MEASURED at HEAD, by hand, before this test existed: such a program survives SIGTERM AND
/// SIGINT and requires SIGKILL. Ctrl-C does not work. The cause is that `RealStdin`
/// (`src/io.rs`) reports `as_raw_fd_for_poll() -> None` although it wraps fd 0, so its read is
/// a bare blocking `read(2)` — the one wait in the substrate that is not a select. Every other
/// wait (admin, clients, timers, lifeline, and a spawned child's `PipeReader` stdin) is
/// multiplexed; `channel/transfer.rs` even implements the exact poll-`[fd, broadcast_fd]`
/// pattern this read needs.
///
/// The blast radius is the whole interactive surface: no `Ctrl-C`, and `systemctl stop` /
/// container SIGTERM all degrade to hard-kill with no cleanup, silently.
/// ⛔ TRACKED RED GATE — `#[ignore]`d because the work it gates is unfinished, NOT because
/// the assertion is wrong. Un-ignore when step 2 below lands; it is the acceptance test.
///
/// Step 1 (`RecvError::Shutdown` carried instead of erased by a wildcard) IS in, and this
/// gate's failure MESSAGE changed as a result — from the false `"peer closed"` to a real
/// type error, because carrying `Shutdown` made a previously-dead path fire:
///
///   `:wat::kernel::LociDiedError/message: expected wat::kernel::*DiedError,
///    got wat::core::Record <wat::kernel::Failure{"service peer lost …"}>`
///   at `wat/kernel/services/stdio.wat:210`, cause built at `wat/spawn.wat:351`
///
/// So there is a SECOND defect behind the first: a `RecvOutcome::Lost` producer that hands
/// a `Failure` where a `LociDiedError` is required — an arc-278 LociDiedError-migration
/// straggler, latent only because the wildcard kept its path unreachable.
///
/// REMAINING WORK, in order:
///   1. ✅ carry `Shutdown` (`kernel/peer.rs`, `runtime.rs` — both unified-Peer arms).
///   2. ⛔ fix the `Failure`-vs-`LociDiedError` producer at `wat/spawn.wat:351`.
///   3. ⛔ `RealStdin::as_raw_fd_for_poll -> Some(0)` + route the stdin read through the
///      poll-`[fd, broadcast_fd]` multiplex `channel/transfer.rs:200` already implements.
///      Until then this read is the one wait in the substrate that is not a select, and it
///      pins the process alive until stdin EOFs.
#[test]
fn sigterm_reaches_a_program_blocked_on_stdin() {
    let program = include_str!("wat_cli__sigterm_blocked_on_stdin.wat");
    let path = write_temp(program);
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut child = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat");

    // Lock-step: READY means the program has reached its read and is parked there. The stdin
    // pipe is held open and deliberately never written, so the read cannot complete on its own
    // — the only thing that can end this process is the signal.
    use std::io::{BufRead, BufReader};
    let stdout = child.stdout.take().expect("child stdout");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).expect("read READY");
    assert_eq!(line.trim().trim_matches('"'), "READY", "expected READY; got {line:?}");

    unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) };

    // Poll rather than block: a FAILING run here means the process ignores the signal
    // forever, and a bare wait() would hang the suite instead of reporting.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let status = loop {
        match child.try_wait().expect("try_wait") {
            Some(s) => break Some(s),
            None if std::time::Instant::now() >= deadline => break None,
            None => std::thread::sleep(std::time::Duration::from_millis(25)),
        }
    };
    let mut child_stderr = String::new();
    if let Some(mut e) = child.stderr.take() {
        use std::io::Read;
        let _ = e.read_to_string(&mut child_stderr);
    }
    if status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let _ = std::fs::remove_file(&path);

    let status = status.expect(
        "the program IGNORED SIGTERM while blocked in read-frame and had to be SIGKILLed. \
         A wait that is not a select cannot observe a stop request — see this test's module \
         docs for the site (RealStdin::as_raw_fd_for_poll returning None).",
    );
    assert_eq!(
        status.code(),
        Some(0),
        "a program blocked in a read must observe the stop request and return cleanly; got {:?}\n\
         ── child stderr ──\n{}",
        status.code(),
        if child_stderr.trim().is_empty() { "<empty>" } else { child_stderr.trim() }
    );
}
