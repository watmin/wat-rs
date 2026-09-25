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

/// Arc 278 peer-lifecycle Strike 4 — the connect' OUTCOME WALL (the LAST peer wall). A
/// `CommAddress::connect` distinguishes its *handleable* failures (the ones `connect'`
/// converts to a matchable `:wat::kernel::ConnectOutcome<S,R>` variant, never a raise)
/// from the must-never-happen raises (which stay `EvalBreak`). `connect` returns
/// `Result<Result<Peer, ConnectFail>, EvalBreak>`: the outer `Err` is an uncatchable raise
/// (an in-process substrate bug — a malformed abstract-name, an arity/type mismatch), the
/// inner `Err(ConnectFail)` is a handleable outcome the eval layer maps to
/// `Closed`/`Undialable`/`WrongPeer`/`Failed`. The exact TWIN of `AcceptFail`
/// (`kernel/listener.rs`).
pub enum ConnectFail {
    /// Nothing is listening at this address, and it is gone for good. Maps to
    /// `ConnectOutcome::Closed[cause]`. The thread arm uses this when the live
    /// rendezvous send fails because the listener was dropped. The process arm
    /// uses this when `connect_addr` returns `ECONNREFUSED` or `ENOENT`. An
    /// autobind name is kernel-minted and nothing rebinds it, so the listener
    /// does not come back at this address. The cause is the sentence that
    /// producer already built; every consuming arm binds it.
    Closed(String),
    /// This address value cannot be dialed here: an inert wire copy. Produced
    /// only by the thread locus (`tx` is `None`). No process locus produces
    /// this. Maps to `ConnectOutcome::Undialable[cause]`.
    Undialable(String),
    /// The answerer is not who the address names. Produced only by the process
    /// locus (`OnlyThisPeer`). No thread locus produces this. Maps to
    /// `ConnectOutcome::WrongPeer[cause]`.
    WrongPeer(String),
    /// A transport io failure: `peer_cred`, socket wrap, or a `connect_addr`
    /// error that is not the gone-for-good fact. Produced only by the process
    /// locus. No thread locus produces this. Maps to
    /// `ConnectOutcome::Failed[cause <- Failure]` (via `message_only_failure`).
    Failed(String),
}

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
    ///
    /// Arc 278 the connect' OUTCOME WALL: `Ok(Ok(peer))` = dialed + admitted;
    /// `Ok(Err(ConnectFail))` = a handleable failure (→ `Closed`/`Undialable`/`WrongPeer`/`Failed`);
    /// `Err(EvalBreak)` = a must-never-happen raise (a malformed abstract-name substrate
    /// bug — the address's own name failed `from_abstract_name`).
    fn connect(&self, sym: &SymbolTable, span: &Span)
        -> Result<Result<Peer, ConnectFail>, EvalBreak>;

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
/// Verbatim body from the former thread arm of `eval_connect_prime`
/// (now `src/kernel/resource.rs`).
pub struct ThreadAddress {
    /// `Some` only on the value `listener'` minted in this process. A copy
    /// rebuilt from `ThreadAddressWire` is `None`: the wire form is inert.
    pub(crate) tx: Option<Arc<crate::comms::thread::Sender<Value>>>,
    pub(crate) wire: ThreadWire,
}

/// What a thread address carries so `ThreadAddressWire` stays a well-formed
/// record. `minter_pid` is the process that minted the live value. `id` is
/// `0` on a mint: the codec's record has two i64 fields, and nothing looks
/// an id up. No nonce and no token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ThreadWire {
    pub(crate) minter_pid: i32,
    pub(crate) id: u64,
}

/// `connect` on a decoded thread address. One sentence for every such copy:
/// the minter's echo and a foreign process say the same thing.
const THREAD_ADDRESS_WIRE_IS_INERT: &str =
    "a thread address is dialable only through the live value; one that crossed a wire is inert.";

fn current_pid() -> i32 {
    // SAFETY: getpid() takes no arguments and does not fail.
    unsafe { libc::getpid() }
}

