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
    /// Arc 272 6c.2 — the pid of the process that autobind-minted this address, stamped at
    /// `listener'` time via `getpid()`. Rides the capability by value as a record field; the
    /// connect gate verifies the kernel-vouched `SO_PEERCRED` answerer pid against it.
    pub(crate) minter_pid: i32,
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
        // Arc 272 6c.2 — MUTUAL UDS peer-cred via the powerbox: the CLIENT verifies the SERVER's
        // kernel-vouched identity through `CommsPolicy::OnlyThisPeer`, symmetric with the accept
        // gate's `OnlyMyPeers` check in `kernel/listener.rs`. The SO_PEERCRED uid+pid checks ARE
        // the security; the autobind name is an exclusive-bind rendezvous token (kernel-minted,
        // not a chosen name), not a secret. The connect gate verifies the kernel-vouched answerer
        // pid against the minter pid stamped in the address capability at autobind time. Read
        // peer_cred BEFORE `OwnedFd::from(stream)` consumes the stream.
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
            if !connect_admits(&server, me, self.minter_pid) {
                return Err(RuntimeError {
                    span: span.clone(),
                    kind: RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!(
                            "comms policy (only-this-peer) refused the connection — \
                             server pid {} != minter pid {}, or server euid {} != our euid {} \
                             (the answerer must be the exact process that minted this address)",
                            server.pid, self.minter_pid, server.uid, me
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
/// Returns `true` when `CommsPolicy::OnlyThisPeer { pid: minter_pid }` admits `server`
/// for a caller whose effective uid is `euid`. That means: `server.uid == euid AND
/// server.pid == minter_pid`. False → the connection is refused by `SocketAddress::connect`.
///
/// The connect gate verifies the kernel-vouched answerer pid against the minter pid stamped
/// in the address capability at autobind time — symmetric with the accept gate's
/// `OnlyMyPeers` pid-set check in `kernel::listener`.
///
/// Extracted so a regression test can drive it with SYNTHESIZED `PeerCred` values
/// (no real socket, no fork, no privilege) — exactly parallel to
/// `kernel::listener::tests::authorizes_only_my_uid_and_an_allowed_pid`.
pub(crate) fn connect_admits(
    server: &crate::comms::process::PeerCred,
    euid: u32,
    minter_pid: i32,
) -> bool {
    crate::capability::CommsPolicy::OnlyThisPeer { pid: minter_pid }.admits(server, euid)
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

    /// Construct a process-tier address from the RAW abstract-namespace name bytes and the
    /// minter pid. Arc 272 6c.2: the autobind path stamps `getpid()` so the connect gate can
    /// verify the kernel-vouched answerer pid against the minter.
    pub fn from_socket_name_bytes(name: Vec<u8>, minter_pid: i32) -> Self {
        Address { inner: Box::new(SocketAddress { name, minter_pid }) }
    }

    /// Arc 272 6c.2 — the PORTABLE form of this address: `(minter_pid, name_bytes)`, IF it is
    /// a process-tier socket address. A process-tier address is a true capability: its name bytes
    /// and minter pid are meaningful across a process boundary, so it may cross the IPC wire (as a
    /// `#wat-edn.cap/address` `#wat.kernel/SocketAddressWire` record). A thread-tier address (a
    /// crossbeam `Sender`) has NO portable form — it is in-memory, same-process only — so this
    /// returns `None` and the address falls to the opaque (non-portable) wire path.
    pub(crate) fn portable_form(&self) -> Option<(i32, Vec<u8>)> {
        self.inner
            .as_any_ref()
            .downcast_ref::<SocketAddress>()
            .map(|s| (s.minter_pid, s.name.clone()))
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
    /// Arc 272 6c.2: the gate now checks `OnlyThisPeer { pid: minter_pid }` — exact pid AND
    /// same euid. Mirrors `kernel::listener::tests::authorizes_only_my_uid_and_an_allowed_pid`
    /// (the accept gate's parity test). A refactor that drops or weakens the `connect_admits`
    /// consult will redden this test.
    #[test]
    fn connect_admits_exact_pid_admitted_wrong_pid_or_uid_refused() {
        let my_euid: u32 = 1000;
        let minter_pid: i32 = 4242;

        // Exact pid + same uid → admitted.
        let exact = PeerCred { pid: minter_pid, uid: my_euid, gid: 0 };
        assert!(
            connect_admits(&exact, my_euid, minter_pid),
            "exact minter pid + same euid must be admitted by OnlyThisPeer"
        );

        // Same uid, wrong pid → refused.
        let wrong_pid = PeerCred { pid: minter_pid + 1, uid: my_euid, gid: 0 };
        assert!(
            !connect_admits(&wrong_pid, my_euid, minter_pid),
            "same uid but wrong pid must be refused by OnlyThisPeer"
        );

        // Right pid, different uid → refused.
        let wrong_uid = PeerCred { pid: minter_pid, uid: my_euid + 1, gid: 0 };
        assert!(
            !connect_admits(&wrong_uid, my_euid, minter_pid),
            "right pid but different uid must be refused by OnlyThisPeer"
        );
    }
}
