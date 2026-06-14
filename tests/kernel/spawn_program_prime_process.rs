//! Arc 214 Slice 4 Stone 4.5 — `spawn-program' :process` tier integration probe.
//!
//! Arc 214 β migration: `spawn_process_peer` now takes a WAT PROGRAM (forms —
//! `Vec<WatAST>`) instead of a `Arc<Function>`. The child runs the program as a
//! `readln`/`println` server (forms-server model). Each test supplies its own
//! `:user::main` forms body via `parse_all_with_file`.
//!
//! # Test shape
//!
//! 1. Build forms from a WAT source string via `parse_all_with_file`.
//! 2. Call `spawn_process_peer` directly to produce a `Value::RustOpaque(Process')`.
//! 3. Downcast to `Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>>`.
//! 4. `bundle.send("42")` → `bundle.recv()` must return the expected EDN result.
//! 5. Reap the child on the wire via `Process::wait`.
//!
//! # Why this lives in the integration test binary
//!
//! `spawn_process_peer` uses `fork`/`clone3`. Fork inside a multi-threaded cargo
//! test binary inherits the thread pool's fd-table and locks — this is the
//! fork-in-multithreaded-parent class that the `run_in_fork` + `setsid`
//! containment pattern prevents. By placing the test here, it runs in the
//! comms integration-test binary (single-threaded at startup) under the
//! setsid+timeout envelope provided by `integration-run.sh`.
//!
//! Marked `#[ignore]` — run via:
//!   `bash scripts/integration-run.sh`
//! or directly:
//!   `cargo test --test kernel spawn_program_prime_process -- --ignored`

use std::sync::Arc;

use wat::kernel::spawn::{PeerRecvError, ProcessPeerCell, PROCESS_PEER_TYPE_PATH};
use wat::rust_deps::marshal::{downcast_ref_opaque, rust_opaque_arc};
use wat::span::Span;

// ─── Shared test helpers ──────────────────────────────────────────────────────

/// Reap the child on the pidfd wire (the load-bearing mora).
///
/// Takes the `ProcessPeerBundle` out of the cell's `Option`, drops the
/// channels (giving the child EOF on its input), then blocks on the pidfd
/// via `Process::wait` (`waitid` — reaps the zombie atomically).
/// No sleep — a sleep is both a race and a leak (`Pidfd::Drop` only closes
/// the fd, it never reaps).
fn reap_child_on_wire(cell: &ProcessPeerCell) {
    let bundle = cell
        .with_mut("test:reap", Span::unknown(), |opt| opt.take())
        .expect("with_mut(reap) must not cross thread boundary")
        .expect("bundle must still be present at reap time");
    bundle
        .peer
        .wait()
        .expect("peer.wait() must reap the child on the pidfd wire");
}

/// Send an EDN-encoded string to the child via the peer channel.
fn peer_send(cell: &ProcessPeerCell, input: &str) {
    cell.with_ref("test:send", |opt_bundle| {
        opt_bundle
            .as_ref()
            .expect("bundle must not be closed")
            .peer
            .send(input.to_string())
            .expect("peer.send must succeed")
    })
    .expect("with_ref(send) must not cross thread boundary");
}

/// Receive a result from the child via the bundle's Select (Ok + Err arms).
///
/// Stone 214 1b-ii-α: uses `bundle.recv()` (the 3-fd io_uring Select) instead
/// of `bundle.peer.recv()`. Returns `Ok(String)` on success, or
/// `Err(PeerRecvError::Crashed(reason))` when the child sent a crash reason,
/// or `Err(PeerRecvError::Disconnected)` on clean disconnect.
fn peer_recv(cell: &ProcessPeerCell) -> Result<String, PeerRecvError> {
    cell.with_ref("test:recv", |opt_bundle| {
        opt_bundle
            .as_ref()
            .expect("bundle must not be closed")
            .recv()
    })
    .expect("with_ref(recv) must not cross thread boundary")
}

