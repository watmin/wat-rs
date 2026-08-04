//! Arc 278 "cut the cap, PROVE the poll arm" — RED-gate probe.
//!
//! ## The mechanism under test
//!
//! `comms::process::Sender::send` (`src/comms/process.rs`) polls
//! `[fd → POLLOUT, SHUTDOWN_BROADCAST_READ_FD → POLLIN|POLLHUP]` before every
//! write attempt into a pipe. This poll arm has never been exercised by any
//! test in the tree — every existing comms probe either never fills the pipe
//! (so the write never actually blocks) or drives a service test that never
//! reaches this write loop at all. This probe fills a REAL OS pipe to kernel
//! capacity, blocks a `Sender::send` on it from a background thread, fires
//! the substrate shutdown broadcast, and proves the blocked send wakes with
//! `SendError::Shutdown` instead of hanging forever.
//!
//! ## Shape — copied from tests/channel/probe_arc170_writer_joins_lockstep.rs
//!
//! That probe solves the identical problem for `PipeWriter::write`; this one
//! reuses its shape for `comms::process::Sender::send`:
//!   - fill the pipe: toggle O_NONBLOCK on the write fd, write 4 KiB chunks
//!     until EAGAIN (the kernel buffer is then exactly full), restore
//!     blocking mode — mirrors `src/comms/process.rs`'s own
//!     non-blocking-probe-then-restore pattern (`Sender::try_send`) and the
//!     arc170 probe's fill step verbatim.
//!   - block on a BACKGROUND thread, never the test's main thread.
//!   - rendezvous on a bounded(0) channel — the writer signals readiness
//!     immediately before calling `send`.
//!   - collect the outcome through a BOUNDED `recv_timeout`, NEVER a raw
//!     `.join()` — so this probe itself can never hang even while the poll
//!     arm is broken (the deliberate-break half of the brief proves this
//!     rather than merely asserting it).
//!
//! ## Firing the broadcast — request_kernel_stop(), not libc::raise
//!
//! `libc::raise` is condemned in this project (a self-directed signal makes
//! the measurer and the measured one process). This probe does not signal
//! itself or spawn a child process to signal: `wat::runtime::
//! request_kernel_stop()` is the EXACT call `substrate_on_stop_signal`'s
//! SIGTERM/SIGINT handler makes (`src/process/child.rs:38`) — an
//! `AtomicBool::store` plus an async-signal-safe `libc::write` of one byte to
//! the shutdown worker's wake pipe. That write wakes the worker thread
//! (`init_shutdown_signal_with_inputs`, `src/runtime.rs`), which writes the
//! broadcast wake byte (POLLIN on `SHUTDOWN_BROADCAST_READ_FD`) and then, on
//! return, drops the broadcast write-end (POLLHUP) — the IDENTICAL arm
//! `Sender::send`'s poll depends on. Calling `request_kernel_stop()` directly
//! is a strict subset of the real SIGTERM path: it skips only the OS
//! signal-delivery hop, which is not the mechanism under test, and needs no
//! child process to signal (this probe has none).
//!
//! ## Arming the broadcast
//!
//! `SHUTDOWN_BROADCAST_READ_FD` must be armed (`>= 0`) before the writer
//! blocks, or `Sender::send`'s poll degenerates to POLLOUT-only and waits
//! forever with nothing able to wake it — a DIFFERENT bug that would make
//! this probe prove the opposite of what it claims. Asserted below, before
//! the writer thread is spawned.
//!
//! ## Process isolation
//!
//! `.config/nextest.toml`: "each test runs in its OWN forked process" — this
//! probe's `init_shutdown_signal()` / `request_kernel_stop()` calls mutate
//! process-wide statics (`SHUTDOWN_BROADCAST_READ_FD`, `SHUTDOWN_TX_PTR`,
//! `KERNEL_STOPPED`) but cannot leak into any other test's process.

use std::os::fd::{FromRawFd, OwnedFd};
use std::time::{Duration, Instant};

use wat::comms::process::sender_receiver_from_split_fds;
use wat::comms::SendError;

