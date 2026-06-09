//! Arc 214 Slice 4 Stone 4.5 — `spawn-program' :process` tier integration probe.
//!
//! Verifies that `spawn_process_peer` (the `:process`-tier implementation of
//! `spawn-program'`) produces a `Value::RustOpaque(PROCESS_PEER_TYPE_PATH)`
//! wrapping `Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>>`, and that the
//! parent can send an EDN-encoded value, the child applies the fn (identity),
//! and the parent receives the EDN-encoded result.
//!
//! # Test shape
//!
//! 1. Build a WAT world with a simple echo fn via `startup_from_source`.
//! 2. Call `spawn_process_peer` directly to produce a `Value::RustOpaque(Process')`.
//! 3. Downcast to `Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>>`.
//! 4. `bundle.peer.send("42")` → `bundle.peer.recv()` must return `"42"` (identity).
//! 5. Take the bundle and reap the child on the wire via `Process::wait` (close +
//!    `Pidfd::wait_status`), then drop the peer value.
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

use wat::freeze::startup_from_source;
use wat::kernel::spawn::{PeerRecvError, ProcessPeerCell, PROCESS_PEER_TYPE_PATH};
use wat::load::InMemoryLoader;
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


// ─── Tests ────────────────────────────────────────────────────────────────────

/// Process-tier spawn-program' round-trip: identity fn echo.
///
/// The child process receives `"42"` (EDN-encoded i64 42), applies the echo fn
/// (identity), encodes the result back to `"42"`, and sends it to the parent.
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
    // ── Step 1: build WAT world with an echo fn ────────────────────────────
    let world = startup_from_source(
        "(:wat::core::defn :my::echo [input <- :wat::core::i64] -> :wat::core::i64 input)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup_from_source for echo fn must succeed");

    let echo_fn_arc = world
        .symbols
        .get(":my::echo")
        .expect(":my::echo must be in symbol table after defn")
        .clone();

    // ── Step 2: spawn process-tier peer ───────────────────────────────────
    let dummy_span = Span::unknown();
    let peer_val =
        wat::kernel::spawn::spawn_process_peer(echo_fn_arc, &world.symbols, &dummy_span)
            .expect("spawn_process_peer must succeed");

    // ── Step 3: downcast to ProcessPeerCell ───────────────────────────────
    // Stone 4.6a-ii: payload is Option-wrapped so close' can take() it.
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

    // ── Step 4: send "42" → recv the echo result ───────────────────────────
    // Wire format: EDN-encoded string. The child decodes "42" → Value::i64(42),
    // applies identity, re-encodes → "42", sends back.
    peer_send(cell, "42");

    let got_str = peer_recv(cell).expect("peer.recv() must return echo result");

    // EDN encoding of i64(42) is "42".
    assert_eq!(
        got_str.trim(),
        "42",
        "identity fn echo must return \"42\" for input \"42\"; got {:?}",
        got_str
    );

    // ── Step 5: reap the child on the WIRE ────────────────────────────────
    reap_child_on_wire(cell);
    drop(peer_val);
}

