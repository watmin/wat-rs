//! # Kernel address entity — Arc 209 Stone C0b.2e-iii
//!
//! Introduces `CommAddress` (a kernel trait with a single `connect` method)
//! and the `Address { inner: Box<dyn CommAddress> }` entity — a proper
//! first-class type under `ADDRESS_TYPE_PATH`, replacing:
//!
//! - The thread-tier fiction of a raw `comms::thread::Sender` called "Address'"
//! - The process-tier `SocketAddress'` opaque (a String name under its own path)
//!
//! Both tiers now produce the same `Address` entity; `connect'` collapses to
//! one arm that downcasts the opaque and calls `inner.connect(sym, span)`.
//!
//! ## Layering
//!
//! `kernel → comms` (crossbeam + process) and `kernel → runtime` (for `Value`,
//! `EvalBreak`, `SymbolTable`) — same kernel→runtime direction already used by
//! `kernel::listener`. No `comms → kernel` direction; no cycle.
//!
//! ## What is NOT here (by design)
//!
//! - `reactor_class` / `as_any` methods on `CommAddress` — an address is
//!   *dialed*, never *poll'd*; those discriminants have no consumer here.
//! - Any remote `CommAddress` impl — organic future addition (a new impl,
//!   zero central edit per the open-trait decision; `remote` is perpetually-
//!   awaiting-definition, unbuilt on purpose).
//!
//! [[feedback_dont_build_the_forcing_function]]
//! [[feedback_vended_primitives_never_deadlock]]

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::kernel::peer::Peer;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::span::Span;

// ─── CommAddress trait ────────────────────────────────────────────────────────

/// Transport-blind kernel trait: dial this address and return the
/// connected client-side `Peer`.
///
/// # Deadlock contract
///
/// Each impl preserves its transport's exact dial semantics:
/// - Crossbeam (`ThreadAddress`): rendezvous `typed_send` (blocks until
///   the server's `accept'` is ready to receive the connect-request).
/// - Socket (`SocketAddress`): non-blocking `UnixStream::connect_addr`
///   on the abstract-namespace UDS — returns immediately with the stream.
///
/// [[feedback_vended_primitives_never_deadlock]]
pub trait CommAddress: Send + Sync {
    /// Dial this address; return the connected client-side Peer.
    fn connect(&self, sym: &SymbolTable, span: &Span) -> Result<Peer, EvalBreak>;

    /// Return a `&dyn Any` reference for downcasting to the concrete impl.
    ///
    /// Used by `listener'` (process tier) to extract the `SocketAddress`'s
    /// abstract-namespace name for `UnixListener::bind_addr`.
    fn as_any_ref(&self) -> &dyn std::any::Any;
}

// ─── ThreadAddress ────────────────────────────────────────────────────────────

/// Thread-tier `CommAddress`: the rendezvous sender from `listener'`.
///
/// `connect` mints two crossbeam pairs (req/resp), wraps the client `Peer'`
/// end locally, then ships the server's raw halves `(req_rx, resp_tx)` over
/// the rendezvous `tx` — blocking until the server's `accept'` is ready.
///
/// Verbatim body from the former thread arm of `eval_connect_prime`.
pub struct ThreadAddress {
    pub(crate) tx: crate::comms::thread::Sender<Value>,
}

impl CommAddress for ThreadAddress {
    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }

    fn connect(&self, sym: &SymbolTable, span: &Span) -> Result<Peer, EvalBreak> {
        const OP: &str = ":wat::kernel::connect'";
        // Mint the two connection pairs.
        // req: client sends (S) → server receives
        // resp: server sends (R) → client receives
        let (req_tx, req_rx) = crate::comms::thread::pair::<Value>();
        let (resp_tx, resp_rx) = crate::comms::thread::pair::<Value>();
        // Wrap the client Peer' end on THIS thread (custody holds).
        let client_peer = Peer::from_thread(req_tx, resp_rx);
        // Build the connect-request: the server's raw halves packed as a Value::Tuple.
        let connect_req = Value::Tuple(Arc::new(vec![
            crate::channel::receiver_from_comms(req_rx),
            crate::channel::sender_from_comms(resp_tx),
        ]));
        // Wrap self.tx as a SenderInner to call typed_send.
        // The closed flag is local; this wrapper is ephemeral (not stored).
        let sender_inner = crate::channel::inner::SenderInner::Comms {
            sender: self.tx.clone(),
            closed: AtomicBool::new(false),
        };
        // Ship the connect-request one-way over the rendezvous (no return leg).
        match crate::channel::typed_send(
            &sender_inner,
            connect_req,
            sym.types().map(|a| a.as_ref()),
            span.clone(),
        ) {
            crate::channel::SendOutcome::Ok => {}
            crate::channel::SendOutcome::Disconnected => {
                return Err(RuntimeError {
                    span: span.clone(),
                    kind: RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "connect': rendezvous send failed — listener was dropped".into(),
                    },
                }
                .into());
            }
        }
        Ok(client_peer)
    }
}

