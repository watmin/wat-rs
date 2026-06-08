//! Arc 214 Slice 4 Stone 4.5 — `spawn-program' :process` tier integration probe.
//!
//! Verifies that `spawn_process_peer` (the `:process`-tier implementation of
//! `spawn-program'`) produces a `Value::RustOpaque(PROCESS_PEER_TYPE_PATH)`
//! wrapping `Arc<ThreadOwnedCell<ProcessPeerBundle>>`, and that the parent can
//! send an EDN-encoded value, the child applies the fn (identity), and the parent
//! receives the EDN-encoded result.
//!
//! # Test shape
//!
//! 1. Build a WAT world with a simple echo fn via `startup_from_source`.
//! 2. Call `spawn_process_peer` directly to produce a `Value::RustOpaque(Process')`.
//! 3. Downcast to `Arc<ThreadOwnedCell<ProcessPeerBundle>>`.
//! 4. `bundle.peer.send("42")` → `bundle.peer.recv()` must return `"42"` (identity).
//! 5. Drop the peer value to close channels and signal the child to exit.
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
//!   `cargo test --test comms spawn_program_prime_process -- --ignored`

use std::sync::Arc;

use wat::freeze::startup_from_source;
use wat::kernel::spawn::{ProcessPeerBundle, PROCESS_PEER_TYPE_PATH};
use wat::load::InMemoryLoader;
use wat::rust_deps::custodia::ThreadOwnedCell;
use wat::rust_deps::marshal::{downcast_ref_opaque, rust_opaque_arc};
use wat::span::Span;

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

    // ── Step 3: downcast to ProcessPeerBundle ──────────────────────────────
    let opaque_arc = rust_opaque_arc(
        &peer_val,
        PROCESS_PEER_TYPE_PATH,
        "test:spawn_program_prime_process_echo_round_trip",
        dummy_span.clone(),
    )
    .expect("peer_val must be Value::RustOpaque(Process')");

    // Stone 4.6a-ii: payload is now Option-wrapped so close' can take() it.
    let cell: &Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>> = downcast_ref_opaque(
        &opaque_arc,
        PROCESS_PEER_TYPE_PATH,
        "test:downcast:ProcessPeerBundle",
        dummy_span.clone(),
    )
    .expect("downcast to Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>> must succeed");

    // ── Step 4: send "42" → recv the echo result ───────────────────────────
    // Wire format: EDN-encoded string. The child decodes "42" → Value::i64(42),
    // applies identity, re-encodes → "42", sends back.
    cell.with_ref("test:send", |opt_bundle| {
        opt_bundle
            .as_ref()
            .expect("bundle must not be closed")
            .peer
            .send("42".to_string())
            .expect("peer.send(\"42\") must succeed")
    })
    .expect("with_ref(send) must not cross thread boundary");

    let got_str = cell
        .with_ref("test:recv", |opt_bundle| {
            opt_bundle
                .as_ref()
                .expect("bundle must not be closed")
                .peer
                .recv()
                .expect("peer.recv() must return echo result")
        })
        .expect("with_ref(recv) must not cross thread boundary");

    // EDN encoding of i64(42) is "42".
    assert_eq!(
        got_str.trim(),
        "42",
        "identity fn echo must return \"42\" for input \"42\"; got {:?}",
        got_str
    );

    // ── Step 5: close channels → child exits ───────────────────────────────
    // Dropping peer_val closes the parent's input Sender to the child.
    // The child's apply-loop recv returns RecvError (EOF) → _exit(0).
    drop(peer_val);
    // Brief pause to allow the child process to reap.
    std::thread::sleep(std::time::Duration::from_millis(100));
}

/// Sandbox rejection: `:process` tier must reject a fn that captures a
/// non-portable value (a thread::Sender). The fn cannot cross the fork
/// address-space boundary safely.
///
/// Note: This test requires the sandbox walker (`closure_extract`) to detect
/// non-portable captures. If the fn is a pure WAT fn (no Rust captures), the
/// walker may not reject it via NonPortableCapture. For now, this is a
/// structural assertion — the rejection path in `spawn_process_peer` is
/// covered when the fn has captured non-portable Rust values.
///
/// The sandbox walker is already probed by the closure_extract unit tests;
/// this integration test documents the boundary behavior.
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
    let _opaque = rust_opaque_arc(
        &peer_val,
        PROCESS_PEER_TYPE_PATH,
        "test:spawn_program_prime_process_sandbox_pure_fn_accepted",
        dummy_span,
    )
    .expect("peer_val must be Value::RustOpaque(Process')");

    // Quick echo test for the double fn: send "21" → expect "42".
    // Stone 4.6a-ii: payload is now Option-wrapped so close' can take() it.
    let opaque_arc = _opaque;
    let cell: &Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>> = downcast_ref_opaque(
        &opaque_arc,
        PROCESS_PEER_TYPE_PATH,
        "test:downcast:ProcessPeerBundle",
        Span::unknown(),
    )
    .expect("downcast must succeed");

    cell.with_ref("test:send", |opt_bundle| {
        opt_bundle.as_ref().expect("bundle must not be closed").peer.send("21".to_string()).expect("send 21 must succeed")
    })
    .expect("with_ref(send)");

    let got = cell
        .with_ref("test:recv", |opt_bundle| {
            opt_bundle.as_ref().expect("bundle must not be closed").peer.recv().expect("recv must return doubled value")
        })
        .expect("with_ref(recv)");

    assert_eq!(
        got.trim(),
        "42",
        "double fn must return 42 for input 21; got {:?}",
        got
    );

    drop(peer_val);
    std::thread::sleep(std::time::Duration::from_millis(100));
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
///   cargo test --test comms spawn_program_prime_process_helper_round_trip -- --ignored
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

    let cell: &Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>> = downcast_ref_opaque(
        &opaque_arc,
        PROCESS_PEER_TYPE_PATH,
        "test:downcast:ProcessPeerBundle",
        dummy_span.clone(),
    )
    .expect("downcast to Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>> must succeed");

    // Send "21" (EDN i64 21) → child calls :my::wrapper(21) → :my::helper(21) → 63.
    cell.with_ref("test:send", |opt_bundle| {
        opt_bundle
            .as_ref()
            .expect("bundle must not be closed")
            .peer
            .send("21".to_string())
            .expect("peer.send(\"21\") must succeed")
    })
    .expect("with_ref(send) must not cross thread boundary");

    let got_str = cell
        .with_ref("test:recv", |opt_bundle| {
            opt_bundle
                .as_ref()
                .expect("bundle must not be closed")
                .peer
                .recv()
                .expect("peer.recv() must return helper result (KR-1 sym clone proof)")
        })
        .expect("with_ref(recv) must not cross thread boundary");

    assert_eq!(
        got_str.trim(),
        "63",
        "wrapper(:my::helper x * 3) must return 63 for input 21; got {:?} \
         (KR-1: if this fails the child sym clone did not survive the fork)",
        got_str
    );

    drop(peer_val);
    std::thread::sleep(std::time::Duration::from_millis(100));
}
