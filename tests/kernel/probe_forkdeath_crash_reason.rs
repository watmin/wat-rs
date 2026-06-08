//! Arc 214 — fork-program-death ENABLER, FM-2-bis disconfirming PROBE (RED at HEAD).
//!
//! Decomposition step 1 of
//! `docs/arc/2026/05/214-concurrency-toolkit/DESIGN-FORK-PROGRAM-DEATH-HERMETIC-AS-PEER.md`:
//! a `:process` peer's crash REASON must reach the parent **through the peer API**
//! — the substrate raises it on the client's behalf at the pending `recv`/`close`
//! (the resolved Q1 decision: *a user cannot fuck up panics; the substrate panics
//! on their behalf*) — NOT via a fd-2-redirect harness trick.
//!
//! ## Why this is RED at HEAD
//!
//! F3 (`emit_structured_exit`, `spawn.rs:485`/`:511`) makes the child EMIT a
//! `#wat.kernel/ProcessPanics` envelope when its fn errors — but on the child's
//! **inherited fd 2**. So through the peer API the parent sees only:
//!   - `recv()` → a bare `RecvError` (channel disconnect), no cause; and
//!   - `wait()` → `Exited(1)`, no cause.
//! The reason is reachable ONLY by redirecting fd 2 — exactly what
//! `spawn_program_prime_process_runtime_error_emits_diagnostic` does to prove the
//! envelope is emitted. This probe deliberately uses **no redirect**: it asserts
//! the reason rides the channel-close error. The ENABLER (pipe the child's
//! Err-channel to the parent; drain on close; raise the reason) flips it GREEN,
//! at which point it graduates from `probe_` to the permanent test.
//!
//! ## Containment
//!
//! Forks a `:process` peer → must run in the kernel integration binary
//! (single-threaded at startup) under the setsid+timeout envelope. Marked
//! `#[ignore]`. Run via:
//!   `cargo test --test kernel probe_forkdeath_process_crash_reason -- --ignored --test-threads=1`
//! or `bash scripts/integration-run.sh`. NEVER via raw `cargo test --test test`.

use std::sync::Arc;

use wat::freeze::startup_from_source;
use wat::kernel::spawn::{ProcessPeerCell, PROCESS_PEER_TYPE_PATH};
use wat::load::InMemoryLoader;
use wat::rust_deps::marshal::{downcast_ref_opaque, rust_opaque_arc};
use wat::span::Span;

/// FM-2-bis: the crash reason must reach the parent through the peer API, with
/// NO fd-2 redirect. RED at HEAD (recv yields a bare disconnect); GREEN when the
/// enabler makes the `ProcessPanics` reason ride the channel-close.
#[test]
#[ignore = "FM-2-bis enabler probe (RED at HEAD): run via integration-run.sh or with --ignored --test-threads=1; never via raw cargo test --test test"]
fn probe_forkdeath_process_crash_reason_reaches_parent_via_api() {
    // A :process peer whose fn divides 100 by its input — x=0 type-checks but
    // is DivisionByZero at runtime (the same trigger the redirect-based
    // diagnostic test uses; here the cause must come back THROUGH THE API).
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

    let dummy_span = Span::unknown();
    let peer_val =
        wat::kernel::spawn::spawn_process_peer(boom_fn_arc, &world.symbols, &dummy_span)
            .expect("spawn_process_peer must succeed");

    let opaque_arc = rust_opaque_arc(
        &peer_val,
        PROCESS_PEER_TYPE_PATH,
        "probe:forkdeath_crash_reason",
        dummy_span.clone(),
    )
    .expect("peer_val must be Value::RustOpaque(Process')");
    let cell: &ProcessPeerCell = downcast_ref_opaque(
        &opaque_arc,
        PROCESS_PEER_TYPE_PATH,
        "probe:downcast:ProcessPeerBundle",
        dummy_span.clone(),
    )
    .expect("downcast to ProcessPeerCell must succeed");

    // Drive it: send "0" → decode ok → apply_function → DivisionByZero → the
    // child emits a #wat.kernel/ProcessPanics envelope on fd 2 and _exit(1).
    cell.with_ref("probe:send", |opt| {
        opt.as_ref()
            .expect("bundle must not be closed")
            .peer
            .send("0".to_string())
            .expect("peer.send must succeed")
    })
    .expect("with_ref(send) must not cross thread boundary");

    // The parent meets the dead child at recv. At HEAD this is a bare disconnect.
    let recv_result = cell
        .with_ref("probe:recv", |opt| {
            opt.as_ref().expect("bundle must not be closed").peer.recv()
        })
        .expect("with_ref(recv) must not cross thread boundary");

    assert!(
        recv_result.is_err(),
        "child div-by-zero must close the channel → recv must be Err; got {:?}",
        recv_result
    );

    let err_text = format!("{:?}", recv_result.unwrap_err());

    // ── THE FM-2-bis DISCONFIRMING ASSERTION (RED at HEAD) ──────────────────────
    // The peer API must surface the crash REASON — with NO fd-2 redirect. At HEAD
    // `err_text` is a bare RecvError (disconnect); the cause is stranded on the
    // child's inherited fd 2. When the enabler pipes the Err-channel to the parent
    // so the reason rides the channel-close, this assertion passes (GREEN).
    assert!(
        err_text.contains("ProcessPanics") || err_text.contains("DivisionByZero"),
        "FM-2-bis (enabler NOT built): the parent must read the crash reason \
         THROUGH the peer API — the substrate raises it on the client's behalf \
         (Q1 decision) — not via a fd-2-redirect harness trick. At HEAD recv \
         returns a bare disconnect with no cause; the ProcessPanics reason is \
         stranded on the child's inherited fd 2. recv error was: {}",
        err_text
    );

    // mora — reap the child on the pidfd wire (no sleep).
    let bundle = cell
        .with_mut("probe:reap", Span::unknown(), |opt| opt.take())
        .expect("with_mut(reap) must not cross thread boundary")
        .expect("bundle must still be present at reap time");
    bundle
        .peer
        .wait()
        .expect("peer.wait() must reap the child on the pidfd wire");
    drop(peer_val);
}
