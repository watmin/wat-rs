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
//! is not in this stone: the polymorphic verbs shipped in Stone 4.6a-ii/4.6b
//! (live in `src/runtime.rs`; registered at runtime.rs:4206-4218); only
//! no-prime wat-level type registration remains, tracked by the
//! `rune:exigere(attested-arc)` in `mod.rs` (arc 214 Stone 4.6) — cited by
//! grep token, not line number, so the cross-ref cannot drift.
//! This stone is Rust structs + methods only.

use crate::comms::{EdnRepresentable, RecvError, SendError};

// ─── Thread<I, O> ─────────────────────────────────────────────────────────────

/// Thread-tier program peer. Holds the two comms::thread channel endpoints,
/// the crash channel, and the join handle for the spawned thread.
///
/// `I` is the type the parent sends INTO the spawned thread (the thread
/// reads `I` from its own `Receiver<I>`). `O` is the type the spawned
/// thread produces back to the parent (the parent reads `O` from
/// `output: Receiver<O>`).
///
/// Construct via the kernel's spawn dispatcher (Stone 4.5) or the
/// lib-test harness in peer.rs (`Thread { input: Some(..), output, crash, join: Some(..) }`
/// in-crate). Do not construct by naming fields externally — field visibility
/// is `pub(crate)` to enforce the input+output+crash+join invariant.
///
/// ## RAII lifecycle (arc 259 S2b)
///
/// `Drop` calls `drain_and_join` automatically — the worker is always reaped
/// when the peer leaves scope, without any explicit `close'` call. The
/// `close'` verb routes through the same idempotent `drain_and_join`, so
/// `close'`-then-`Drop` is safe (the second call is a no-op via `Option::take`).
///
/// ## Crash channel (arc 259 S3.5a-0)
///
/// Mirrors the `ProcessPeerBundle::err` channel: on a genuine panic (`catch_unwind`
/// returns `Err(payload)`), the worker extracts the reason and sends it over
/// `crash_tx` before that sender drops. `recv` reads the output channel first;
/// on EOF it reads the crash channel — `Ok(reason)` → `Crashed(reason)`;
/// `Err(_)` (crash_tx dropped without sending) → `Disconnected` (clean exit).
pub struct Thread<I: Send + 'static, O: Send + 'static> {
    /// Parent → spawned thread. `Option` so `Drop`/`drain_and_join` can
    /// `take`+drop it BEFORE joining (drain-before-join invariant).
    pub(crate) input: Option<crate::comms::thread::Sender<I>>,
    /// Spawned thread → parent.
    pub(crate) output: crate::comms::thread::Receiver<O>,
    /// Crash channel receiver — the death-time half of the `Result<T,E>` response
    /// (arc 259 S3.5a-0). Mirrors `ProcessPeerBundle::err`. The worker sends
    /// the panic reason here ONLY on a genuine `catch_unwind` `Err(payload)`;
    /// a clean drain (RAII `drain_and_join`) does NOT send — `crash_tx` just
    /// drops. `recv()` reads this on output-EOF: buffered reason → `Crashed`;
    /// EOF only → `Disconnected`.
    pub(crate) crash: crate::comms::thread::Receiver<String>,
    /// Handle for the spawned OS thread. `Option` so `drain_and_join` is
    /// idempotent via `take` — a second call returns `None` (no-op).
    pub(crate) join: Option<std::thread::JoinHandle<()>>,
}

impl<I: Send + 'static, O: Send + 'static> Thread<I, O> {
    /// Send a value to the spawned thread.
    ///
    /// Returns `Err(SendError(value))` if the thread has exited and its
    /// receiver end has been dropped (channel disconnected), or if the
    /// input has already been drained (peer closed).
    pub fn send(&self, value: I) -> Result<(), SendError<I>> {
        match self.input.as_ref() {
            Some(tx) => tx.send(value),
            None => Ok(()), // already drained — worker exited, silently drop
        }
    }

    /// Blocking recv from the spawned thread.
    ///
    /// Mirrors `ProcessPeerBundle::recv` (arc 259 S3.5a-0). Reads the output
    /// channel first. On EOF (the thread's `output_tx` dropped) reads the crash
    /// channel:
    /// - `Ok(reason)` → `Err(PeerRecvError::Crashed(reason))` — genuine panic,
    ///   reason sent before `crash_tx` dropped.
    /// - `Err(_)` → `Err(PeerRecvError::Disconnected)` — clean exit or RAII
    ///   drain (crash_tx dropped without sending).
    ///
    /// `drain_and_join` does NOT call `recv` — it drops input + joins — so
    /// the RAII path is unaffected.
    pub fn recv(&self) -> Result<O, crate::kernel::spawn::PeerRecvError> {
        use crate::kernel::spawn::PeerRecvError;
        match self.output.recv() {
            Ok(v) => Ok(v),
            Err(_) => match self.crash.recv() {
                Ok(reason) => Err(PeerRecvError::Crashed(reason)),
                Err(_) => Err(PeerRecvError::Disconnected),
            },
        }
    }