/// Build forms from a WAT source string. The source must define `:user::main`.
/// Returns `Vec<WatAST>` ready for `spawn_process_peer`.
fn forms_from_src(src: &str) -> Vec<wat::ast::WatAST> {
    wat::parser::parse_all_with_file(src, "<test-forms-server>")
        .expect("test forms must parse")
}

/// Build a no-op post-spawn-fn: `fn [_l <- ProcessLaunch] -> nil nil`.
/// Used to satisfy the `post_spawn_fn` arg when tests don't need the hook.
fn noop_process_post_spawn_fn() -> Arc<wat::Function> {
    let world = wat::freeze::startup_from_source(
        "(:wat::core::defn :my::noop-post-spawn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)",
        None,
        Arc::new(wat::load::InMemoryLoader::new()),
    )
    .expect("startup for noop process post-spawn fn must succeed");
    world
        .symbols
        .get(":my::noop-post-spawn")
        .expect(":my::noop-post-spawn must be in the symbol table")
        .clone()
}

// ─── The proven echo+1 server body (arc 214 β canonical shape) ───────────────
//
// Reads one i64 from fd 0 (`readln -> :i64`), writes n+1 to fd 1 (`println`).
// Known-good under spawn-process (arc 112 slice 2b). Used as the base server
// body for all round-trip tests.
const ECHO_PLUS_1_SERVER: &str = r#"
    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:wat::core::let [n (:wat::kernel::readln -> :wat::core::i64)
                        _ (:wat::kernel::println (:wat::core::i64::+ n 1))]
        nil))
"#;

// ─── Division-by-zero crash server ───────────────────────────────────────────
//
// Reads one i64 from fd 0 (`readln -> :i64`), writes (100 / n) to fd 1.
// Sending n=0 triggers DivisionByZero in the child → crash reason via err channel.
const DIVISION_CRASH_SERVER: &str = r#"
    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:wat::core::let [n (:wat::kernel::readln -> :wat::core::i64)
                        _ (:wat::kernel::println (:wat::core::i64::/ 100 n))]
        nil))
"#;


// ─── Tests ────────────────────────────────────────────────────────────────────

/// Process-tier spawn-program' round-trip: echo+1 server.
///
/// The child process receives `"41"` (EDN-encoded i64 41), applies the echo+1
/// server logic (readln -> :i64, println (i64::+ n 1)), and sends back `"42"`.
///
/// Arc 214 β migration: spawn_process_peer now takes forms (Vec<WatAST>) instead
/// of Arc<Function>. The server body is the proven arc112 echo+1 shape.
///
/// Marked `#[ignore]` — run via `integration-run.sh` or `--ignored` flag.
/// MUST use `--test-threads=1` when running both process-tier tests together:
/// cargo runs parallel threads by default; two concurrent forks from a
/// multi-threaded parent create the FM 7-ter hazard. The `--test-threads=1`
/// flag serializes the two probes without changing the test structure.
/// NEVER run via raw `cargo test --test test` (deadlocks on the old stack).
#[test]
#[ignore = "process-tier probe: run via integration-run.sh or with --ignored --test-threads=1; never via raw cargo test --test test"]
fn spawn_program_prime_process_echo_round_trip() {
    let forms = forms_from_src(ECHO_PLUS_1_SERVER);
    let dummy_span = Span::unknown();
    let sym = wat::runtime::SymbolTable::new();
    let noop_psf = noop_process_post_spawn_fn();

    let peer_val =
        wat::kernel::spawn::spawn_process_peer(forms, noop_psf, "(:wat::program::EmptyEnv)".to_string(), &sym, &dummy_span)
            .expect("spawn_process_peer must succeed");

    let opaque_arc = rust_opaque_arc(
        &peer_val,
        PROCESS_PEER_TYPE_PATH,
        "test:spawn_program_prime_process_echo_round_trip",
        dummy_span.clone(),
    )
    .expect("peer_val must be Value::RustOpaque(Process')");

    let cell: &ProcessPeerCell = downcast_ref_opaque(
        &opaque_arc,
        PROCESS_PEER_TYPE_PATH,
        "test:downcast:ProcessPeerBundle",
        dummy_span.clone(),
    )
    .expect("downcast to ProcessPeerCell must succeed");

    // Send 41 → echo+1 server returns 42.
    peer_send(cell, "41");

    let got_str = peer_recv(cell).expect("peer.recv() must return echo+1 result");

    assert_eq!(
        got_str.trim(),
        "42",
        "echo+1 server must return \"42\" for input \"41\"; got {:?}",
        got_str
    );

    reap_child_on_wire(cell);
    drop(peer_val);
}

