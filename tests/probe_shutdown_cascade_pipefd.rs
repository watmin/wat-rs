//! Arc 170 Phase 2 probe — shutdown cascade reaches blocked PipeFd recv.
//!
//! Mirrors `probe_shutdown_cascade_crossbeam.rs` but uses a tier-2
//! (pipe-fd-backed) Receiver instead of a crossbeam Receiver.
//!
//! # Design
//!
//! Uses `run_in_fork` for child-process isolation so the SIGTERM raised
//! during the probe does not kill the cargo test binary. The forked child:
//!
//! 1. Initialises the substrate shutdown infrastructure
//!    (`init_shutdown_signal`). Phase 2 creates the broadcast pipe here.
//! 2. Installs the substrate signal handlers
//!    (`install_substrate_signal_handlers`).
//! 3. Creates a raw OS pipe pair via `libc::pipe`. The write-end is held
//!    alive in scope (so the recv sees no EOF from writer-close — only
//!    the shutdown broadcast can wake it).
//! 4. Wraps the read-end in a `PipeReader` → `ReceiverInner::PipeFd`.
//! 5. Uses a rendezvous (bounded-0) crossbeam channel to lock-step:
//!    recv-thread sends `()` right before calling `typed_recv`; main
//!    thread receives `()`, then raises SIGTERM.
//! 6. Joins the recv-thread. If Phase 2 is wired correctly, the thread
//!    wakes with `RecvOutcome::Shutdown` within 100ms.
//! 7. Child exits 0. Parent (`run_in_fork`) asserts exit 0 — probe PASSES.
//!
//! If shutdown cascade does NOT work:
//! - The recv-thread never wakes → join blocks → child hangs →
//!   `waitpid` in `run_in_fork` hangs → cargo test reporter reports
//!   timeout. The test is deliberately structured to hang (not silently
//!   pass) on regression.
//!
//! # Lock-step discipline
//!
//! No `thread::sleep` or `recv_timeout` anywhere. Synchronisation via:
//! - Rendezvous channel: recv-thread signals readiness before blocking.
//! - `thread::JoinHandle::join`: blocks until recv-thread exits.
//!
//! `std::time::Instant` is used ONLY to MEASURE elapsed time after join
//! returns (not as a timeout — the join IS the blocking point).
//!
//! # ZERO-MUTEX compliance
//!
//! No `Mutex`, `RwLock`, or `CondVar` in this file. Synchronisation via
//! crossbeam bounded(0) rendezvous + `thread::join`.

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use std::time::Instant;
use wat::io::{PipeReader, WatReader};
use wat::span::Span;
use wat::typed_channel::{ReceiverInner, RecvOutcome};

/// Arc 170 Phase 2 — blocked PipeFd recv wakes on SIGTERM within 100ms.
///
/// Runs inside a forked child (process isolation). See module doc for
/// full design.
#[test]
fn probe_shutdown_cascade_pipefd_wakes_pipe_recv() {
    wat::fork::run_in_fork(|| {
        // ── Step 1: initialise substrate shutdown infrastructure ───────────
        // init_shutdown_signal() creates SHUTDOWN_RX + SHUTDOWN_TX_PTR +
        // SHUTDOWN_WAKE_WRITE_FD + SHUTDOWN_BROADCAST_READ_FD (Phase 2)
        // and spawns the shutdown-worker thread.
        wat::runtime::init_shutdown_signal();

        // ── Step 2: install substrate signal handlers ──────────────────────
        // install_substrate_signal_handlers() wires SIGTERM → substrate_on_stop_signal.
        wat::fork::install_substrate_signal_handlers();

        // ── Step 3: create a data pipe pair ───────────────────────────────
        // The write-end is held alive in this scope. The recv should block
        // on this pipe until the shutdown broadcast wakes it — NOT because
        // the write-end was closed (which would be EOF / Disconnected, not
        // Shutdown).
        let mut pipe_fds = [0_i32; 2];
        let ret = unsafe { libc::pipe(pipe_fds.as_mut_ptr()) };
        assert_eq!(ret, 0, "libc::pipe must succeed for data pipe");
        let data_read_fd = unsafe { OwnedFd::from_raw_fd(pipe_fds[0]) };
        let data_write_fd = unsafe { OwnedFd::from_raw_fd(pipe_fds[1]) };

        // ── Step 4: wrap read-end in PipeFd Receiver ──────────────────────
        let reader: Arc<dyn WatReader> =
            Arc::new(PipeReader::from_owned_fd(data_read_fd));
        let rx_inner = Arc::new(ReceiverInner::PipeFd(reader));

        // ── Step 5: rendezvous channel (lock-step ready signal) ────────────
        // bounded(0) = rendezvous: recv-thread send blocks until this thread
        // receives. Guarantees recv-thread is AT the typed_recv call point
        // before we raise SIGTERM.
        let (ready_tx, ready_rx) = crossbeam_channel::bounded::<()>(0);

        // ── Step 6: spawn recv-thread ──────────────────────────────────────
        let rx_inner_clone = rx_inner.clone();
        let recv_thread = std::thread::Builder::new()
            .name("probe-pipefd-recv".into())
            .spawn(move || {
                // Signal: about to call typed_recv. The rendezvous blocks
                // until the main thread is ready to receive (lock-step).
                ready_tx.send(()).expect("ready signal send");

                // Now call typed_recv on the PipeFd receiver. This blocks
                // on poll(pipe_fd, broadcast_fd) — Phase 2 wiring.
                let outcome = wat::typed_channel::typed_recv(
                    rx_inner_clone.as_ref(),
                    None,
                    Span::unknown(),
                );
                outcome
            })
            .expect("recv-thread spawn succeeds");

        // ── Step 7: wait for recv-thread to signal readiness ──────────────
        // Rendezvous: this recv unblocks the recv-thread's send, which
        // means the recv-thread proceeds to call typed_recv immediately.
        ready_rx.recv().expect("ready signal recv");

        // Measure: start the clock AFTER recv-thread is ready.
        let t0 = Instant::now();

        // ── Step 8: raise SIGTERM ──────────────────────────────────────────
        // SIGTERM → substrate_on_stop_signal fires
        //   → KERNEL_STOPPED.store(true)
        //   → libc::write(SHUTDOWN_WAKE_WRITE_FD, b'!')
        //     → shutdown-worker thread wakes from poll()
        //       → trigger_shutdown() drops SHUTDOWN_TX
        //         → libc::close(broadcast_w_fd)  [Phase 2]
        //           → SHUTDOWN_BROADCAST_READ_FD fd sees POLLHUP
        //             → poll() in typed_recv PipeFd arm fires broadcast arm
        //               → RecvOutcome::Shutdown returned to recv-thread
        //                 → recv-thread exits
        let raise_result = unsafe { libc::raise(libc::SIGTERM) };
        assert_eq!(raise_result, 0, "libc::raise(SIGTERM) must succeed");

        // ── Step 9: join recv-thread ───────────────────────────────────────
        // Blocks until typed_recv returns. If Phase 2 works, this returns
        // quickly (< 100ms). If it hangs, the child hangs → waitpid blocks
        // → cargo test reports timeout.
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
            "shutdown cascade should reach blocked PipeFd recv within 100ms; elapsed: {:?}",
            elapsed
        );

        // Keep write-end alive until after recv-thread joined — ensures
        // the pipe stays open (disconnect comes from broadcast, not data_write_fd drop).
        drop(data_write_fd);
    });
}