    /// Drain THEN join — idempotent. The ONE internal reap.
    ///
    /// Drops the input `Sender` first (the worker's `recv'` raises →
    /// the worker exits), then joins the thread (synchronous wait). Both
    /// steps use `Option::take` so repeated calls are no-ops (returns `None`
    /// after the first reap).
    ///
    /// ## Load-bearing order: drain BEFORE join
    ///
    /// Joining first would deadlock: `join` waits for the worker; the worker
    /// is blocked on `recv'` (input not yet dropped). The drain-first order
    /// is the cascade-safety that makes the join hang-free.
    pub(crate) fn drain_and_join(&mut self) -> Option<std::thread::Result<()>> {
        drop(self.input.take()); // drain FIRST: worker's recv' raises → worker exits
        self.join.take().map(|j| j.join()) // THEN join (synchronous); None if already reaped
    }
}

impl<I: Send + 'static, O: Send + 'static> Drop for Thread<I, O> {
    /// RAII backstop — reaps the worker when the peer leaves scope.
    ///
    /// Calls `drain_and_join` (idempotent). `Drop` cannot propagate thread
    /// panics; they are swallowed here. The `close'` verb surfaces panics
    /// explicitly via `drain_and_join`'s return value.
    fn drop(&mut self) {
        let _ = self.drain_and_join();
    }
}

impl<I: Send + 'static + std::fmt::Debug, O: Send + 'static + std::fmt::Debug> std::fmt::Debug
    for Thread<I, O>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Thread")
            .field("input", &self.input)
            .field("output", &self.output)
            .field("crash", &self.crash)
            .field("join", &self.join.as_ref().map(|_| "JoinHandle"))
            .finish()
    }
}

// ─── Peer ─────────────────────────────────────────────────────────────────────

/// Unified, transport-blind bidirectional connection/self peer — arc 209 Stone C0b.2e-i-b.
///
/// `Peer` is the single non-generic endpoint used for BOTH the worker self-peer
/// (handed to a spawned thread via `send'`/`recv'`) AND a connection handle
/// (produced by `peer-pair'`, `connect'`, `accept'`, `socket-pair'`).  The
/// self-vs-connection role is positional at the call site (e.g. arg 0 of
/// `select'`), not a type distinction.
///
/// Arc 258.5b-ii: the send path is now symmetric with recv.  Thread-tier peers
/// carry a `CommSender<Value>` (crossbeam; no serialisation); socket-tier peers
/// carry a `CommSender<String>` (process; `String::to_wire()` is a raw
/// passthrough).  The eval layer encodes with `sym.types()` and calls
/// `Peer::send_wire(String)` for socket-tier; thread-tier goes through the
/// existing `Peer::send(Value)`.  NO thread-local type env is involved.
///
/// Construct via `Peer::from_thread` (thread tier) or `Peer::from_socket`
/// (socket tier).  Do not construct by naming fields directly.
///
/// Carries no `JoinHandle` — lifecycle belongs to the parent (`Thread'` today;
/// RAII in S2b).  For the thread-tier self-peer the instance is created INSIDE
/// the spawned thread's closure to satisfy the `ThreadOwnedCell` owner-thread
/// invariant.

/// Transport-erased send endpoint for a `Peer`.
///
/// `Thread` holds the crossbeam Value channel (no serialisation).
/// `Socket` holds the process String channel (raw-passthrough EDN).
/// The variant determines which `Peer::send*` method is valid at a given
/// call site.
pub(crate) enum PeerTx {
    Thread(Box<dyn crate::comms::CommSender<crate::value::Value> + Send>),
    Socket(Box<dyn crate::comms::CommSender<String> + Send>),
}

pub struct Peer {
    /// Send endpoint — Thread or Socket tier.
    pub(crate) tx: PeerTx,
    /// Receive endpoint (transport-erased; `Send` required; `as_any` for `select'` downcast).
    pub(crate) rx: Box<dyn crate::comms::CommReceiver<crate::value::Value> + Send>,
}

impl Peer {
    /// Construct a crossbeam (thread-tier) peer from a concrete Sender/Receiver pair.
    pub fn from_thread(
        tx: crate::comms::thread::Sender<crate::value::Value>,
        rx: crate::comms::thread::Receiver<crate::value::Value>,
    ) -> Self {
        // Both thread::Sender<Value> and thread::Receiver<Value> are Send.
        Self { tx: PeerTx::Thread(Box::new(tx)), rx: Box::new(rx) }
    }