/// A pure WAT forms-server must round-trip correctly through the child.
///
/// This test proves the affirmative case: `spawn_process_peer` accepts a pure
/// WAT forms-server and the child correctly executes it (echo+1: 21 → 22).
///
/// Arc 214 β migration: spawn_process_peer now takes forms instead of Arc<Function>.
///
/// Marked `#[ignore]` — run with `--test-threads=1`.
#[test]
#[ignore = "process-tier probe: run via integration-run.sh or with --ignored --test-threads=1; never via raw cargo test --test test"]
fn spawn_program_prime_process_sandbox_pure_fn_accepted() {
    let forms = forms_from_src(ECHO_PLUS_1_SERVER);
    let dummy_span = Span::unknown();
    let sym = wat::runtime::SymbolTable::new();
    let noop_psf = noop_process_post_spawn_fn();

    let peer_val =
        wat::kernel::spawn::spawn_process_peer(forms, noop_psf, "(:wat::program::EmptyEnv)".to_string(), &sym, &dummy_span)
            .expect("pure WAT forms-server must spawn successfully");

    let opaque_arc = rust_opaque_arc(
        &peer_val,
        PROCESS_PEER_TYPE_PATH,
        "test:spawn_program_prime_process_sandbox_pure_fn_accepted",
        dummy_span.clone(),
    )
    .expect("peer_val must be Value::RustOpaque(Process')");

    let cell: &ProcessPeerCell = downcast_ref_opaque(
        &opaque_arc,
        PROCESS_PEER_TYPE_PATH,
        "test:downcast:ProcessPeerBundle",
        dummy_span.clone(),
    )
    .expect("downcast to ProcessPeerCell must succeed");

    // Send 21 → echo+1 server returns 22.
    peer_send(cell, "21");
    let got = peer_recv(cell).expect("recv must return echo+1 result");

    assert_eq!(
        got.trim(),
        "22",
        "echo+1 server must return 22 for input 21; got {:?}",
        got
    );

    reap_child_on_wire(cell);
    drop(peer_val);
}

/// KR-1 regression: `:process` tier forms-server must start cleanly.
///
/// Before the KR-1 fix (sym.clone() pre-fork), the child would use an empty
/// SymbolTable. With the forms-server model, the child runs startup_from_forms
/// which registers all symbols from the forms — KR-1 is not a concern for
/// forms-servers (they're self-contained). This test proves the forms-server
/// spawns and runs correctly end-to-end.
///
/// Run via:
///   cargo test --test kernel spawn_program_prime_process_helper_round_trip -- --ignored
#[test]
#[ignore = "KR-1 regression probe: run via integration-run.sh or with --ignored --test-threads=1"]
fn spawn_program_prime_process_helper_round_trip() {
    // Forms-server with echo+1 logic (arc 214 β canonical shape).
    // The helper-round-trip concept from the fn era is now expressed as:
    // the forms-server IS self-contained — startup_from_forms handles all
    // symbol registration. Prove the server executes correctly end-to-end.
    let forms = forms_from_src(ECHO_PLUS_1_SERVER);
    let dummy_span = Span::unknown();
    let sym = wat::runtime::SymbolTable::new();
    let noop_psf = noop_process_post_spawn_fn();

    let peer_val =
        wat::kernel::spawn::spawn_process_peer(forms, noop_psf, "(:wat::program::EmptyEnv)".to_string(), &sym, &dummy_span)
            .expect("spawn_process_peer must succeed (forms-server startup)");

    let opaque_arc = rust_opaque_arc(
        &peer_val,
        PROCESS_PEER_TYPE_PATH,
        "test:spawn_program_prime_process_helper_round_trip",
        dummy_span.clone(),
    )
    .expect("peer_val must be Value::RustOpaque(Process')");

    let cell: &ProcessPeerCell = downcast_ref_opaque(
        &opaque_arc,
        PROCESS_PEER_TYPE_PATH,
        "test:downcast:ProcessPeerBundle",
        dummy_span.clone(),
    )
    .expect("downcast to ProcessPeerCell must succeed");

    // Send 41 → echo+1 server returns 42.
    peer_send(cell, "41");

    let got_str = peer_recv(cell).expect("peer.recv() must return result (forms-server startup proof)");

    assert_eq!(
        got_str.trim(),
        "42",
        "forms-server echo+1 must return 42 for input 41; got {:?} \
         (arc 214 β: forms-server startup_from_forms is self-contained)",
        got_str
    );

    reap_child_on_wire(cell);
    drop(peer_val);
}

