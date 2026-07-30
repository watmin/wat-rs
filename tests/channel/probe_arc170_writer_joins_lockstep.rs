//! Arc 170 closure #5 — "the writer joins the lock-step." RED-gate probe.
//!
//! ## The defect (pre-fix)
//!
//! `PipeWriter::write` (`src/io.rs`) is a bare `libc::write(2)` loop with no
//! poll and a blind `EINTR → continue`. A write to a pipe whose reader is
//! not draining blocks in the kernel — no OS-level event the substrate's
//! shutdown broadcast fires reaches it, because the write syscall itself
//! never returns control to Rust to check anything.
//!
//! This probe fills a real OS pipe to its kernel buffer capacity (the read
//! end is held open but never drained — no EOF, ever), attempts one more
//! `PipeWriter::write` from a background thread (so this probe's own main
//! thread never blocks), fires the substrate shutdown signal (SIGTERM,
//! exactly as `tests/process/shutdown_cascade_pipefd.rs` already does for
//! the read side), and waits — BOUNDED, never a raw `.join()` — for the
//! write to return.
//!
//! Before the fix: the underlying `libc::write(2)` cannot observe anything
//! and blocks forever — the bounded wait below times out and the assertion
//! panics. RED.
//!
//! After the fix: `PipeWriter::write` polls `[fd → POLLOUT,
//! SHUTDOWN_BROADCAST_READ_FD → POLLIN|POLLHUP]` before every attempt,
//! mirroring `channel/transfer.rs`'s read-side `read_one_line` exactly. The
//! broadcast firing wakes it and it returns
//! `Err(RuntimeErrorKind::WriteStopped)` — a NAMED stop, not a bare I/O
//! error — promptly. GREEN.
//!
//! ## Fill strategy
//!
//! Toggle `O_NONBLOCK` on the write fd, write 4 KiB chunks until `EAGAIN`
//! (kernel buffer is then exactly full), restore blocking mode — mirrors
//! the existing non-blocking-probe-then-restore pattern in
//! `src/comms/process.rs`. `PipeWriter` assumes a blocking fd, matching
//! every production pipe it wraps.
//!
//! ## Lock-step discipline
//!
//! Rendezvous via a bounded(0) channel (writer-thread signals readiness
//! right before calling `PipeWriter::write`); the outcome comes back over a
//! second channel via a BOUNDED `recv_timeout` — never a raw `.join()` —
//! so this probe itself can never hang even while the defect is present.
//!
//! ## Why not `typed_send` / `SendOutcome`?
//!
//! `channel/transfer.rs`'s `typed_send` still folds a `WriteStopped` error
//! into `SendOutcome::Disconnected` — unchanged from today. Distinguishing
//! it there needs a new `SendOutcome::Shutdown` variant, which cascades to
//! `kernel/address.rs`'s `ThreadAddress::connect` (the only other match
//! site over `SendOutcome`) and has no honest `ConnectFail` variant to land
//! on without extending the arc 278 "connect' OUTCOME WALL" — a design call
//! outside this brief (STOP-2, reported in the brief's response, not routed
//! around here). This probe therefore targets `PipeWriter::write` directly,
//! the piece actually shipped by this change.

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use std::time::{Duration, Instant};

use wat::io::{PipeWriter, WatWriter};
use wat::runtime::RuntimeErrorKind;

/// Arc 170 — a `PipeWriter::write` blocked writing into a full, undrained
/// pipe wakes with a NAMED `RuntimeErrorKind::WriteStopped` when the
/// substrate shutdown broadcast fires, instead of blocking forever.
#[test]
fn probe_pipe_writer_joins_lockstep_on_shutdown() {
    // ── Step 1: substrate shutdown infra + signal handlers ─────────────
    wat::runtime::init_shutdown_signal();
    wat::process::install_substrate_signal_handlers();

    // ── Step 2: raw OS pipe; fill the write end to kernel capacity ─────
    // The read end is held open (never read, never dropped) for the whole
    // probe — the pipe stays genuinely full; nothing EOFs, nothing drains.
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
    // ownership transfers to the OwnedFd, which PipeWriter takes next.
    let write_fd = unsafe { OwnedFd::from_raw_fd(write_raw) };

    let writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(write_fd));

    // ── Step 3: rendezvous + spawn writer-thread ────────────────────────
    let (ready_tx, ready_rx) = crossbeam_channel::bounded::<()>(0);
    let (result_tx, result_rx) =
        crossbeam_channel::bounded::<Result<usize, wat::runtime::RuntimeError>>(1);

    let writer_clone = writer.clone();
    std::thread::Builder::new()
        .name("probe-pipe-writer".into())
        .spawn(move || {
            // Rendezvous: blocks until the main thread is ready to receive
            // (lock-step) — guarantees this thread is AT the write call
            // point before SIGTERM is raised.
            ready_tx.send(()).expect("ready signal send");

            // One more byte into an already-full pipe — the fill loop above
            // left zero free space, so this call blocks.
            let outcome = writer_clone.write(b"x", wat::rust_caller_span!());
            let _ = result_tx.send(outcome);
        })
        .expect("writer-thread spawn succeeds");

    // ── Step 4: wait for writer-thread readiness, then fire SIGTERM ────
    ready_rx.recv().expect("ready signal recv");
    let t0 = Instant::now();
    let raise_result = unsafe { libc::raise(libc::SIGTERM) };
    assert_eq!(raise_result, 0, "libc::raise(SIGTERM) must succeed");

    // ── Step 5: bounded wait — never a raw join ─────────────────────────
    let outcome = result_rx.recv_timeout(Duration::from_secs(3)).unwrap_or_else(|_| {
        panic!(
            "PipeWriter::write did not return within 3s of SIGTERM firing (elapsed {:?}) — \
             the write is blocked in the kernel with no way to observe the shutdown \
             broadcast. This is the RED state before the arc-170 writer-joins-lockstep fix \
             (src/io.rs PipeWriter::write).",
            t0.elapsed()
        )
    });
    let elapsed = t0.elapsed();

    // ── Assertions ────────────────────────────────────────────────────
    match outcome {
        Err(e) => {
            assert!(
                matches!(e.kind(), RuntimeErrorKind::WriteStopped),
                "expected a NAMED stop (RuntimeErrorKind::WriteStopped); got: {:?}",
                e
            );
        }
        Ok(n) => panic!(
            "write unexpectedly succeeded ({} bytes) into a full, undrained pipe — the fill \
             step above did not actually fill the kernel buffer",
            n
        ),
    }
    assert!(
        elapsed.as_millis() < 100,
        "shutdown cascade should reach a blocked PipeWriter::write within 100ms; elapsed: {:?}",
        elapsed
    );

    // Keep the read-end alive until the assertions above finish — dropping
    // it earlier would let the writer see EPIPE, muddying the RED/GREEN
    // signal with a second, unrelated failure mode.
    drop(read_fd);
}