impl CommAddress for ThreadAddress {
    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }

    fn connect(&self, sym: &SymbolTable, span: &Span)
        -> Result<Result<Peer, ConnectFail>, EvalBreak> {
        let rendezvous = match &self.tx {
            Some(tx) => Arc::clone(tx),
            // Undialable. A wire copy never becomes the live value. The sentence
            // does not claim a wrong process: the minter's own echo is inert too.
            // No process locus produces this fact.
            None => {
                return Ok(Err(ConnectFail::Undialable(
                    THREAD_ADDRESS_WIRE_IS_INERT.to_string(),
                )));
            }
        };
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
            sender: (*rendezvous).clone(),
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
            // Arc 278 the connect' OUTCOME WALL — HANDLEABLE: the rendezvous is gone
            // (the listener was dropped / never accepted). Nothing installs a new
            // receiver on this sender, so the listener is gone for good →
            // ConnectOutcome::Closed, not a raise the dialer unwinds past. The
            // thread-tier twin of the process tier's ECONNREFUSED.
            crate::channel::SendOutcome::Disconnected => {
                return Ok(Err(ConnectFail::Closed(
                    "connect: rendezvous send failed — listener was dropped (no listener)".into(),
                )));
            }
        }
        Ok(Ok(client_peer))
    }
}

// ─── SocketAddress ────────────────────────────────────────────────────────────