/// circumspicere F3 (Stone 6.w) regression: a `:process` peer forms-server child
/// that hits a runtime error must NOT die silently — it emits a structured
/// `#wat.kernel/ProcessPanics` envelope on fd 2 (the err channel).
///
/// Arc 214 β migration: the server is now a forms-server. When malformed EDN
/// is sent, `readln -> :i64` fails to parse the line → RuntimeError → panic →
/// catch_unwind → finish_forked_child → emit_structured_exit writes the
/// ProcessPanics envelope to fd 2 (the err channel). The parent reads it via
/// bundle.recv() → Crashed(reason).
///
/// Run via:
///   cargo test --test kernel spawn_program_prime_process_error_emits_diagnostic -- --ignored
#[test]
#[ignore = "process-tier probe: run via integration-run.sh or with --ignored --test-threads=1; never via raw cargo test --test test"]
fn spawn_program_prime_process_error_emits_diagnostic() {
    let forms = forms_from_src(ECHO_PLUS_1_SERVER);
    let dummy_span = Span::unknown();
    let sym = wat::runtime::SymbolTable::new();
    let noop_psf = noop_process_post_spawn_fn();

    let peer_val =
        wat::kernel::spawn::spawn_process_peer(forms, noop_psf, "(:wat::program::EmptyEnv)".to_string(), &sym, &dummy_span)
            .expect("spawn_process_peer must succeed");

    let opaque_arc = rust_opaque_arc(
        &peer_val,
        PROCESS_PEER_TYPE_PATH,
        "test:spawn_program_prime_process_error_emits_diagnostic",
        dummy_span.clone(),
    )
    .expect("peer_val must be Value::RustOpaque(Process')");
    let cell: &ProcessPeerCell = downcast_ref_opaque(
        &opaque_arc,
        PROCESS_PEER_TYPE_PATH,
        "test:downcast:ProcessPeerBundle",
        dummy_span.clone(),
    )
    .expect("downcast to ProcessPeerCell must succeed");

    // Send malformed EDN → child's readln -> :i64 fails to parse → crash.
    peer_send(cell, "((( not valid edn");

    // Stone 214 1b-ii-α: crash reason arrives through the io_uring Err arm.
    let recv_result = peer_recv(cell);
    let diagnostic = match recv_result {
        Ok(s) => panic!(
            "child must die on malformed input → bundle.recv() must return Err; got Ok({:?})",
            s
        ),
        Err(PeerRecvError::Crashed(reason)) => reason,
        Err(PeerRecvError::Disconnected) => panic!(
            "child died on malformed input but crash reason was NOT delivered through \
             the Err arm — got Disconnected instead of Crashed(reason); \
             check that emit_structured_exit runs before _exit in the child error arm"
        ),
    };

    assert!(
        diagnostic.contains("#wat.kernel/ProcessPanics"),
        "dead :process peer forms-server child must surface a structured ProcessPanics envelope \
         through the Err arm (bundle.recv()), not vanish; reason was {:?}",
        diagnostic
    );
    // Arc 214 β: the error now comes from readln failing to parse/coerce the malformed EDN.
    // The panic envelope carries the readln failure details.
    assert!(
        diagnostic.contains("EDN") || diagnostic.contains("parse") || diagnostic.contains("malformed"),
        "the reason must name the EDN/parse failure cause; reason was {:?}",
        diagnostic
    );

    reap_child_on_wire(cell);
    drop(peer_val);
}