    /// Construct a socket (process-tier) peer.
    ///
    /// Arc 258.5b-ii: the sender is `Sender<String>` (raw-passthrough EDN) —
    /// the eval layer pre-encodes with `sym.types()` and calls
    /// `Peer::send_wire(String)`.  The receiver remains `Receiver<Value>` so
    /// `Peer::recv()` delivers `Value` directly (decode happens inside the boxed
    /// `CommReceiver<Value>` via `Value::from_wire`, which is acceptable for the
    /// current PEER_TYPE_PATH recv path).
    pub fn from_socket(
        tx: crate::comms::process::Sender<String>,
        rx: crate::comms::process::Receiver<crate::value::Value>,
    ) -> Self {
        Self { tx: PeerTx::Socket(Box::new(tx)), rx: Box::new(rx) }
    }

    /// Returns `true` if this peer is socket-tier (process comms); `false` for
    /// thread-tier (crossbeam).  Used by `eval_peer_send_prime` to choose between
    /// `send(Value)` (thread-tier) and `send_wire(String)` (socket-tier).
    pub fn is_socket_tier(&self) -> bool {
        matches!(self.tx, PeerTx::Socket(_))
    }

    /// Send a value over a **thread-tier** peer.  Encoding is handled internally
    /// by the crossbeam channel (no serialisation — values pass in-process).
    ///
    /// Panics if called on a socket-tier peer — use `send_wire` instead.
    ///
    /// Returns `Err(SendError(value))` if the peer's receiver endpoint has been
    /// dropped (channel disconnected or thread exited).
    pub fn send(&self, value: crate::value::Value) -> Result<(), SendError<crate::value::Value>> {
        match &self.tx {
            PeerTx::Thread(tx) => tx.send(value),
            PeerTx::Socket(_) => panic!("Peer::send called on socket-tier peer — use send_wire"),
        }
    }

    /// Send a pre-encoded EDN wire string over a **socket-tier** peer.
    ///
    /// Arc 258.5b-ii: the eval layer encodes with `sym.types()` and ships the
    /// `String`; the transport (`Sender<String>`) writes the bytes as-is
    /// (`String::to_wire()` is a raw passthrough).
    ///
    /// Returns `Err(SendError(wire))` if the peer's receiver endpoint has been
    /// dropped (channel disconnected or process exited).
    pub fn send_wire(&self, wire: String) -> Result<(), SendError<String>> {
        match &self.tx {
            PeerTx::Socket(tx) => tx.send(wire),
            PeerTx::Thread(_) => panic!("Peer::send_wire called on thread-tier peer — use send"),
        }
    }

    /// Blocking recv.  Decoding is handled internally by the boxed transport impl.
    ///
    /// Cascade-aware (inherited from the underlying comms tier): wakes on
    /// substrate shutdown and returns `Err(RecvError)` rather than hanging.
    /// Also returns `Err(RecvError)` when the peer's sender endpoint is dropped.
    pub fn recv(&self) -> Result<crate::value::Value, RecvError> {
        self.rx.recv()
    }

    /// Read the raw EDN wire string from a **socket-tier** peer WITHOUT decoding.
    ///
    /// Arc 272 6b-ii-α — the trusted-wire door (`decode_trusted_wire`) needs the raw
    /// EDN string so it can reconstruct user-defined records with the type registry.
    /// `recv()` decodes internally via `Value::from_wire` (no type registry) and
    /// fails on tagged user records (e.g. `#user/Counter {:base 1000}`).
    ///
    /// The eval layer (`eval_peer_recv_prime`) calls this for socket-tier self-peers,
    /// then passes the returned string through `decode_trusted_wire(s, sym.types())`.
    /// Thread-tier peers do not go through EDN serialisation; they must use `recv()`.
    ///
    /// Panics if called on a thread-tier peer (programming error — use `recv()`).
    pub fn recv_wire(&self) -> Result<String, RecvError> {
        // Arc 272 6b-ii-α: downcast the type-erased CommReceiver<Value> back to the
        // concrete process::Receiver<Value> so we can call recv_wire_raw(), which reads
        // the pipe bytes and returns the UTF-8 frame without calling T::from_wire.
        self.rx
            .as_any()
            .downcast_ref::<crate::comms::process::Receiver<crate::value::Value>>()
            .expect("recv_wire called on non-socket-tier peer (thread::Receiver does not impl from_wire via pipe)")
            .recv_wire_raw()
    }
}