/// Process-tier `CommAddress`: the abstract-namespace UDS name.
///
/// `connect` calls `UnixStream::connect_addr` on the abstract name, then
/// wraps the stream as a `Peer` via `Peer::from_socket`.
///
/// Verbatim body from the former socket arm of `eval_connect_prime`
/// (now `src/kernel/resource.rs`).
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

    fn connect(&self, _sym: &SymbolTable, span: &Span)
        -> Result<Result<Peer, ConnectFail>, EvalBreak> {
        const OP: &str = ":wat::kernel::connect";
        use std::os::fd::OwnedFd;
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::{SocketAddr, UnixStream};
        // Arc 278 the connect' OUTCOME WALL — MUST-NEVER-HAPPEN raise (STAYS a raise, the
        // outer `Err(EvalBreak)`). `self.name` is either kernel-minted (autobind, 5 random
        // bytes) or a wire-received `SocketAddressWire` already fully validated at decode to
        // the abstract-UDS constraint (non-empty, <=107 bytes, bytes 0..=255 —
        // `capability::registry::socket_address_wire_from_record`), so a `from_abstract_name`
        // failure here is an in-process substrate bug, NOT adversarial wire data (STOP-3,
        // grounded — the accept' malformed-connect-request precedent).
        let sa = SocketAddr::from_abstract_name(&self.name).map_err(|e| RuntimeError::new(span.clone(), RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("abstract addr for connect: {}", e),
            }))?;
        // Arc 278 the connect' OUTCOME WALL — HANDLEABLE. Measured on this locus:
        // an absent abstract name yields ECONNREFUSED (111), not ENOENT. ENOENT is
        // still the gone-for-good fact when a path name is missing. Both map to
        // Closed. Any other connect_addr error (the dial reached the kernel and
        // failed for a reason other than "nothing is there") is Failed. A blocking
        // connect waits out a full backlog, so this arm does not see EAGAIN.
        // An autobind name is kernel-minted and nothing rebinds it: ECONNREFUSED
        // here means the listener is gone for good.
        let stream = match UnixStream::connect_addr(&sa) {
            Ok(s) => s,
            Err(e) => {
                let msg = format!("connect abstract UDS: {}", e);
                let gone = matches!(e.raw_os_error(), Some(libc::ECONNREFUSED | libc::ENOENT));
                return Ok(Err(if gone {
                    ConnectFail::Closed(msg)
                } else {
                    ConnectFail::Failed(msg)
                }));
            }
        };
        // Arc 272 6c.2 — MUTUAL UDS peer-cred via the powerbox: the CLIENT verifies the SERVER's
        // kernel-vouched identity through `CommsPolicy::OnlyThisPeer`, symmetric with the accept
        // gate's `OnlyMyPeers` check in `kernel/listener.rs`. The SO_PEERCRED uid+pid checks ARE
        // the security; the autobind name is an exclusive-bind rendezvous token (kernel-minted,
        // not a chosen name), not a secret. The connect gate verifies the kernel-vouched answerer
        // pid against the minter pid stamped in the address capability at autobind time. Read
        // peer_cred BEFORE `OwnedFd::from(stream)` consumes the stream.
        {
            use std::os::fd::AsRawFd;
            // Arc 278 the connect' OUTCOME WALL — HANDLEABLE: reading the server's
            // peer-cred failed (io error) → ConnectOutcome::Failed[cause].
            let server = match crate::comms::process::peer_cred(stream.as_raw_fd()) {
                Ok(c) => c,
                Err(e) => {
                    return Ok(Err(ConnectFail::Failed(format!(
                        "mutual UDS peer-cred: peer_cred on the server socket: {}",
                        e
                    ))));
                }
            };
            // SAFETY: geteuid() is always-succeeds, no args, no memory effects.
            let me = unsafe { libc::geteuid() };
            // Arc 278 the connect' OUTCOME WALL — HANDLEABLE: the `OnlyThisPeer` identity
            // check failed (the answerer is not the exact process that minted this address)
            // → ConnectOutcome::WrongPeer[cause]. Process locus only. This FIRES here
            // (unlike accept', where the gate bounces the stranger internally) — the
            // client dials once and a server-identity mismatch is a caller-visible outcome.
            if !connect_admits(&server, me, self.minter_pid) {
                return Ok(Err(ConnectFail::WrongPeer(format!(
                    "comms policy (only-this-peer) refused the connection — \
                     server pid {} != minter pid {}, or server euid {} != our euid {} \
                     (the answerer must be the exact process that minted this address)",
                    server.pid, self.minter_pid, server.uid, me
                ))));
            }
        }
        // Arc 258.5b-ii: reinterpret Sender<Value> as Sender<String> — eval pre-encodes.
        // Arc 278 the connect' OUTCOME WALL — HANDLEABLE: wrapping the connected stream
        // failed (io error) → ConnectOutcome::Failed[cause].
        let (tx, rx) =
            match crate::comms::process::sender_receiver_from_fd::<Value>(OwnedFd::from(stream)) {
                Ok(pair) => pair,
                Err(e) => {
                    return Ok(Err(ConnectFail::Failed(format!(
                        "wrap socket stream failed: {}",
                        e
                    ))));
                }
            };
        Ok(Ok(Peer::from_socket(tx.reinterpret::<String>(), rx)))
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
/// Stored as a `RustOpaque` under `ADDRESS_TYPE_PATH` (`:wat::kernel::Address`).
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
        // `id` stays on the record so the codec's two-field shape is unchanged.
        // Nothing resolves it, so the mint stamps 0.
        let wire = ThreadWire { minter_pid: current_pid(), id: 0 };
        Address {
            inner: Box::new(ThreadAddress {
                tx: Some(Arc::new(tx)),
                wire,
            }),
        }
    }

    /// Reconstruct a thread address from its wire stamp. The capability codec
    /// is the only caller. The result is always undialable: `tx` is `None`.
    pub(crate) fn from_thread_wire(minter_pid: i32, id: u64) -> Self {
        let wire = ThreadWire { minter_pid, id };
        Address { inner: Box::new(ThreadAddress { tx: None, wire }) }
    }

    /// The thread-tier portable form, `(minter_pid, id)`. Not
    /// [`Address::portable_form`]: that answer is what `address-wire?` reads
    /// as "a process may dial this", which a thread address is not.
    pub(crate) fn thread_portable_form(&self) -> Option<(i32, u64)> {
        self.inner
            .as_any_ref()
            .downcast_ref::<ThreadAddress>()
            .map(|t| (t.wire.minter_pid, t.wire.id))
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
    /// `#wat.kernel/Address` `#wat.kernel/SocketAddressWire` record). A thread-tier address (a
    /// crossbeam `Sender`) has NO portable form — it is in-memory, same-process only — so this
    /// returns `None`. A thread address has [`Address::thread_portable_form`]
    /// instead, and still falls through this door so `address-wire?` stays false.
    pub(crate) fn portable_form(&self) -> Option<(i32, Vec<u8>)> {
        self.inner
            .as_any_ref()
            .downcast_ref::<SocketAddress>()
            .map(|s| (s.minter_pid, s.name.clone()))
    }

    /// Dispatch connect to the concrete impl and build the matchable
    /// `:wat::kernel::ConnectOutcome<S,R>` `Value` for the eval layer (Arc 278 the connect'
    /// OUTCOME WALL — the LAST peer wall). `Ok(peer)` → `Connected[peer]` (the `Peer`
    /// wrapped as a `PEER_TYPE_PATH` opaque); `Err(ConnectFail::Closed(reason))` →
    /// `Closed[cause]`; `Err(ConnectFail::Undialable(reason))` → `Undialable[cause]`;
    /// `Err(ConnectFail::WrongPeer(reason))` → `WrongPeer[cause]`;
    /// `Err(ConnectFail::Failed(reason))` → `Failed[cause <- Failure]` (all via
    /// `message_only_failure`). A must-never-happen raise stays an `EvalBreak` (the `?`).
    pub fn connect_as_value(
        &self,
        sym: &SymbolTable,
        span: &Span,
    ) -> Result<Value, EvalBreak> {
        use crate::kernel::spawn::PEER_TYPE_PATH;
        use crate::rust_deps::custodia::ThreadOwnedCell;
        use crate::rust_deps::marshal::make_rust_opaque;
        match self.inner.connect(sym, span)? {
            Ok(peer) => {
                let peer_val = make_rust_opaque(
                    PEER_TYPE_PATH,
                    Arc::new(ThreadOwnedCell::new(Some(peer))),
                );
                Ok(crate::kernel::outcome::connect_outcome_connected(peer_val))
            }
            Err(ConnectFail::Closed(reason)) => Ok(crate::kernel::outcome::connect_outcome_closed(reason)),
            Err(ConnectFail::Undialable(reason)) => Ok(crate::kernel::outcome::connect_outcome_undialable(reason)),
            Err(ConnectFail::WrongPeer(reason)) => Ok(crate::kernel::outcome::connect_outcome_wrong_peer(reason)),
            Err(ConnectFail::Failed(reason)) => Ok(crate::kernel::outcome::connect_outcome_failed(reason)),
        }
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

/// Stone 255.29 / 255.31 — a thread address is data, and a decoded copy is inert.
#[cfg(test)]
mod thread_address_is_data {
    use super::*;
    use crate::kernel::listener::Listener;
    use crate::kernel::spawn::ADDRESS_TYPE_PATH;
    use crate::rust_deps::marshal::make_rust_opaque;

    fn world() -> crate::freeze::FrozenWorld {
        crate::freeze::startup_from_source(
            "",
            None,
            Arc::new(crate::load::loader::InMemoryLoader::new()),
        )
        .expect("empty world")
    }

    fn addr_of(v: &Value) -> &Address {
        match v {
            Value::RustOpaque(inner) => inner
                .payload
                .downcast_ref::<Address>()
                .expect("an Address opaque"),
            _ => panic!("expected an Address opaque, got {:?}", v.type_name()),
        }
    }

    fn outcome(r: Result<Result<Peer, ConnectFail>, EvalBreak>) -> String {
        match r {
            Ok(Ok(_)) => "Connected".into(),
            Ok(Err(ConnectFail::Closed(m))) => format!("Closed: {m}"),
            Ok(Err(ConnectFail::Undialable(m))) => format!("Undialable: {m}"),
            Ok(Err(ConnectFail::WrongPeer(m))) => format!("WrongPeer: {m}"),
            Ok(Err(ConnectFail::Failed(m))) => format!("Failed: {m}"),
            Err(_) => "RAISE".into(),
        }
    }

    #[test]
    fn a_thread_address_round_trips_through_the_trusted_door() {
        let w = world();
        let sym = &w.symbols;
        let types = Some(&w.types);
        let (tx, rx) = crate::comms::thread::pair::<Value>();
        let _listener = Listener::from_crossbeam(rx);
        let addr_val = make_rust_opaque(ADDRESS_TYPE_PATH, Address::from_thread(tx));
        let (pid, id) = addr_of(&addr_val).thread_portable_form().expect("thread form");
        let payload = Value::Tuple(Arc::new(vec![Value::i64(7), addr_val]));
        let wire = crate::edn::render::value_to_edn_string_with(&payload, types).expect("encodes");
        assert_eq!(
            wire,
            format!(
                "[7 #wat.kernel/Address #wat.kernel/ThreadAddressWire {{:minter-pid {pid} :id {id}}}]"
            )
        );
        assert!(
            crate::edn::render::edn_string_to_value(&wire).is_err(),
            "general decode must refuse a capability tag"
        );
        let decoded =
            crate::edn::render::decode_trusted_wire(&wire, types, None).expect("trusted door");
        drop(payload);
        let d_addr = match &decoded {
            Value::Tuple(xs) | Value::Vec(xs) => addr_of(&xs[1]),
            other => panic!("tuple/vec expected, got {:?}", other.type_name()),
        };
        let span = crate::rust_caller_span!();
        assert_eq!(
            outcome(d_addr.inner.connect(sym, &span)),
            format!("Undialable: {THREAD_ADDRESS_WIRE_IS_INERT}")
        );
        assert!(d_addr.portable_form().is_none(), "address-wire? stays false");
        // The listener is still there. The decoded copy must not have dialed it.
        // A live dial on a freshly minted address still reaches an accept.
        let (tx2, rx2) = crate::comms::thread::pair::<Value>();
        let listener2 = Listener::from_crossbeam(rx2);
        let live = make_rust_opaque(ADDRESS_TYPE_PATH, Address::from_thread(tx2));
        assert_eq!(outcome(addr_of(&live).inner.connect(sym, &span)), "Connected");
        assert!(
            matches!(listener2.inner.accept(sym, &span), Ok(Ok(_))),
            "the live value still dials"
        );
    }

    #[test]
    fn a_same_process_wire_copy_is_inert_while_the_listener_lives() {
        let w = world();
        let sym = &w.symbols;
        let span = crate::rust_caller_span!();
        let (tx, rx) = crate::comms::thread::pair::<Value>();
        let listener = Listener::from_crossbeam(rx);
        let live = Address::from_thread(tx);
        let (pid, id) = live.thread_portable_form().expect("thread form");
        let copy = Address::from_thread_wire(pid, id);
        assert_eq!(
            outcome(copy.inner.connect(sym, &span)),
            format!("Undialable: {THREAD_ADDRESS_WIRE_IS_INERT}")
        );
        assert_eq!(outcome(live.inner.connect(sym, &span)), "Connected");
        assert!(
            matches!(listener.inner.accept(sym, &span), Ok(Ok(_))),
            "the live value still dials"
        );
    }

    #[test]
    fn a_foreign_wire_copy_is_inert() {
        let w = world();
        let sym = &w.symbols;
        let span = crate::rust_caller_span!();
        let (tx, _rx) = crate::comms::thread::pair::<Value>();
        let live = Address::from_thread(tx);
        let (pid, id) = live.thread_portable_form().expect("thread form");
        let other = Address::from_thread_wire(pid.wrapping_add(1), id);
        assert_eq!(
            outcome(other.inner.connect(sym, &span)),
            format!("Undialable: {THREAD_ADDRESS_WIRE_IS_INERT}")
        );
        assert!(live.portable_form().is_none());
    }

    /// Stone 255.34. A listener is this process; the address names a different
    /// minter pid. `SO_PEERCRED` is set at connect, so the gate runs without
    /// an accept. Pre-stone this same drive returned `Rejected`.
    #[test]
    fn a_live_connect_to_the_wrong_minter_pid_is_wrong_peer() {
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::{SocketAddr, UnixListener};
        let name = format!(
            "wat.255.34.{}.{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        );
        let sa = SocketAddr::from_abstract_name(name.as_bytes()).expect("abstract name");
        let listener = UnixListener::bind_addr(&sa).expect("bind");
        let w = world();
        let span = crate::rust_caller_span!();
        let server_pid = current_pid();
        let minter = server_pid.wrapping_add(1);
        let me = unsafe { libc::geteuid() };
        let addr = Address::from_socket_name_bytes(name.into_bytes(), minter);
        let got = outcome(addr.inner.connect(&w.symbols, &span));
        assert_eq!(
            got,
            format!(
                "WrongPeer: comms policy (only-this-peer) refused the connection — \
                 server pid {server_pid} != minter pid {minter}, or server euid {me} != our euid {me} \
                 (the answerer must be the exact process that minted this address)"
            )
        );
        drop(listener);
    }
}