// ─── SocketAddress ────────────────────────────────────────────────────────────

/// Process-tier `CommAddress`: the abstract-namespace UDS name.
///
/// `connect` calls `UnixStream::connect_addr` on the abstract name, then
/// wraps the stream as a `Peer` via `Peer::from_socket`.
///
/// Verbatim body from the former socket arm of `eval_connect_prime`.
pub struct SocketAddress {
    /// The abstract-namespace UDS name as RAW BYTES. Arc 272: an autobind address is
    /// kernel-minted (5 random bytes), NOT UTF-8 — a `String` would corrupt it. The name is
    /// always the kernel-minted autobind bytes; the legacy UTF-8 `socket-address'` path was
    /// annihilated in arc 272 step 5.
    pub(crate) name: Vec<u8>,
}

impl CommAddress for SocketAddress {
    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }

    fn connect(&self, _sym: &SymbolTable, span: &Span) -> Result<Peer, EvalBreak> {
        const OP: &str = ":wat::kernel::connect'";
        use std::os::fd::OwnedFd;
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::{SocketAddr, UnixStream};
        let sa = SocketAddr::from_abstract_name(&self.name).map_err(|e| RuntimeError {
            span: span.clone(),
            kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("abstract addr for connect: {}", e),
            },
        })?;
        let stream = UnixStream::connect_addr(&sa).map_err(|e| RuntimeError {
            span: span.clone(),
            kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("connect abstract UDS: {}", e),
            },
        })?;
        // Arc 272 v4 / step 5 — MUTUAL UDS peer-cred via the powerbox: the CLIENT verifies the
        // SERVER's kernel-vouched identity through the SAME `CommsPolicy` the accept gate consults
        // (kernel/listener.rs `authorizes`); both gates route through one mediator. The connect side
        // stands on `AnyOfMyUser` (the euid floor) — but post-step-5 the address is ALWAYS an
        // unguessable autobind capability handed over the lineage channel (guessable names are
        // annihilated; abstract names are exclusive-bind), so the answerer IS the lineage minter:
        // holding the capability is the lineage proof, the euid floor is defense-in-depth. (6c.2 — a
        // per-Address minter-pid verified right here — is the belt-and-suspenders for the narrow
        // leak-then-rebind edge; deferred, not load-bearing.) Read peer_cred BEFORE
        // `OwnedFd::from(stream)` consumes the stream.
        {
            use std::os::fd::AsRawFd;
            let server = crate::comms::process::peer_cred(stream.as_raw_fd()).map_err(|e| RuntimeError {
                span: span.clone(),
                kind: RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("mutual UDS peer-cred: peer_cred on the server socket: {}", e),
                },
            })?;
            // SAFETY: geteuid() is always-succeeds, no args, no memory effects.
            let me = unsafe { libc::geteuid() };
            if !connect_admits(&server, me) {
                return Err(RuntimeError {
                    span: span.clone(),
                    kind: RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!(
                            "comms policy (any-of-my-user) refused the connection — server euid {} \
                             != our euid {} (the answerer is not a process of our user)",
                            server.uid, me
                        ),
                    },
                }
                .into());
            }
        }
        // Arc 258.5b-ii: reinterpret Sender<Value> as Sender<String> — eval pre-encodes.
        let (tx, rx) =
            crate::comms::process::sender_receiver_from_fd::<Value>(OwnedFd::from(stream))
                .map_err(|e| RuntimeError {
                    span: span.clone(),
                    kind: RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!("wrap socket stream failed: {}", e),
                    },
                })?;
        Ok(Peer::from_socket(tx.reinterpret::<String>(), rx))
    }
}

// ─── Connect-gate seam ───────────────────────────────────────────────────────

