//! Arc 214 Slice 4 Stone 4.4 — `kernel::peer::Process<I, O>` round-trip.
//!
//! Integration probe for the Process peer struct. Constructs a real forked
//! echo child using `comms::process::pair` + `spawn_lifelined`, builds a
//! `kernel::peer::Process<String, String>` from the resulting endpoints,
//! and asserts a send→recv round-trip returns the original value.
//!
//! # Why this lives here (not in src/kernel/peer.rs as a lib test)
//!
//! The process-tier comms::process::pair + fork combination creates child
//! processes. A lib test that forks inside the cargo test binary causes
//! fd/lock inheritance from all sibling tests — this is the process-leak
//! class the `run_in_fork` + `setsid` containment pattern prevents in the
//! comms/ tier tests. By placing the process peer test here, build.rs picks
//! it up automatically (no mod list to edit) and it runs in the comms
//! integration-test binary alongside the existing process-tier tests.
//!
//! # Containment
//!
//! The test uses `spawn_lifelined` directly (like `pidfd_primitive.rs`)
//! rather than `run_in_fork`. The echo child calls `libc::_exit` — it
//! never returns to Rust, so no parent atexit handlers fire. The Pidfd
//! returned by `spawn_lifelined` is handed to the Process peer and consumed
//! by `Process::wait`, which calls `Pidfd::wait_status` (blocking waitid).
//!
//! # Leak safety
//!
//! `comms::process` is RAII — both Sender and Receiver hold OwnedFds whose
//! Drop calls `libc::close`. No fd leak class is possible here: the Process
//! peer struct's `wait(self)` consumes all three resources (Sender, Receiver,
//! Pidfd) via `close(self)` → Drop. The echo child's pipe end (write-end of
//! the output pipe inherited at fork) is closed by the child's `libc::_exit`.
//!
//! # Test shape
//!
//! 1. Create two `comms::process::pair::<String>()` channel pairs:
//!    - `(input_tx, input_rx)`: parent sends input → child reads.
//!    - `(output_tx, output_rx)`: child sends reply → parent reads.
//! 2. `spawn_lifelined` forks the echo child. The child:
//!    - Reads one String from `input_rx` (blocking recv on the read pipe).
//!    - Writes the same String back on `output_tx` (echo).
//!    - Calls `libc::_exit(0)`.
//! 3. The parent builds `Process { input: input_tx, output: output_rx, child: pidfd }`.
//! 4. `peer.send("hello")` → `peer.recv()` must return `"hello"`.
//! 5. `peer.wait()` must return `ExitStatus::Exited(0)`.

use wat::comms::process::pair;
use wat::process::{spawn_lifelined, ExitStatus};
use wat::kernel::peer::Process;

/// Process peer round-trip: send a String to the echo child; recv it back.
///
/// Marked #[ignore] so it runs via `integration-run.sh` (setsid + timeout
/// per-binary), not via `cargo test --test test` which deadlocks on the old
/// typed_channel/thread_io/fork stack per the campaign discipline. Run
/// directly via:
///   cargo test --test comms peer_process_round_trip -- --ignored
#[test]
#[ignore = "process-tier probe: run via integration-run.sh or with --ignored flag; never via raw cargo test --test test"]
fn process_peer_round_trip() {
    // ── Step 1: create channel pairs ─────────────────────────────────────
    // input: parent writes → child reads. output: child writes → parent reads.
    let (input_tx, input_rx) = pair::<String>().expect("input pair");
    let (output_tx, output_rx) = pair::<String>().expect("output pair");

    // Capture raw file descriptors BEFORE fork so the child branch can
    // access them (clone3 copies the parent fd table; the child sees the
    // same raw fd numbers in its own fd table copy).
    //
    // We do NOT pass the typed Sender/Receiver into the closure directly —
    // comms::process::Sender<T> and Receiver<T> are !Clone; their OwnedFds
    // own the pipe fds. We instead use the raw fds on the child side and
    // rebuild typed endpoints there (the same pattern fork.rs uses for the
    // full spawn machinery).
    //
    // However, for the CHILD branch here we only need to:
    //   (a) recv one String from input_rx  (the child's read-end of input pipe)
    //   (b) send it back on output_tx      (the child's write-end of output pipe)
    //
    // The simplest correct approach: move the Receiver and Sender endpoints
    // into the child closure directly. After clone3 the child address space
    // is a COW copy of the parent; the OwnedFds are duplicated in the
    // child's fd table. The child's Drop on these OwnedFds is safe because
    // each process has its OWN fd table — there is no double-close across
    // the fork boundary (the parent's OwnedFds remain alive in the parent's
    // fd table independently).
    //
    // UnwindSafe: OwnedFd + the comms types don't impl UnwindSafe. Wrap in
    // AssertUnwindSafe because the child calls libc::_exit (never unwinds).
    use std::panic::AssertUnwindSafe;
    let child_input_rx = AssertUnwindSafe(input_rx);
    let child_output_tx = AssertUnwindSafe(output_tx);

    let (pidfd, _lifeline) = spawn_lifelined(move |_lifeline_r: i32| {
        // ── CHILD BRANCH ─────────────────────────────────────────────────
        // Unwrap the AssertUnwindSafe wrappers to access the actual values.
        let input_rx = child_input_rx.0;
        let output_tx = child_output_tx.0;

        // Echo: recv one String from parent, send it back.
        let value = match input_rx.recv() {
            Ok(v) => v,
            Err(_) => unsafe { libc::_exit(1) },
        };
        match output_tx.send(value) {
            Ok(()) => {}
            Err(_) => unsafe { libc::_exit(1) },
        }
        // Clean exit: drop both endpoints (pipes close → parent sees EOF).
        drop(input_rx);
        drop(output_tx);
        unsafe { libc::_exit(0) };
    })
    .expect("spawn_lifelined must succeed");

    // ── PARENT BRANCH ─────────────────────────────────────────────────────
    // Build the Process peer using the parent-side endpoints.
    let peer = Process {
        input: input_tx,
        output: output_rx,
        child: pidfd,
    };

    // Send "hello" to the child; expect it echoed back.
    peer.send("hello".to_string()).expect("peer.send must succeed");
    let got = peer.recv().expect("peer.recv must return the echoed value");
    assert_eq!(got, "hello", "echo child must return the sent string; got {:?}", got);

    // Wait for the child to exit cleanly (peer.wait consumes the peer).
    let status = peer.wait().expect("peer.wait (Pidfd::wait_status) must succeed");
    assert_eq!(
        status,
        ExitStatus::Exited(0),
        "echo child must exit 0; got {:?}",
        status
    );
}
