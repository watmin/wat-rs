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
//! fork-in-multithreaded-parent class that the per-test-process + `setsid`
//! containment pattern prevents. By placing the test here, it runs in the
//! comms integration-test binary (single-threaded at startup) under the
//! setsid+timeout envelope provided by `integration-run.sh`.
//!
//! Marked `#[ignore]` — run via:
//!   `bash scripts/integration-run.sh`
//! or directly:
//!   `cargo test --test kernel spawn_program_prime_process -- --ignored`

use std::sync::Arc;

use wat::freeze::{startup_bare, startup_beside, FrozenWorld};
use wat::kernel::spawn::{PeerRecvError, ProcessPeerCell, PROCESS_PEER_TYPE_PATH};
use wat::rust_deps::marshal::{downcast_ref_opaque, rust_opaque_arc};

/// A stdlib-loaded world for the direct `spawn_process_peer` calls below.
///
/// These probes used a bare `SymbolTable::new()`. That was sufficient when they
/// were written, but arc 209 C0b.3b-c gave `spawn_process_peer` an owner-side
/// post-spawn hook which CONSTRUCTS `(:wat::spawn::ProcessLaunch' pid)` — a wat
/// aggregate ctor that only exists once the stdlib is registered. Against an
/// empty table that ctor is genuinely unknown, so every one of these probes died
/// on `UnknownFunction` at the hook, reporting a missing name when the real gap
/// was a missing UNIVERSE. Their sibling process probes never saw it because they
/// go through `startup_from_source`/`startup_beside`, which load the stdlib.
///
/// `startup_bare()` is the sanctioned stdlib-only world (same helper
/// `bootstrap_wat_vm_process` uses); its `symbols()` is the table the hook needs.
fn stdlib_world() -> FrozenWorld {
    startup_bare().expect("startup_bare must succeed")
}

// ─── Shared test helpers ──────────────────────────────────────────────────────

/// Reap the child on the pidfd wire (the load-bearing mora).
///
/// Takes the `ProcessPeerBundle` out of the cell's `Option`, drops the
/// channels (giving the child EOF on its input), then blocks on the pidfd
/// via `Process::wait` (`waitid` — reaps the zombie atomically).
/// No sleep — a sleep is both a race and a leak (`Pidfd::Drop` only closes
/// the fd, it never reaps).
fn reap_child_on_wire(cell: &ProcessPeerCell) {
    let selectable = cell
        .with_mut("test:reap", wat::rust_caller_span!(), |opt| opt.take())
        .expect("with_mut(reap) must not cross thread boundary")
        .expect("bundle must still be present at reap time");
    match selectable {
        wat::kernel::spawn::ProcessSelectable::Spawned(bundle) => {
            bundle
                .peer
                .wait()
                .expect("peer.wait() must reap the child on the pidfd wire");
        }
        wat::kernel::spawn::ProcessSelectable::Timer(_) => {
            panic!("reap_child_on_wire: expected Spawned, got Timer");
        }
    }
}

/// Send an EDN-encoded string to the child via the peer channel.
fn peer_send(cell: &ProcessPeerCell, input: &str) {
    cell.with_ref("test:send", |opt_bundle| {
        match opt_bundle.as_ref().expect("bundle must not be closed") {
            wat::kernel::spawn::ProcessSelectable::Spawned(bundle) => {
                bundle
                    .peer
                    .send(input.to_string())
                    .expect("peer.send must succeed")
            }
            wat::kernel::spawn::ProcessSelectable::Timer(_) => {
                panic!("peer_send: expected Spawned, got Timer");
            }
        }
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
        match opt_bundle.as_ref().expect("bundle must not be closed") {
            wat::kernel::spawn::ProcessSelectable::Spawned(bundle) => bundle.recv(),
            wat::kernel::spawn::ProcessSelectable::Timer(_) => {
                panic!("peer_recv: expected Spawned, got Timer");
            }
        }
    })
    .expect("with_ref(recv) must not cross thread boundary")
}

/// Build forms from a co-located `.wat` fixture (never inlined). The fixture must define
/// `:user::main`. Returns `Vec<WatAST>` ready for `spawn_process_peer`.
fn forms_from_file(path: &str) -> Vec<wat::ast::WatAST> {
    let src = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("wat fixture {path:?} must exist (run from crate root): {e}"));
    wat::parser::parse_all_with_file(&src, path).expect("test forms must parse")
}

