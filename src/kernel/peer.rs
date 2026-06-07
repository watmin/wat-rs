//! # Kernel peer types — Stone 4.4 (arc 214 Slice 4)
//!
//! `Thread<I, O>` and `Process<I, O>` are the handle-shaped wrappers over
//! the comms tiers. The substrate spawns a worker, produces one of these
//! structs, and the caller uses send/recv/join without ever touching raw
//! Sender/Receiver pairs or JoinHandles directly.
//!
//! ## Thread<I, O>
//!
//! Wraps a `comms::thread` channel pair plus a `std::thread::JoinHandle`.
//! The parent sends `I` into `input` and receives `O` from `output`.
//! `close(self)` drops both endpoints (letting the crossbeam channels
//! disconnect naturally) and returns the JoinHandle so the caller can
//! block on thread exit. `join(self)` is a convenience that closes
//! both endpoints and blocks on the handle in one call.
//!
//! ## Process<I, O>
//!
//! Wraps a `comms::process` channel pair plus a `Pidfd` (the canonical
//! arc 213 child-process handle — race-free, bound to the exact child at
//! fork time). The parent sends `I` into `input` and receives `O` from
//! `output`. `close(self)` drops both channel endpoints and returns the
//! Pidfd so the caller can wait for the child. `wait(self)` is the
//! convenience that closes both and calls `Pidfd::wait_status`.
//!
//! ## Cascade contract (inherited from comms tiers)
//!
//! All blocking operations (`recv`, `wait`) inherit the cascade contract
//! from the underlying comms tier — they wake on substrate shutdown without
//! any additional wiring here. The kernel layer adds NO new blocking
//! primitives.
//!
//! ## Not in this stone
//!
//! Wat-level type registration (`:wat::kernel::Thread<I,O>` / `Process<I,O>`)
//! is intentionally absent — it ships with the polymorphic verbs in Stone 4.6.
//! This stone is Rust structs + methods only.

use crate::comms::{HolonRepresentable, RecvError, SendError};

// ─── Thread<I, O> ─────────────────────────────────────────────────────────────

/// Thread-tier program peer. Holds the two comms::thread channel endpoints
/// and the join handle for the spawned thread.
///
/// `I` is the type the parent sends INTO the spawned thread (the thread
/// reads `I` from its own `Receiver<I>`). `O` is the type the spawned
/// thread produces back to the parent (the parent reads `O` from
/// `output: Receiver<O>`).
///
/// Construct via the kernel's spawn dispatcher (Stone 4.5); this struct
/// is never constructed directly by user code.
pub struct Thread<I: Send + 'static, O: Send + 'static> {
    /// Parent → spawned thread.
    pub input: crate::comms::thread::Sender<I>,
    /// Spawned thread → parent.
    pub output: crate::comms::thread::Receiver<O>,
    /// Handle for the spawned OS thread.
    pub join: std::thread::JoinHandle<()>,
}

impl<I: Send + 'static, O: Send + 'static> Thread<I, O> {
    /// Send a value to the spawned thread.
    ///
    /// Returns `Err(SendError(value))` if the thread has exited and its
    /// receiver end has been dropped (channel disconnected).
    pub fn send(&self, value: I) -> Result<(), SendError<I>> {
        self.input.send(value)
    }

    /// Blocking recv from the spawned thread.
    ///
    /// Cascade-aware (inherited from `comms::thread::Receiver`): wakes on
    /// substrate shutdown and returns `Err(RecvError)` rather than hanging.
    /// Also returns `Err(RecvError)` when the thread exits and its sender
    /// end is dropped.
    pub fn recv(&self) -> Result<O, RecvError> {
        self.output.recv()
    }

    /// Non-blocking recv. Returns `Some(value)` if a value is immediately
    /// available; `None` if nothing is ready now OR the thread has exited.
    ///
    /// Arc 253 2-state: the old 3-state Empty/Disconnected split is
    /// eliminated — both map to `None` (inherited from comms tier).
    pub fn try_recv(&self) -> Option<O> {
        self.output.try_recv()
    }

    /// Close both channel endpoints and return the JoinHandle.
    ///
    /// Dropping the input Sender disconnects the thread's channel (the
    /// thread's Receiver sees `RecvError` on its next recv). Dropping the
    /// output Receiver is RAII. The returned JoinHandle lets the caller
    /// block on thread exit without introducing a new blocking layer here.
    ///
    /// Prefer `join(self)` for the common "close + wait" pattern.
    pub fn close(self) -> std::thread::JoinHandle<()> {
        // Drop input: the thread's receiver side sees disconnected.
        // Drop output: RAII cleanup of our receive side.
        drop(self.input);
        drop(self.output);
        self.join
    }

    /// Close both channel endpoints and block until the spawned thread exits.
    ///
    /// Equivalent to `self.close().join()`. Returns the thread's join result.
    pub fn join(self) -> std::thread::Result<()> {
        let handle = self.close();
        handle.join()
    }
}

impl<I: Send + 'static + std::fmt::Debug, O: Send + 'static + std::fmt::Debug> std::fmt::Debug
    for Thread<I, O>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Thread")
            .field("input", &self.input)
            .field("output", &self.output)
            .field("join", &"JoinHandle")
            .finish()
    }
}

// ─── Process<I, O> ────────────────────────────────────────────────────────────

