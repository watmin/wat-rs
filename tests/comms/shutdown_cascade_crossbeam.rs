//! Arc 170 Slice B probe — shutdown cascade reaches blocked crossbeam recv.
//!
//! Mirrors the pre-Slice-B gap demonstrated in `/tmp/shutdown_gap_proof.rs`
//! (50-line standalone Rust: SIGTERM fires but blocked crossbeam recv does
//! NOT wake). After Slice B, the same scenario MUST wake within 100ms.
//!
//! # Design
//!
//! Uses `run_in_fork` for child-process isolation so the SIGTERM raised
//! during the probe does not kill the cargo test binary. The forked child:
//!
//! 1. Initialises the substrate shutdown infrastructure (`init_shutdown_signal`).
//! 2. Installs the substrate signal handlers (`install_substrate_signal_handlers`).
//! 3. Creates a crossbeam channel (no sender held — sender stays alive until
//!    after the recv thread has started and signalled ready).
//! 4. Uses a rendezvous (bounded-0) channel to lock-step: recv-thread sends
//!    `()` right before calling `typed_recv`; main thread receives `()`,
//!    then raises SIGTERM.
//! 5. Joins the recv-thread. If Slice B is wired correctly, the thread wakes
//!    with `RecvOutcome::Shutdown`. The elapsed time is asserted < 100ms.
//! 6. Child exits 0. Parent (`run_in_fork`) asserts exit 0 — probe PASSES.
//!
//! If shutdown cascade does NOT work:
//! - The recv-thread never wakes → join blocks → child hangs → `waitpid`
//!   in `run_in_fork` hangs → cargo test runner reports timeout. The test
//!   is deliberately structured to hang (not silently pass) on regression.
//!
//! # Lock-step discipline
//!
//! No `thread::sleep` or `recv_timeout` anywhere. Synchronisation via:
//! - Rendezvous channel: recv-thread signals readiness before blocking.
//! - `thread::JoinHandle::join`: blocks until recv-thread exits (which it
//!   does only when `typed_recv` returns).
//!
//! `std::time::Instant` is used ONLY to MEASURE elapsed time after join
//! returns (it is not used as a timeout — the join IS the blocking point).
//!
//! # ZERO-MUTEX compliance
//!
//! No `Mutex`, `RwLock`, or `CondVar` in this file. Synchronisation is via
//! crossbeam bounded(0) rendezvous + `thread::join`.

use std::sync::Arc;
use std::time::Instant;
use wat::runtime::Value;
use wat::span::Span;
use wat::typed_channel::{ReceiverInner, RecvOutcome};

/// Row I — blocked crossbeam recv wakes on SIGTERM within 100ms.
///
/// Runs inside a forked child (process isolation). See module doc for
/// full design.
#[test]
fn probe_shutdown_cascade_wakes_crossbeam_recv() {
    wat::fork::run_in_fork(|| {
        // ── Step 1: initialise substrate shutdown infrastructure ───────────
        // init_shutdown_signal() creates SHUTDOWN_RX + SHUTDOWN_TX_PTR +
        // SHUTDOWN_WAKE_WRITE_FD and spawns the shutdown-worker thread.
        // This is safe to call directly in the forked child.
        wat::runtime::init_shutdown_signal();

        // ── Step 2: install substrate signal handlers ──────────────────────
        // install_substrate_signal_handlers() wires SIGTERM → substrate_on_stop_signal,
        // which (after Slice B) writes to SHUTDOWN_WAKE_WRITE_FD in addition
        // to setting KERNEL_STOPPED.
        wat::fork::install_substrate_signal_handlers();

        // ── Step 3: create a crossbeam channel ────────────────────────────
        // The Sender is kept alive in this scope (data will never flow — we
        // want the recv to block until shutdown wakes it).
        let (tx, rx) = crossbeam_channel::unbounded::<Value>();

        // Wrap in ReceiverInner::Crossbeam for typed_recv dispatch.
        let rx_inner = Arc::new(ReceiverInner::Crossbeam(rx));

        // ── Step 4: rendezvous channel (lock-step ready signal) ────────────
        // bounded(0) = rendezvous: recv-thread send blocks until this thread
        // receives. Guarantees recv-thread is AT the typed_recv call point
        // before we raise SIGTERM.
        let (ready_tx, ready_rx) = crossbeam_channel::bounded::<()>(0);

        // ── Step 5: spawn recv-thread ──────────────────────────────────────
        let rx_inner_clone = rx_inner.clone();
        let recv_thread = std::thread::Builder::new()
            .name("probe-recv".into())
            .spawn(move || {
                // Signal: about to call typed_recv. The rendezvous blocks
                // until the main thread is ready to receive (lock-step).
                ready_tx.send(()).expect("ready signal send");

                // Now call typed_recv. This blocks on the crossbeam recv,
                // multiplexed against SHUTDOWN_RX (Slice B wiring).
                let outcome = wat::typed_channel::typed_recv(
                    rx_inner_clone.as_ref(),
                    None,
                    Span::unknown(),
                );
                outcome
            })
            .expect("thread spawn succeeds");

        // ── Step 6: wait for recv-thread to signal readiness ──────────────
        // Rendezvous: this recv unblocks the recv-thread's send, which
        // means the recv-thread proceeds to call typed_recv immediately.
        ready_rx.recv().expect("ready signal recv");

        // Measure: start the clock AFTER recv-thread is ready.
        let t0 = Instant::now();

        // ── Step 7: raise SIGTERM ──────────────────────────────────────────
        // SIGTERM delivered to this process:
        // → substrate_on_stop_signal fires
        //   → KERNEL_STOPPED.store(true)
        //   → libc::write(SHUTDOWN_WAKE_WRITE_FD, b'!')
        //     → shutdown-worker thread wakes from read()
        //       → trigger_shutdown() drops SHUTDOWN_TX
        //         → SHUTDOWN_RX disconnects
        //           → crossbeam::select! in typed_recv fires Shutdown arm
        //             → RecvOutcome::Shutdown returned to recv-thread
        //               → recv-thread exits
        let raise_result = unsafe { libc::raise(libc::SIGTERM) };
        assert_eq!(raise_result, 0, "libc::raise(SIGTERM) must succeed");

        // ── Step 8: join recv-thread ───────────────────────────────────────
        // Blocks until typed_recv returns. If shutdown cascade works, this
        // returns quickly (< 100ms). If it hangs, the child hangs → parent's
        // waitpid blocks → cargo test reports timeout.
        let outcome = recv_thread.join().expect("recv-thread panicked");
        let elapsed = t0.elapsed();

        // ── Assertions ────────────────────────────────────────────────────
        assert!(
            matches!(outcome, RecvOutcome::Shutdown),
            "expected RecvOutcome::Shutdown after SIGTERM; got {:?}",
            outcome
        );
        assert!(
            elapsed.as_millis() < 100,
            "shutdown cascade should reach blocked recv within 100ms; elapsed: {:?}",
            elapsed
        );

        // Keep sender alive until after the recv-thread joined — ensures the
        // channel stays alive (disconnect comes from SHUTDOWN_RX, not tx drop).
        drop(tx);
    });
}
