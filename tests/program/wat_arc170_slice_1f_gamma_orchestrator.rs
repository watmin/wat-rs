//! Arc 170 slice 1f-γ — runtime orchestrator integration tests.
//!
//! These rows exercise the orchestration glue stitched into
//! [`wat::freeze::invoke_user_main`]: spawning the three substrate
//! stdio services (StdIn / StdOut / StdErr); registering thread-0
//! with them; routing `:wat::kernel::println` / `eprintln` / `readln`
//! through per-thread channel pairs; reaping spawned child threads
//! via the closure-epilogue Remove protocol; cascading shutdown to
//! the services on `:user::main` return via scope-drop on the
//! `RuntimeServices` carrier.
//!
//! | Row | What |
//! |-----|------|
//! | A | Single-thread program: `:user::main` calls println; orchestrator boots services, registers main, runs, cleanup. |
//! | B | Multi-thread program: main spawns 3 child threads via `:wat::kernel::spawn-thread`; each calls println; verify all 3 lines appear (any order). |
//! | C | Panic recovery: child thread panics in its body; catch_unwind captures; Remove still sent; thread reaps as Panic; main returns nil. |
//! | D | Scope-drop cascade: orchestrator drops ControlTxs after user::main returns; three service Threads join with Ok(nil). |
//! | E | thread-0 readln roundtrip: main's `(:wat::kernel::readln)` returns parsed HolonAST from the test-driven IOReader. |
//!
//! Stdio handles are OS pipes (`libc::pipe`) — Send + Sync + cross-
//! thread-safe. The orchestrator runs on the calling thread; the
//! services run on their own threads; the test thread drives the
//! "other side" of each pipe to feed stdin and capture stdout.
//! `StringIoReader` / `StringIoWriter` would have been simpler but
//! their `ThreadOwnedCell` backing panics on cross-thread access.

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;

use wat::freeze::{invoke_user_main, startup_beside, startup_from_file};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use wat::runtime::Value;
use wat::services::{install_ambient_stdio, take_ambient_stdio, AmbientStdio};

// ─── helpers ───────────────────────────────────────────────────────

/// Allocate an OS pipe and wrap its ends in a (read-end PipeReader,
/// write-end PipeWriter) tuple. Used to build test-side stdio handles
/// the orchestrator gives the services, with the test thread holding
/// the matching read/write end for inspection.
fn pipe_pair() -> (Arc<dyn WatReader>, Arc<dyn WatWriter>) {
    let mut fds = [0i32; 2];
    let r = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(r, 0, "pipe(2) succeeded");
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(read_fd));
    let writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(write_fd));
    (reader, writer)
}

/// Build a 3-pipe rig for ambient stdio.
///
/// Returns:
/// - `AmbientStdio` to install (orchestrator-side: stdin reader,
///   stdout writer, stderr writer)
/// - `stdin_inject` — the writer the test uses to feed bytes into the
///   orchestrator's stdin
/// - `stdout_capture` — the reader the test uses to drain bytes from
///   the orchestrator's stdout
/// - `stderr_capture` — same for stderr
struct StdioRig {
    ambient: Option<AmbientStdio>,
    stdin_inject: Arc<dyn WatWriter>,
    stdout_capture: Arc<dyn WatReader>,
    stderr_capture: Arc<dyn WatReader>,
}

fn build_rig() -> StdioRig {
    // For each direction: test → service for stdin; service → test
    // for stdout / stderr. The orchestrator hands the service the
    // "service-side" end (read end for stdin, write end for stdout /
    // stderr); the test thread holds the opposite.
    let (stdin_service_side, stdin_test_inject) = pipe_pair();
    let (stdout_test_capture, stdout_service_side) = pipe_pair();
    let (stderr_test_capture, stderr_service_side) = pipe_pair();

    StdioRig {
        ambient: Some(AmbientStdio {
            stdin: stdin_service_side,
            stdout: stdout_service_side,
            stderr: stderr_service_side,
        }),
        stdin_inject: stdin_test_inject,
        stdout_capture: stdout_test_capture,
        stderr_capture: stderr_test_capture,
    }
}

