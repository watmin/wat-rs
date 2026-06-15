//! # Kernel listener entity — Arc 209 Stone C0b.2e-ii
//!
//! Introduces `CommListener` (a kernel trait with a single `accept` method)
//! and the `Listener { inner: Box<dyn CommListener> }` entity — a proper
//! first-class type under `LISTENER_TYPE_PATH`, replacing:
//!
//! - The thread-tier fiction of a raw `comms::thread::Receiver` called "Listener'"
//! - The process-tier socket listener opaque (arc 209 C0b.2c, now retired)
//!
//! Both tiers now produce the same `Listener` entity; `accept'` collapses to
//! one arm that downcasts the opaque and calls `inner.accept(sym, span)`.
//!
//! ## Layering
//!
//! `kernel → comms` (crossbeam + process) and `kernel → runtime` (for `Value`,
//! `EvalBreak`, `SymbolTable`) — same kernel→runtime direction already used by
//! `kernel::spawn`. No `comms → kernel` direction; no cycle.
//!
//! ## What is NOT here (by design)
//!
//! - `reactor_class` / `listen_fd` methods on `CommListener` — added by C0b.3a-ii,
//!   which is both the first consumer AND the definition site. Building them here
//!   would be an unbuilt forcing function.
//! - Any remote `CommListener` impl — organic future addition.

use std::collections::HashSet;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixListener;
use std::sync::{Arc, Mutex};

use crate::channel::inner::ReceiverInner;
use crate::channel::SenderInner;
use crate::kernel::peer::Peer;
use crate::rust_deps::custodia::ThreadOwnedCell;
use crate::rust_deps::marshal::make_rust_opaque;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::span::Span;

// ─── CommListener trait ───────────────────────────────────────────────────────

/// Transport-blind kernel trait: block until a connection arrives and return
/// the server-side `Peer`.
///
/// # Deadlock contract
///
/// Each impl preserves its transport's exact accept semantics:
/// - Crossbeam: rendezvous `recv` (blocks until a `connect'` client sends).
/// - Socket: poll-driven non-blocking accept (C0b.3a-i invariant — the fd
///   is non-blocking; a spurious POLLIN → EWOULDBLOCK → re-poll, never blocks).
///
/// [[feedback_vended_primitives_never_deadlock]]
pub trait CommListener: Send + Sync {
    /// Block until a connection arrives; wrap + return the server-side Peer.
    fn accept(&self, sym: &SymbolTable, span: &Span) -> Result<Peer, EvalBreak>;

    /// Return a `&dyn Any` reference for downcasting to the concrete impl.
    ///
    /// Used by `poll'` to extract the raw crossbeam `Receiver` (thread tier)
    /// or the raw listen fd (process tier, C0b.3a-ii) from the concrete impl.
    fn as_any_ref(&self) -> &dyn std::any::Any;

    /// Arc 209 C0b.3a-ii — which wait-primitive class this listener uses.
    ///
    /// `CrossbeamListener` → `ReactorClass::InMemory` (parked-thread crossbeam select).
    /// `SocketListener`    → `ReactorClass::Fd`       (kernel fd-poll via io_uring).
    ///
    /// Mirrors `CommReceiver::reactor_class` — one named discriminant across the
    /// whole connection surface (Receiver, Listener, any future remote).
    fn reactor_class(&self) -> crate::comms::ReactorClass;
}

// ─── CrossbeamListener ────────────────────────────────────────────────────────

/// Thread-tier `CommListener`: the rendezvous receiver from `listener'`.
///
/// `accept` blocks on the rendezvous until a connect-request arrives (sent by
/// `connect'`), then unpacks the server's raw halves and wraps them as a
/// unified `Peer` via `Peer::from_thread`.
///
/// Verbatim body from the former thread arm of `eval_accept_prime`.
pub struct CrossbeamListener {
    pub(crate) rx: crate::comms::thread::Receiver<Value>,
}