/// Build a no-op post-spawn-fn: `fn [_l <- ProcessLaunch] -> nil nil`.
/// Used to satisfy the `post_spawn_fn` arg when tests don't need the hook.
fn noop_process_post_spawn_fn() -> Arc<wat::Function> {
    let world = startup_beside(file!())
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
// body for all round-trip tests. Co-located: tests/kernel/spawn_program_prime_process_echo.wat.
const ECHO_PLUS_1_SERVER_WAT: &str = "tests/kernel/spawn_program_prime_process_echo.wat";

/// `spawn_process_peer`'s `env_fn` arg — a wat EXPRESSION (not a program) the child
/// re-parses to build its `ProgramEnv`. Every test here uses the trivial empty env.
/// Co-located: tests/kernel/spawn_program_prime_process_empty_env.wat.
fn empty_env_expr() -> String {
    std::fs::read_to_string("tests/kernel/spawn_program_prime_process_empty_env.wat")
        .expect("empty-env fixture must exist (run from crate root)")
}

// ─── Division-by-zero crash server ───────────────────────────────────────────
//
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
fn spawn_program_prime_process_echo_round_trip() {
    let forms = forms_from_file(ECHO_PLUS_1_SERVER_WAT);
    let dummy_span = wat::rust_caller_span!();
    let world = stdlib_world();
    let sym = world.symbols();
    let noop_psf = noop_process_post_spawn_fn();

    let peer_val =
        wat::kernel::spawn::spawn_process_peer(forms, noop_psf, empty_env_expr(), wat::edn_shim::DEFAULT_MAX_FRAME_BYTES, None, sym, &dummy_span)
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
fn spawn_program_prime_process_sandbox_pure_fn_accepted() {
    let forms = forms_from_file(ECHO_PLUS_1_SERVER_WAT);
    let dummy_span = wat::rust_caller_span!();
    let world = stdlib_world();
    let sym = world.symbols();
    let noop_psf = noop_process_post_spawn_fn();

    let peer_val =
        wat::kernel::spawn::spawn_process_peer(forms, noop_psf, empty_env_expr(), wat::edn_shim::DEFAULT_MAX_FRAME_BYTES, None, sym, &dummy_span)
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
fn spawn_program_prime_process_helper_round_trip() {
    // Forms-server with echo+1 logic (arc 214 β canonical shape).
    // The helper-round-trip concept from the fn era is now expressed as:
    // the forms-server IS self-contained — startup_from_forms handles all
    // symbol registration. Prove the server executes correctly end-to-end.
    let forms = forms_from_file(ECHO_PLUS_1_SERVER_WAT);
    let dummy_span = wat::rust_caller_span!();
    let world = stdlib_world();
    let sym = world.symbols();
    let noop_psf = noop_process_post_spawn_fn();

    let peer_val =
        wat::kernel::spawn::spawn_process_peer(forms, noop_psf, empty_env_expr(), wat::edn_shim::DEFAULT_MAX_FRAME_BYTES, None, sym, &dummy_span)
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

// ⊘ 2026-08-16 — TWO UNWRITTEN TESTS WERE DELETED FROM HERE, and a third with them
// (`tests/kernel/probe_arc214_alpha_crash_autoraise.rs`, the whole file).
//
//   spawn_program_prime_process_error_emits_diagnostic
//   spawn_program_prime_process_runtime_error_emits_diagnostic
//
// Both bodies were `unimplemented!()`. Their own `#[ignore]` reasons said so: "the body is
// unimplemented!() — running this out-of-band panics, it does not measure." They were
// placeholders wearing a test's clothes, and no arc closing could ever turn them green
// because there was nothing to turn green.
//
// The crash path they were written against is real and still open — child runtime error →
// panic → catch_unwind → finish_forked_child → emit_structured_exit → `#wat.kernel/ProcessPanics`
// on fd 2 → parent via bundle.recv() → Crashed(reason). The design content, the io_uring
// `1b-ii-α` gap it depends on, and what closing arc 214 must RULE on are preserved in:
//
//   docs/arc/2026/05/214-concurrency-toolkit/
//     NOTE-three-unwritten-crash-diagnostic-tests-were-deleted.md
//
// ⚠ Do NOT resurrect them from git. Write what the ruling calls for — the deleted file
// asserted, in its own header, that one of these two was "already green", and it had never
// run at all.