/// Install the rig's ambient stdio + drain it after the closure exits
/// so cargo's test-thread reuse can't leak the cell across rows.
fn run_with_rig<F, T>(rig: &mut StdioRig, body: F) -> T
where
    F: FnOnce() -> T,
{
    let stdio = rig.ambient.take().expect("rig.ambient consumed twice");
    install_ambient_stdio(stdio);
    let result = body();
    let _ = take_ambient_stdio();
    result
}

/// Drain any leftover ambient stdio before the row's body runs.
fn fresh_thread() {
    let _ = take_ambient_stdio();
}

/// Freeze a wat fixture file as a `FrozenWorld`.
fn freeze(fixture: &str) -> wat::freeze::FrozenWorld {
    startup_from_file(fixture).expect("startup succeeds")
}

/// Read every byte the WatReader has buffered, decode as UTF-8.
/// Drains until EOF.
fn drain_to_string(reader: &Arc<dyn WatReader>) -> String {
    let bytes = reader
        .read_all(wat::rust_caller_span!())
        .expect("read-all");
    String::from_utf8(bytes).expect("utf8")
}

// ─── A. Single-thread program — main calls println ─────────────────

#[test]
fn row_a_single_thread_println() {
    fresh_thread();
    let mut rig = build_rig();
    let world = freeze("tests/program/wat_arc170_slice_1f_gamma_orchestrator_row_a.wat");
    let stdout_capture = Arc::clone(&rig.stdout_capture);
    let _stdin_inject = Arc::clone(&rig.stdin_inject); // keep alive
    let _stderr_capture = Arc::clone(&rig.stderr_capture);

    let result = run_with_rig(&mut rig, || invoke_user_main(&world, Vec::new()));
    assert!(matches!(result, Ok(Value::Unit)), "got {:?}", result);

    // Drain the orchestrator's stdout captured by the test pipe.
    // The wat-side StdOutService writes line + newline via
    // IOWriter/writeln. The String "hello slice 1f-gamma" renders to
    // its EDN form `"hello slice 1f-gamma"` (quoted) per
    // value_to_edn_with.
    let captured = drain_to_string(&stdout_capture);
    assert_eq!(
        captured,
        "\"hello slice 1f-gamma\"\n",
        "s1f_rowA: orchestrator stdout golden"
    );
}

// ─── D. Scope-drop cascade — services exit cleanly on main return ──

#[test]
fn row_d_scope_drop_cascade() {
    fresh_thread();
    let mut rig = build_rig();
    // Main does nothing; orchestrator boots services, registers
    // thread-0, runs main (returns nil immediately), cleans up,
    // drops carrier → services see disconnected control-rx → exit.
    // join_service in the orchestrator must return Ok for all three;
    // any failure surfaces as Err from invoke_user_main.
    // Primary fixture (canonical main) via startup_beside.
    let world = startup_beside(file!()).expect("startup");
    let result = run_with_rig(&mut rig, || invoke_user_main(&world, Vec::new()));
    assert!(
        matches!(result, Ok(Value::Unit)),
        "scope-drop cascade should join all 3 services; got {:?}",
        result
    );
}

// ─── E. readln roundtrip — main reads parsed form from stdin ───────

#[test]
fn row_e_readln_roundtrip() {
    fresh_thread();
    let mut rig = build_rig();
    // Test thread writes an EDN form to the orchestrator's stdin
    // BEFORE invoking main. The orchestrator's StdInService passes
    // the raw line; the substrate parses + coerces to T per
    // readln's `-> :T` annotation (arc 170 slice 1f-ι contract).
    //
    // Test thread feeds the pipe BEFORE main runs, then runs main.
    // The pipe's kernel buffer is large enough to hold a small EDN
    // form without blocking on the write side, so a single write_all
    // up front is safe.
    let bytes = b"\"echoed string\"\n";
    let stdin_inject = Arc::clone(&rig.stdin_inject);
    stdin_inject
        .write_all(bytes, wat::rust_caller_span!())
        .expect("write to stdin pipe");

    let world = freeze("tests/program/wat_arc170_slice_1f_gamma_orchestrator_row_e.wat");
    let result = run_with_rig(&mut rig, || invoke_user_main(&world, Vec::new()));
    assert!(
        matches!(result, Ok(Value::Unit)),
        "readln roundtrip should land main nil; got {:?}",
        result
    );
    let _ = stdin_inject; // keep alive across the run
}