impl CommListener for CrossbeamListener {
    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }

    fn reactor_class(&self) -> crate::comms::ReactorClass {
        crate::comms::ReactorClass::InMemory
    }

    fn accept(&self, sym: &SymbolTable, span: &Span) -> Result<Peer, EvalBreak> {
        const OP: &str = ":wat::kernel::accept'";
        // Block on the rendezvous until a connect-request arrives.
        let cr_value = match crate::channel::typed_recv(
            &ReceiverInner::Comms(self.rx.clone()),
            sym.types().map(|a| a.as_ref()),
            span.clone(),
        ) {
            crate::channel::RecvOutcome::Value(v) => v,
            crate::channel::RecvOutcome::Disconnected
            | crate::channel::RecvOutcome::Shutdown => {
                return Err(RuntimeError {
                    span: span.clone(),
                    kind: RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "accept': rendezvous recv failed — address was dropped or shutdown"
                            .into(),
                    },
                }
                .into());
            }
            crate::channel::RecvOutcome::DecodeError(msg) => {
                return Err(RuntimeError {
                    span: span.clone(),
                    kind: RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!("accept': rendezvous recv decode error: {}", msg),
                    },
                }
                .into());
            }
        };
        // Unpack + wrap the server Peer'<R,S> end on THIS thread.
        // Mirrors wrap_connect_request (runtime.rs) but returns Peer directly.
        let mut items: Vec<Value> = match cr_value {
            Value::Tuple(arc) => match Arc::try_unwrap(arc) {
                Ok(vec) => vec,
                Err(arc) => (*arc).clone(),
            },
            other => {
                return Err(RuntimeError {
                    span: span.clone(),
                    kind: RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!(
                            "connect-request must be a Tuple; got {:?}",
                            other.type_name()
                        ),
                    },
                }
                .into());
            }
        };
        if items.len() != 2 {
            return Err(RuntimeError {
                span: span.clone(),
                kind: RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                        "connect-request tuple must have 2 elements; got {}",
                        items.len()
                    ),
                },
            }
            .into());
        }
        let resp_tx_val = items.remove(1);
        let req_rx_val = items.remove(0);
        // Extract req_rx (Receiver<Value>) — moved out, unique owner.
        let req_rx = match req_rx_val {
            Value::wat__kernel__Receiver(arc) => match Arc::try_unwrap(arc) {
                Ok(ReceiverInner::Comms(rx)) => rx,
                Ok(_) => {
                    return Err(RuntimeError {
                        span: span.clone(),
                        kind: RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: "connect-request rx is not a comms (thread-tier) receiver"
                                .into(),
                        },
                    }
                    .into());
                }
                Err(_) => {
                    return Err(RuntimeError {
                        span: span.clone(),
                        kind: RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason:
                                "connect-request rx has unexpected additional references".into(),
                        },
                    }
                    .into());
                }
            },
            other => {
                return Err(RuntimeError {
                    span: span.clone(),
                    kind: crate::runtime::RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "Receiver (connect-request req_rx)",
                        got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                    },
                }
                .into());
            }
        };
        // Extract resp_tx (Sender<Value>) — moved out, unique owner.
        let resp_tx = match resp_tx_val {
            Value::wat__kernel__Sender(arc) => match Arc::try_unwrap(arc) {
                Ok(SenderInner::Comms { sender, .. }) => sender,
                Ok(_) => {
                    return Err(RuntimeError {
                        span: span.clone(),
                        kind: RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: "connect-request tx is not a comms (thread-tier) sender"
                                .into(),
                        },
                    }
                    .into());
                }
                Err(_) => {
                    return Err(RuntimeError {
                        span: span.clone(),
                        kind: RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason:
                                "connect-request tx has unexpected additional references".into(),
                        },
                    }
                    .into());
                }
            },
            other => {
                return Err(RuntimeError {
                    span: span.clone(),
                    kind: crate::runtime::RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "Sender (connect-request resp_tx)",
                        got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                    },
                }
                .into());
            }
        };
        // Wrap the server Peer'<R,S> end on THIS thread (custody holds).
        Ok(Peer::from_thread(resp_tx, req_rx))
    }
}

// ─── SocketListener ───────────────────────────────────────────────────────────