/// circumspicere F2 (Stone 6.w) — runtime-error arm coverage.
///
/// The forms-server child that hits a DivisionByZero error must emit a structured
/// `#wat.kernel/ProcessPanics` envelope on fd 2. Uses a forms-server that reads
/// one i64 and writes (100 / n): sending n=0 triggers DivisionByZero.
///
/// Arc 214 β migration: spawn_process_peer now takes forms. The crash mechanism
/// is the same: panic → catch_unwind in run_user_main_in_child → finish_forked_child
/// → emit_structured_exit → fd 2 (err channel) → parent reads via bundle.recv().
///
/// Run via:
///   cargo test --test kernel spawn_program_prime_process_runtime_error_emits_diagnostic -- --ignored
#[test]
#[ignore = "process-tier probe: run via integration-run.sh or with --ignored --test-threads=1; never via raw cargo test --test test"]
fn spawn_program_prime_process_runtime_error_emits_diagnostic() {
    // Division server: reads i64, writes (100 / n). n=0 → DivisionByZero.
    let forms = forms_from_src(DIVISION_CRASH_SERVER);
    let dummy_span = Span::unknown();
    let sym = wat::runtime::SymbolTable::new();
    let noop_psf = noop_process_post_spawn_fn();

    let peer_val =
        wat::kernel::spawn::spawn_process_peer(forms, noop_psf, "(:wat::program::EmptyEnv)".to_string(), &sym, &dummy_span)
            .expect("spawn_process_peer must succeed");

    let opaque_arc = rust_opaque_arc(
        &peer_val,
        PROCESS_PEER_TYPE_PATH,
        "test:spawn_program_prime_process_runtime_error_emits_diagnostic",
        dummy_span.clone(),
    )
    .expect("peer_val must be Value::RustOpaque(Process')");
    let cell: &ProcessPeerCell = downcast_ref_opaque(
        &opaque_arc,
        PROCESS_PEER_TYPE_PATH,
        "test:downcast:ProcessPeerBundle",
        dummy_span.clone(),
    )
    .expect("downcast to ProcessPeerCell must succeed");

    // Send "0" (valid EDN) → decode succeeds → (100 / 0) → DivisionByZero → crash.
    peer_send(cell, "0");

    // Stone 214 1b-ii-α: crash reason arrives through the io_uring Err arm.
    let recv_result = peer_recv(cell);
    let diagnostic = match recv_result {
        Ok(s) => panic!(
            "child must die on division-by-zero → bundle.recv() must return Err; got Ok({:?})",
            s
        ),
        Err(PeerRecvError::Crashed(reason)) => reason,
        Err(PeerRecvError::Disconnected) => panic!(
            "child died on division-by-zero but crash reason was NOT delivered through \
             the Err arm — got Disconnected instead of Crashed(reason); \
             check that emit_structured_exit runs before _exit in the forms-server child panic arm"
        ),
    };

    assert!(
        diagnostic.contains("#wat.kernel/ProcessPanics"),
        "dead :process peer forms-server child must surface a structured ProcessPanics envelope \
         through the Err arm (bundle.recv()) for runtime errors; reason was {:?}",
        diagnostic
    );
    assert!(
        diagnostic.contains("DivisionByZero"),
        "the reason must name the cause (#wat.kernel/DivisionByZero — the structured \
         EDN tag from the forms-server panic); reason was {:?}",
        diagnostic
    );

    reap_child_on_wire(cell);
    drop(peer_val);
}