/// Arc 278 — a `comms::process::Sender::send` blocked writing into a full,
/// undrained pipe wakes and returns `SendError::Shutdown` when the substrate
/// shutdown broadcast fires, instead of blocking forever.
#[test]
fn probe_sender_send_wakes_on_shutdown_broadcast() {
    // ── Step 1: arm the shutdown broadcast BEFORE anything can block ───
    wat::runtime::init_shutdown_signal();
    assert!(
        wat::runtime::SHUTDOWN_BROADCAST_READ_FD.load(std::sync::atomic::Ordering::SeqCst) >= 0,
        "SHUTDOWN_BROADCAST_READ_FD must be armed after init_shutdown_signal() — otherwise \
         Sender::send's poll degenerates to POLLOUT-only and this probe would wait forever with \
         nothing able to wake it (a DIFFERENT bug; would prove the opposite of what this probe \
         claims)."
    );

    // ── Step 2: raw OS pipe; fill the write end to kernel capacity ─────
    // The read end is wrapped into a live Receiver (never `recv`'d, never
    // dropped) for the whole probe — the pipe stays genuinely full; nothing
    // EOFs, nothing drains, and the sole writer never sees EPIPE.
    let mut pipe_fds = [0_i32; 2];
    let ret = unsafe { libc::pipe(pipe_fds.as_mut_ptr()) };
    assert_eq!(ret, 0, "libc::pipe must succeed for the data pipe");
    let read_fd = unsafe { OwnedFd::from_raw_fd(pipe_fds[0]) };
    let write_raw = pipe_fds[1];

    unsafe {
        let orig_flags = libc::fcntl(write_raw, libc::F_GETFL);
        assert!(orig_flags >= 0, "fcntl F_GETFL must succeed");
        let set = libc::fcntl(write_raw, libc::F_SETFL, orig_flags | libc::O_NONBLOCK);
        assert_eq!(set, 0, "fcntl F_SETFL O_NONBLOCK must succeed");

        let chunk = [0u8; 4096];
        loop {
            let n = libc::write(write_raw, chunk.as_ptr() as *const _, chunk.len());
            if n < 0 {
                let err = std::io::Error::last_os_error();
                assert_eq!(
                    err.kind(),
                    std::io::ErrorKind::WouldBlock,
                    "fill-write failed with an unexpected error: {}",
                    err
                );
                break; // kernel pipe buffer is now completely full
            }
        }
        let restore = libc::fcntl(write_raw, libc::F_SETFL, orig_flags);
        assert_eq!(restore, 0, "fcntl F_SETFL restore-to-blocking must succeed");
    }
    // SAFETY: write_raw is a valid, still-open fd from libc::pipe above;
    // ownership transfers to the OwnedFd, which sender_receiver_from_split_fds
    // takes next.
    let write_fd = unsafe { OwnedFd::from_raw_fd(write_raw) };

    let (sender, receiver) = sender_receiver_from_split_fds::<String>(read_fd, write_fd)
        .expect("sender_receiver_from_split_fds over the pre-filled pipe");
    // Held alive for the whole probe: dropping it would close the read end,
    // and the sole writer would see EPIPE (SendError::Disconnected) instead
    // of blocking — a different, unrelated outcome from the one under test.
    let _keep_receiver_alive = receiver;

    // ── Step 3: rendezvous + spawn writer-thread ────────────────────────
    let (ready_tx, ready_rx) = crossbeam_channel::bounded::<()>(0);
    let (result_tx, result_rx) =
        crossbeam_channel::bounded::<Result<(), SendError<String>>>(1);

    std::thread::Builder::new()
        .name("probe-comms-sender".into())
        .spawn(move || {
            // Rendezvous: blocks until the main thread is ready to receive
            // (lock-step) — guarantees this thread is AT the send call point
            // before the shutdown broadcast fires.
            ready_tx.send(()).expect("ready signal send");

            // One more frame into an already-full pipe — the fill loop above
            // left zero free space, so this call blocks in Sender::send's
            // poll loop.
            let outcome = sender.send("x".to_string());
            let _ = result_tx.send(outcome);
        })
        .expect("writer-thread spawn succeeds");

    // ── Step 4: wait for writer-thread readiness, then fire the broadcast ──
    ready_rx.recv().expect("ready signal recv");
    let t0 = Instant::now();
    // Direct broadcast trigger — see module doc "Firing the broadcast" above
    // for why this is used instead of libc::raise or signaling a child.
    wat::runtime::request_kernel_stop();

    // ── Step 5: bounded wait — never a raw join ─────────────────────────
    let outcome = result_rx.recv_timeout(Duration::from_secs(3)).unwrap_or_else(|_| {
        panic!(
            "Sender::send did not return within 3s of the shutdown broadcast firing (elapsed \
             {:?}) — the write is blocked in the kernel with no way to observe the shutdown \
             broadcast. This is the RED state when the poll arm is missing/broken \
             (src/comms/process.rs Sender::send).",
            t0.elapsed()
        )
    });
    let elapsed = t0.elapsed();

    // ── Assertions ────────────────────────────────────────────────────
    match outcome {
        Err(SendError::Shutdown(value)) => {
            assert_eq!(value, "x", "SendError::Shutdown must carry the unsent value back");
        }
        other => panic!(
            "expected Err(SendError::Shutdown(_)) — the poll arm firing on the substrate \
             shutdown broadcast; got {:?}",
            other
        ),
    }
    assert!(
        elapsed.as_millis() < 1000,
        "the shutdown broadcast should reach a blocked Sender::send within ~1s; elapsed: {:?}",
        elapsed
    );
}