/// Process-tier `CommListener`: the bound non-blocking `UnixListener`.
///
/// `accept` runs the poll-driven non-blocking accept loop (C0b.3a-i):
/// `Select::listener(fd)` → `POLLIN` → non-blocking `.accept()` →
/// `Ok(stream)` → wrap as `Peer`. Spurious wakeup (`WouldBlock`) → re-poll.
/// Shutdown → clean error.
///
/// Verbatim body from the former socket arm of `eval_accept_prime`.
pub struct SocketListener {
    pub(crate) listener: UnixListener,
    /// Arc 209 C0b.3b-b — the allow-set: a pid is in it or it isn't. Birth-seeded with
    /// the owner's pid (getppid() = the spawner, trusted by construction). A connector
    /// whose SO_PEERCRED pid ∉ this set (or whose uid ≠ ours) is bounced at accept.
    pub(crate) allowed_pids: Mutex<HashSet<i32>>,
}

impl SocketListener {
    /// The accept-gate decision: admit only a peer in **`only-my-peers`** — our euid AND a pid in the
    /// allow-set (the lineage set). Arc 272 v4 — the rule itself lives in `capability::CommsPolicy`
    /// (the powerbox); this gate CONSULTS it. The connect gate (`kernel::address`) consults the same
    /// policy from the other side. (Prior "SO_PEERCRED is local mTLS" was an overclaim — it is mutual
    /// UDS peer-cred, NOT TLS.) Pure + Rust-testable.
    pub(crate) fn authorizes(&self, cred: &crate::comms::process::PeerCred) -> bool {
        let lineage = self.allowed_pids.lock().unwrap();
        crate::capability::CommsPolicy::OnlyMyPeers { lineage: &lineage }
            .admits(cred, unsafe { libc::geteuid() })
    }
    /// Owner provisions another pid (beyond the birth-seeded self).
    pub(crate) fn allow(&self, pid: i32) {
        self.allowed_pids.lock().unwrap().insert(pid);
    }
    /// Owner de-provisions a pid (future accepts of it bounce).
    pub(crate) fn deny(&self, pid: i32) {
        self.allowed_pids.lock().unwrap().remove(&pid);
    }
}

impl CommListener for SocketListener {
    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }

    fn reactor_class(&self) -> crate::comms::ReactorClass {
        crate::comms::ReactorClass::Fd
    }

    fn accept(&self, _sym: &SymbolTable, span: &Span) -> Result<Peer, EvalBreak> {
        const OP: &str = ":wat::kernel::accept'";
        // Arc 209 C0b.3a-i — poll-driven non-blocking accept.
        // Build a Select with just the listener arm (one fd). Loop:
        //   Listener → non-blocking accept → Ok(stream) → wrap | WouldBlock → re-poll
        //   Shutdown → clean error; Recv impossible (no receivers registered).
        // The listen fd is non-blocking (set at listener' bind time) so a spurious
        // POLLIN wakeup → EWOULDBLOCK → re-poll, never blocks. Ring is reused across
        // iterations (same sel, not rebuilt per loop).
        use std::os::fd::AsRawFd;
        let raw = self.listener.as_raw_fd();
        let mut sel = crate::comms::process::Select::<Value>::new();
        sel.listener(raw);
        loop {
            match sel.select().map_err(|e| RuntimeError {
                span: span.clone(),
                kind: RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("accept' select: {}", e),
                },
            })? {
                crate::comms::SelectOutcome::Listener => {
                    match self.listener.accept() {
                        Ok((stream, _)) => {
                            // Inline wrap_stream_as_socket_peer (Arc 209 C0b.2e-i-b).
                            // Arc 209 C0b.2e-i-b: switched from String to Value — encoding
                            // is internal.
                            let (tx, rx) =
                                crate::comms::process::sender_receiver_from_fd::<Value>(
                                    OwnedFd::from(stream),
                                )
                                .map_err(|e| RuntimeError {
                                    span: span.clone(),
                                    kind: RuntimeErrorKind::MalformedForm {
                                        head: OP.into(),
                                        reason: format!("wrap socket stream failed: {}", e),
                                    },
                                })?;
                            return Ok(Peer::from_socket(tx, rx));
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            continue; // spurious; re-poll
                        }
                        Err(e) => {
                            return Err(RuntimeError {
                                span: span.clone(),
                                kind: RuntimeErrorKind::MalformedForm {
                                    head: OP.into(),
                                    reason: format!(
                                        "accept on abstract UDS listener failed: {}",
                                        e
                                    ),
                                },
                            }
                            .into());
                        }
                    }
                }
                crate::comms::SelectOutcome::Shutdown => {
                    return Err(RuntimeError {
                        span: span.clone(),
                        kind: RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: "accept': interrupted by shutdown".into(),
                        },
                    }
                    .into());
                }
                crate::comms::SelectOutcome::Recv { .. } => {
                    unreachable!("accept' Select has no receivers")
                }
            }
        }
    }
}