/// A pure WAT fn (no non-portable Rust captures) must pass the `:process`
/// sandbox walker and round-trip correctly through the child apply-loop.
///
/// This test proves the affirmative case: `spawn_process_peer` accepts a pure
/// WAT fn and the child correctly applies it (double fn: 21 → 42).  The
/// rejection path (NonPortableCapture) is covered by the `closure_extract`
/// unit tests.
///
/// Marked `#[ignore]` — run with `--test-threads=1` alongside the echo probe
/// (two parallel forks from a multi-threaded parent cause the FM 7-ter hazard).
#[test]
#[ignore = "process-tier probe: run via integration-run.sh or with --ignored --test-threads=1; never via raw cargo test --test test"]
fn spawn_program_prime_process_sandbox_pure_fn_accepted() {
    // A pure WAT fn (no Rust captures) must be accepted by the sandbox walker.
    let world = startup_from_source(
        "(:wat::core::defn :my::double [x <- :wat::core::i64] -> :wat::core::i64 \
         (:wat::core::i64::* x 2))",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup_from_source for double fn must succeed");

    let double_fn_arc = world
        .symbols
        .get(":my::double")
        .expect(":my::double must be in symbol table")
        .clone();

    let dummy_span = Span::unknown();
    let peer_val =
        wat::kernel::spawn::spawn_process_peer(double_fn_arc.clone(), &world.symbols, &dummy_span)
            .expect("pure WAT fn must pass sandbox walker — spawn_process_peer must succeed");

    // Verify: must be RustOpaque(Process').
    let opaque_arc = rust_opaque_arc(
        &peer_val,
        PROCESS_PEER_TYPE_PATH,
        "test:spawn_program_prime_process_sandbox_pure_fn_accepted",
        dummy_span,
    )
    .expect("peer_val must be Value::RustOpaque(Process')");

    // Quick echo test for the double fn: send "21" → expect "42".
    // Stone 4.6a-ii: payload is Option-wrapped so close' can take() it.
    let cell: &ProcessPeerCell = downcast_ref_opaque(
        &opaque_arc,
        PROCESS_PEER_TYPE_PATH,
        "test:downcast:ProcessPeerBundle",
        Span::unknown(),
    )
    .expect("downcast to ProcessPeerCell must succeed");

    peer_send(cell, "21");
    let got = peer_recv(cell).expect("recv must return doubled value");

    assert_eq!(
        got.trim(),
        "42",
        "double fn must return 42 for input 21; got {:?}",
        got
    );

    // mora — reap the child on the WIRE.
    reap_child_on_wire(cell);
    drop(peer_val);
}

/// KR-1 regression: `:process` tier must resolve user-defined helpers.
///
/// Before the KR-1 fix (sym.clone() pre-fork), the child apply-loop used
/// `SymbolTable::new()` — an empty registry. Any fn that called a user-defined
/// helper (e.g. `:my::helper`) would fail with `UnknownFunction` in the child
/// and `_exit(1)`, while the same fn worked fine under `:thread`.
///
/// This test proves the cloned sym survives the fork: the program fn is
/// `:my::wrapper`, which internally calls `:my::helper` (a multiply-by-3 fn).
/// The child must resolve `:my::helper` from the cloned sym to compute 21 * 3 = 63.
///
/// Run via:
///   cargo test --test kernel spawn_program_prime_process_helper_round_trip -- --ignored
#[test]
#[ignore = "KR-1 regression probe: run via integration-run.sh or with --ignored --test-threads=1"]
fn spawn_program_prime_process_helper_round_trip() {
    // Build a world with two fns: :my::helper (triple) + :my::wrapper (calls helper).
    let world = startup_from_source(
        "(:wat::core::defn :my::helper [x <- :wat::core::i64] -> :wat::core::i64 \
         (:wat::core::i64::* x 3)) \
         (:wat::core::defn :my::wrapper [x <- :wat::core::i64] -> :wat::core::i64 \
         (:my::helper x))",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup_from_source for helper+wrapper fns must succeed");

    let wrapper_fn_arc = world
        .symbols
        .get(":my::wrapper")
        .expect(":my::wrapper must be in symbol table")
        .clone();

    // Spawn a :process peer with the wrapper fn.
    // KR-1: spawn_process_peer must clone sym before fork so the child can
    // resolve :my::helper. Without the fix the child exits with _exit(1).
    let dummy_span = Span::unknown();
    let peer_val =
        wat::kernel::spawn::spawn_process_peer(wrapper_fn_arc, &world.symbols, &dummy_span)
            .expect("spawn_process_peer must succeed (KR-1 sym clone)");

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

    // Send "21" (EDN i64 21) → child calls :my::wrapper(21) → :my::helper(21) → 63.
    peer_send(cell, "21");

    let got_str = peer_recv(cell).expect("peer.recv() must return helper result (KR-1 sym clone proof)");

    assert_eq!(
        got_str.trim(),
        "63",
        "wrapper(:my::helper x * 3) must return 63 for input 21; got {:?} \
         (KR-1: if this fails the child sym clone did not survive the fork)",
        got_str
    );

    // mora — reap the child on the WIRE.
    reap_child_on_wire(cell);
    drop(peer_val);
}

/// circumspicere F3 (Stone 6.w) regression: a `:process` peer child that hits
/// a malformed-input error must NOT die silently — it emits a structured
/// `#wat.kernel/ProcessPanics` envelope on fd 2 (the same shape every verbs.rs
/// fork child emits), so a dead peer names its cause instead of vanishing into a
/// bare `Exited(1)`.
///
/// Before the fix the child apply-loop did `Err(_) => libc::_exit(1)` on both
/// the malformed-input and runtime-error arms — a silent swallow (dark class).
///
/// Capture mechanism (Stone 214 fork-death enabler): `spawn_process_peer` wires
/// the child's fd 2 onto a diagnostic Err-channel pipe the bundle owns (the child
/// `dup2`s it before the close-sweep). After the child dies (recv → Err), the
/// parent drains the reason THROUGH the peer API via `take_crash_reason` — no
/// fd-2-redirect harness trick; the process-tier read of the locked remote
/// Q-channel's Err-discriminant.
///
/// The malformed-input arm is the deterministic trigger here; the runtime-error
/// arm is exercised by `spawn_program_prime_process_runtime_error_emits_diagnostic`.
///
/// Run via:
///   cargo test --test kernel spawn_program_prime_process_error_emits_diagnostic -- --ignored
#[test]
#[ignore = "process-tier probe: run via integration-run.sh or with --ignored --test-threads=1; never via raw cargo test --test test"]
fn spawn_program_prime_process_error_emits_diagnostic() {
    // ── Step 1: build a world with an echo fn (the fn is irrelevant — the
    //            child dies at EDN-decode, before it ever applies the fn). ────
    let world = startup_from_source(
        "(:wat::core::defn :my::echo [input <- :wat::core::i64] -> :wat::core::i64 input)",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup_from_source for echo fn must succeed");

    let echo_fn_arc = world
        .symbols
        .get(":my::echo")
        .expect(":my::echo must be in symbol table")
        .clone();

    // ── Step 3: spawn the peer (its diagnostic Err-channel captures fd 2). ──
    let dummy_span = Span::unknown();
    let peer_val =
        wat::kernel::spawn::spawn_process_peer(echo_fn_arc, &world.symbols, &dummy_span)
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

    // ── Step 4: send malformed EDN → child fails to decode → sends crash reason → _exit(1). ─
    peer_send(cell, "((( not valid edn");

    // Stone 214 1b-ii-α: the crash reason arrives through the io_uring Err arm,
    // so bundle.recv() (peer_recv) returns Crashed(reason) instead of a generic
    // disconnect. Extract the reason directly from the error variant.
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
             check that err_tx.send() ran before _exit in the child error arm"
        ),
    };

    assert!(
        diagnostic.contains("#wat.kernel/ProcessPanics"),
        "dead :process peer child must surface a structured ProcessPanics envelope \
         through the Err arm (bundle.recv()), not vanish; reason was {:?}",
        diagnostic
    );
    assert!(
        diagnostic.contains("malformed EDN input"),
        "the reason must name the cause (malformed EDN input); reason was {:?}",
        diagnostic
    );

    // mora — reap the child on the WIRE.
    reap_child_on_wire(cell);
    drop(peer_val);
}

/// circumspicere F2 (Stone 6.w) — runtime-error arm coverage.
///
/// The `Err(runtime_err)` arm of `apply_function` in the child apply-loop must
/// also emit a structured `#wat.kernel/ProcessPanics` envelope on fd 2.  This
/// complements `spawn_program_prime_process_error_emits_diagnostic` (malformed
/// input) by exercising the path where EDN decode SUCCEEDS but the fn itself
/// returns a `RuntimeError` (division-by-zero when the divisor is 0).
///
/// Trigger: the fn is `(:wat::core::i64::/ 100 x)` — x=0 is valid i64 at
/// check-time but causes a division-by-zero RuntimeError at runtime.  Sending
/// `"0"` (valid EDN) passes the decode step; the child reaches `apply_function`
/// which returns `Err(RuntimeError::DivisionByZero)`.
///
/// Run via:
///   cargo test --test kernel spawn_program_prime_process_runtime_error_emits_diagnostic -- --ignored
#[test]
#[ignore = "process-tier probe: run via integration-run.sh or with --ignored --test-threads=1; never via raw cargo test --test test"]
fn spawn_program_prime_process_runtime_error_emits_diagnostic() {
    // ── Step 1: build a world with a division fn (triggers runtime error at x=0). ─
    let world = startup_from_source(
        "(:wat::core::defn :my::boom [x <- :wat::core::i64] -> :wat::core::i64 \
         (:wat::core::i64::/ 100 x))",
        None,
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup_from_source for boom fn must succeed");

    let boom_fn_arc = world
        .symbols
        .get(":my::boom")
        .expect(":my::boom must be in symbol table")
        .clone();

    // ── Step 3: spawn the peer (its diagnostic Err-channel captures fd 2). ──
    let dummy_span = Span::unknown();
    let peer_val =
        wat::kernel::spawn::spawn_process_peer(boom_fn_arc, &world.symbols, &dummy_span)
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

    // ── Step 4: send "0" (valid EDN) → decode succeeds → apply_function returns
    //           Err(RuntimeError::DivisionByZero) → child sends crash reason → _exit(1). ───
    peer_send(cell, "0");

    // Stone 214 1b-ii-α: the crash reason arrives through the io_uring Err arm,
    // so bundle.recv() (peer_recv) returns Crashed(reason) carrying the cause.
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
             check that err_tx.send() ran before _exit in the child runtime-error arm"
        ),
    };

    assert!(
        diagnostic.contains("#wat.kernel/ProcessPanics"),
        "dead :process peer child must surface a structured ProcessPanics envelope \
         through the Err arm (bundle.recv()) for runtime errors; reason was {:?}",
        diagnostic
    );
    assert!(
        diagnostic.contains("DivisionByZero"),
        "the reason must name the cause (#wat.kernel/DivisionByZero, the structured \
         EDN tag — the runtime-error arm emits the serialized RuntimeError, not Display text); \
         reason was {:?}",
        diagnostic
    );

    // mora — reap the child on the WIRE.
    reap_child_on_wire(cell);
    drop(peer_val);
}