/// The connect-gate policy consult — a single named seam so the comms-policy decision
/// is one tested, located place rather than inlined at the call site.
///
/// Returns `true` when `CommsPolicy::AnyOfMyUser` admits `server` for a caller whose
/// effective uid is `euid` (i.e. `server.uid == euid`). False → the connection is
/// refused by `SocketAddress::connect`.
///
/// Extracted so a regression test can drive it with SYNTHESIZED `PeerCred` values
/// (no real socket, no fork, no privilege) — exactly parallel to
/// `kernel::listener::tests::authorizes_only_my_uid_and_an_allowed_pid`.
pub(crate) fn connect_admits(server: &crate::comms::process::PeerCred, euid: u32) -> bool {
    crate::capability::CommsPolicy::AnyOfMyUser.admits(server, euid)
}

// ─── Address entity ───────────────────────────────────────────────────────────

/// The unified, transport-blind address entity.
///
/// Stored as a `RustOpaque` under `ADDRESS_TYPE_PATH` (`:wat::kernel::Address'`).
/// Produced by:
/// - `listener'` (process): autobind via `Address::from_socket_name_bytes` — kernel-minted
///   abstract UDS name, never a user-chosen string (arc 272 step 5 annihilated `socket-address'`).
/// - `listener'` (thread): `Address{ inner: Box::new(ThreadAddress{tx}) }` for
///   the Address' tuple slot (was a bare `Sender` fiction).
///
/// `connect'` downcasts the opaque to `Address`, calls `inner.connect(sym, span)`,
/// wraps the returned `Peer` as a `PEER_TYPE_PATH` opaque, and returns it.
pub struct Address {
    pub(crate) inner: Box<dyn CommAddress>,
}

impl Address {
    /// Construct a thread-tier address from the rendezvous sender.
    pub fn from_thread(tx: crate::comms::thread::Sender<Value>) -> Self {
        Address { inner: Box::new(ThreadAddress { tx }) }
    }

    /// Construct a process-tier address from the RAW abstract-namespace name bytes.
    /// Arc 272: the autobind path — the kernel-minted name (binary, not UTF-8) is the
    /// capability `connect'` dials.
    pub fn from_socket_name_bytes(name: Vec<u8>) -> Self {
        Address { inner: Box::new(SocketAddress { name }) }
    }

    /// Arc 272 6a-i — the PORTABLE wire bytes of this address, IF it is a process-tier socket
    /// address (the kernel-minted name). A process-tier address is a true capability: its name
    /// bytes are meaningful across a process boundary, so it may cross the IPC wire (as a
    /// `#wat-edn.cap/address` tag). A thread-tier address (a crossbeam `Sender`) has NO portable
    /// form — it is in-memory, same-process only — so this returns `None` and the address falls to
    /// the opaque (non-portable) wire path. The `Address` owns this knowledge so the wire layer
    /// never reaches into the concrete `CommAddress` impls.
    pub(crate) fn portable_name_bytes(&self) -> Option<Vec<u8>> {
        self.inner
            .as_any_ref()
            .downcast_ref::<SocketAddress>()
            .map(|s| s.name.clone())
    }

    /// Dispatch connect to the concrete impl; wrap the returned `Peer` as a
    /// `PEER_TYPE_PATH` opaque `Value` for the eval layer.
    pub fn connect_as_value(
        &self,
        sym: &SymbolTable,
        span: &Span,
    ) -> Result<Value, EvalBreak> {
        let peer = self.inner.connect(sym, span)?;
        use crate::kernel::spawn::PEER_TYPE_PATH;
        use crate::rust_deps::custodia::ThreadOwnedCell;
        use crate::rust_deps::marshal::make_rust_opaque;
        Ok(make_rust_opaque(
            PEER_TYPE_PATH,
            Arc::new(ThreadOwnedCell::new(Some(peer))),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comms::process::PeerCred;

    /// Regression guard: the connect gate's policy consult is exercised with SYNTHESIZED peer
    /// credentials — no real socket, no fork, no privilege required.
    ///
    /// Mirrors `kernel::listener::tests::authorizes_only_my_uid_and_an_allowed_pid` (the accept
    /// gate's parity test). A refactor that drops or weakens the `connect_admits` consult will
    /// redden this test.
    #[test]
    fn connect_admits_same_euid_admitted_different_euid_refused() {
        let my_euid: u32 = 1000;

        // Same uid as caller → admitted.
        let same_user = PeerCred { pid: 42, uid: my_euid, gid: 0 };
        assert!(
            connect_admits(&same_user, my_euid),
            "a peer with the same euid must be admitted by AnyOfMyUser"
        );

        // Different uid → refused.
        let foreign_user = PeerCred { pid: 42, uid: my_euid + 1, gid: 0 };
        assert!(
            !connect_admits(&foreign_user, my_euid),
            "a peer with a different euid must be refused by AnyOfMyUser"
        );
    }
}