// ─── Listener entity ──────────────────────────────────────────────────────────

/// The unified, transport-blind listener entity.
///
/// Stored as a `RustOpaque` under `LISTENER_TYPE_PATH` (`:wat::kernel::Listener'`).
/// `listener'` wraps its mechanism here:
/// - Thread tier: `CrossbeamListener { rx }` (the rendezvous receiver).
/// - Process tier: `SocketListener { listener }` (the bound non-blocking UDS).
///
/// `accept'` downcasts the opaque to `Listener`, calls `inner.accept(sym, span)`,
/// wraps the returned `Peer` as a `PEER_TYPE_PATH` opaque, and returns it.
pub struct Listener {
    pub(crate) inner: Box<dyn CommListener>,
}

impl Listener {
    /// Construct a thread-tier listener from the rendezvous receiver.
    pub fn from_crossbeam(rx: crate::comms::thread::Receiver<Value>) -> Self {
        Listener { inner: Box::new(CrossbeamListener { rx }) }
    }

    /// Construct a process-tier listener from the bound `UnixListener`.
    ///
    /// Arc 209 C0b.3b-b — BIRTH-SEED: `{getppid()}` = the owner. `getppid()` in the service
    /// child IS the spawner — spawn is clone3-direct (no CLONE_PARENT; clone.rs:388) and
    /// `run_forms_as_server_child` runs the body in that child (spawn.rs:632). Dissolves the
    /// bootstrap circularity → the gate is LIVE from construction.
    pub fn from_socket(listener: UnixListener) -> Self {
        let owner_pid = unsafe { libc::getppid() };
        let mut seed = HashSet::new();
        seed.insert(owner_pid);
        Listener { inner: Box::new(SocketListener { listener, allowed_pids: Mutex::new(seed) }) }
    }

    /// Dispatch accept to the concrete impl; wrap the returned `Peer` as a
    /// `PEER_TYPE_PATH` opaque `Value` for the eval layer.
    pub fn accept_as_value(&self, sym: &SymbolTable, span: &Span) -> Result<Value, EvalBreak> {
        let peer = self.inner.accept(sym, span)?;
        use crate::kernel::spawn::PEER_TYPE_PATH;
        Ok(make_rust_opaque(
            PEER_TYPE_PATH,
            Arc::new(ThreadOwnedCell::new(Some(peer))),
        ))
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comms::process::PeerCred;

    fn listener_with(pids: &[i32]) -> SocketListener {
        // Bind a throwaway abstract UDS just to own a UnixListener; seed the set directly.
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::SocketAddr;
        let sa = SocketAddr::from_abstract_name(b"wat.arc209.c0b3bb.unit").unwrap();
        let listener = UnixListener::bind_addr(&sa).unwrap();
        SocketListener { listener, allowed_pids: Mutex::new(pids.iter().copied().collect()) }
    }

    #[test]
    fn authorizes_only_my_uid_and_an_allowed_pid() {
        let me = unsafe { libc::geteuid() };
        let mine = std::process::id() as i32;
        let sl = listener_with(&[]);
        assert!(!sl.authorizes(&PeerCred { pid: mine, uid: me, gid: 0 })); // empty set → no
        sl.allow(mine);
        assert!(sl.authorizes(&PeerCred { pid: mine, uid: me, gid: 0 })); // allowed → yes
        assert!(!sl.authorizes(&PeerCred { pid: mine + 999_999, uid: me, gid: 0 })); // wrong pid
        assert!(!sl.authorizes(&PeerCred { pid: mine, uid: me + 1, gid: 0 })); // wrong uid → no
    }
}