impl std::fmt::Debug for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Peer").finish_non_exhaustive()
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
/// Both `I` and `O` must implement `EdnRepresentable` because the
/// comms::process tier serializes values through `HolonAST` ↔ EDN bytes
/// over the anonymous pipe.
///
/// Construct via the kernel's spawn dispatcher (Stone 4.5) or via
/// `Process::new_for_test` in integration tests. Do not construct by
/// naming fields directly — field visibility is `pub(crate)` to enforce
/// the invariant that input, output, and child are always co-created.
pub struct Process<I: EdnRepresentable, O: EdnRepresentable> {
    /// Parent → child process.
    pub(crate) input: crate::comms::process::Sender<I>,
    /// Child process → parent.
    pub(crate) output: crate::comms::process::Receiver<O>,
    /// Canonical child-process handle (arc 213 Pidfd). Race-free: the fd
    /// is bound to this exact child at fork time — not to the (potentially
    /// reused) PID. Used by `wait` and `close` for child lifecycle management.
    /// The field name is `pidfd` to mirror the type name.
    pub(crate) pidfd: crate::process::Pidfd,
}

impl<I: EdnRepresentable, O: EdnRepresentable> Process<I, O> {
    /// Construct a `Process` peer from its three components.
    ///
    /// Intended for integration tests that create the underlying channel
    /// pairs + fork the child directly (e.g. `peer_process_round_trip.rs`).
    /// In-crate production paths use `spawn_process_peer` (kernel/spawn.rs)
    /// which constructs the peer via struct literal (pub(crate) access).
    #[doc(hidden)]
    pub fn new_for_test(
        input: crate::comms::process::Sender<I>,
        output: crate::comms::process::Receiver<O>,
        pidfd: crate::process::Pidfd,
    ) -> Self {
        Self { input, output, pidfd }
    }

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

    /// Close both channel endpoints and return the Pidfd.
    ///
    /// Dropping `self.output` (the `comms::process::Receiver`) closes the
    /// parent's read end of the pipe AND releases the persistent io_uring
    /// ring fd it owns (see `comms::process::Receiver::ring`). Dropping
    /// `self.input` (the Sender) closes the parent's write end of the pipe
    /// (the child's Receiver sees EOF on its next recv). The returned Pidfd
    /// owns only the pidfd itself; its Drop closes that single fd.
    ///
    /// Prefer `wait(self)` for the common "close + wait" pattern.
    pub fn close(self) -> crate::process::Pidfd {
        // Drop input: child sees EOF on its input pipe.
        // Drop output: RAII cleanup of parent's output receiver.
        drop(self.input);
        drop(self.output);
        self.pidfd
    }

    /// Close both channel endpoints and block until the child process exits.
    ///
    /// Equivalent to `self.close().wait_status()`. Returns the child's exit
    /// status via Pidfd::wait_status (blocking waitid on the pidfd; reaps
    /// the zombie atomically).
    pub fn wait(self) -> std::io::Result<crate::process::ExitStatus> {
        let pidfd = self.close();
        pidfd.wait_status()
    }
}

impl<I: EdnRepresentable + std::fmt::Debug, O: EdnRepresentable + std::fmt::Debug>
    std::fmt::Debug for Process<I, O>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("input", &self.input)
            .field("output", &self.output)
            .field("pidfd", &self.pidfd)
            .finish()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Lib-safe unit test for Thread<I, O> round-trip.
    ///
    /// Constructs a Thread peer by hand (bypassing the spawn dispatcher) to
    /// stay lib-safe — no WatAST parsing or dispatcher wiring in a unit test.
    /// Creates two comms::thread pairs, spawns a std::thread that recvs I
    /// from its Receiver and sends O = transform(I) to its Sender, then
    /// builds a Thread peer using the parent-side endpoints. Asserts that
    /// peer.send(x) followed by peer.recv() returns transform(x), and that
    /// join completes cleanly.
    ///
    /// This is the lib-safe gate for Stone 4.4. The process peer round-trip
    /// lives in the integration test (tests/kernel/peer_process_round_trip.rs)
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

        // Crash channel — not exercised in this round-trip test (no panic),
        // but required by the struct layout (arc 259 S3.5a-0).
        let (_crash_tx, crash_rx) = crate::comms::thread::pair::<String>();

        // Build the Thread peer using parent-side endpoints.
        let mut peer = Thread {
            input: Some(input_tx),
            output: output_rx,
            crash: crash_rx,
            join: Some(join),
        };

        // Send 21 → expect 42 back (doubling transform).
        peer.send(21_i64).expect("peer.send must succeed");
        let got = peer.recv().expect("peer.recv must return the reply");
        assert_eq!(got, 42_i64, "thread doubled 21 → 42; got {}", got);

        // Drain and join cleanly — thread should have exited after one send.
        peer.drain_and_join().expect("drain_and_join must return Some").expect("thread join must succeed");
    }
}