/// Process-tier program peer. Holds the two comms::process channel endpoints
/// and the canonical arc 213 Pidfd (race-free process handle).
///
/// `I` is the type the parent sends INTO the child process (the child reads
/// `I` from its own `Receiver<I>`). `O` is the type the child process produces
/// back to the parent (the parent reads `O` from `output: Receiver<O>`).
///
/// Both `I` and `O` must implement `HolonRepresentable` because the
/// comms::process tier serializes values through `HolonAST` ↔ EDN bytes
/// over the anonymous pipe.
///
/// Construct via the kernel's spawn dispatcher (Stone 4.5); this struct
/// is never constructed directly by user code.
pub struct Process<I: HolonRepresentable, O: HolonRepresentable> {
    /// Parent → child process.
    pub input: crate::comms::process::Sender<I>,
    /// Child process → parent.
    pub output: crate::comms::process::Receiver<O>,
    /// Canonical child-process handle (arc 213 Pidfd). Race-free: the fd
    /// is bound to this exact child at fork time — not to the (potentially
    /// reused) PID. Used by `wait` and `close` for child lifecycle management.
    pub child: crate::fork::Pidfd,
}

impl<I: HolonRepresentable, O: HolonRepresentable> Process<I, O> {
    /// Send a value to the child process via the comms::process pipe.
    ///
    /// Returns `Err(SendError(value))` if the child has exited and the
    /// pipe's read end is closed (EPIPE).
    pub fn send(&self, value: I) -> Result<(), SendError<I>> {
        self.input.send(value)
    }

    /// Blocking recv from the child process.
    ///
    /// Cascade-aware (inherited from `comms::process::Receiver`): wakes on
    /// substrate shutdown via the broadcast pipe and returns `Err(RecvError)`
    /// rather than hanging. Also returns `Err(RecvError)` when the child exits
    /// and the pipe's write end is closed (EOF).
    pub fn recv(&self) -> Result<O, RecvError> {
        self.output.recv()
    }

    /// Non-blocking recv. Returns `Some(value)` if a frame is immediately
    /// available; `None` if nothing is ready now OR the child has exited.
    ///
    /// Arc 253 2-state: the old 3-state Empty/Disconnected split is
    /// eliminated — both map to `None` (inherited from comms tier).
    pub fn try_recv(&self) -> Option<O> {
        self.output.try_recv()
    }

    /// Close both channel endpoints and return the Pidfd.
    ///
    /// Dropping the input Sender closes the parent's write end of the pipe
    /// (the child's Receiver sees EOF on its next recv). Dropping the output
    /// Receiver closes the parent's read end. The returned Pidfd lets the
    /// caller block on child exit via `pidfd.wait_status()`.
    ///
    /// Prefer `wait(self)` for the common "close + wait" pattern.
    pub fn close(self) -> crate::fork::Pidfd {
        // Drop input: child sees EOF on its input pipe.
        // Drop output: RAII cleanup of parent's output receiver.
        drop(self.input);
        drop(self.output);
        self.child
    }

    /// Close both channel endpoints and block until the child process exits.
    ///
    /// Equivalent to `self.close().wait_status()`. Returns the child's exit
    /// status via Pidfd::wait_status (blocking waitid on the pidfd; reaps
    /// the zombie atomically).
    pub fn wait(self) -> std::io::Result<crate::fork::ExitStatus> {
        let pidfd = self.close();
        pidfd.wait_status()
    }
}

impl<I: HolonRepresentable + std::fmt::Debug, O: HolonRepresentable + std::fmt::Debug>
    std::fmt::Debug for Process<I, O>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("input", &self.input)
            .field("output", &self.output)
            .field("child", &self.child)
            .finish()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Lib-safe unit test for Thread<I, O> round-trip.
    ///
    /// Constructs a Thread peer by hand (no spawn dispatcher — Stone 4.5
    /// is not yet built): create two comms::thread pairs, spawn a std::thread
    /// that recvs I from its Receiver and sends O = transform(I) to its Sender,
    /// then build a Thread peer using the parent-side endpoints. Assert that
    /// peer.send(x) followed by peer.recv() returns transform(x), and that
    /// join completes cleanly.
    ///
    /// This is the lib-safe gate for Stone 4.4. The process peer round-trip
    /// lives in the integration test (tests/comms/peer_process_round_trip.rs)
    /// because it forks and must run under setsid containment.
    #[test]
    fn thread_peer_round_trip() {
        // Two channel pairs: one for parent→thread (input), one for thread→parent (output).
        let (input_tx, input_rx) = crate::comms::thread::pair::<i64>();
        let (output_tx, output_rx) = crate::comms::thread::pair::<i64>();

        // Spawn a thread that recvs one i64 and sends back doubled value.
        let join = std::thread::spawn(move || {
            let value = input_rx.recv().expect("thread: recv from parent");
            output_tx
                .send(value * 2)
                .expect("thread: send reply to parent");
        });

        // Build the Thread peer using parent-side endpoints.
        let peer = Thread {
            input: input_tx,
            output: output_rx,
            join,
        };

        // Send 21 → expect 42 back (doubling transform).
        peer.send(21_i64).expect("peer.send must succeed");
        let got = peer.recv().expect("peer.recv must return the reply");
        assert_eq!(got, 42_i64, "thread doubled 21 → 42; got {}", got);

        // Join cleanly — thread should have exited after one send.
        peer.join().expect("thread join must succeed");
    }
}
