//! `:wat::kernel::` message intrinsics — arc 255 home #5 (255.1c-kernel-message).
//! Five verbs — `send`, `try-send`, `recv`, `select`, `poll` — all
//! `@Category Message`: deliver or receive a payload across a peer/channel
//! boundary to another locus (`wat/runtime-meta.wat:161–167`). The locus is
//! a TYPED VALUE (`peer<I,O>`) the caller already holds — contrast
//! `kernel_ambient.rs`'s four readers (`:Ambient`: process-global state no
//! caller-held value addresses) and `kernel_stdio.rs` (`:Io`: an ambient OS
//! stream with no caller-held handle).
//!
//! Every one of the five delegates to the SAME `crate::kernel::message::eval_*` fn
//! that already existed as a literal-match arm in `runtime.rs` — see
//! `kernel/mod.rs` for the tier-wide "bodies do not live here" claim this
//! home is an instance of.
//!
//! ## ★★ The first rows no gate can check
//!
//! These five have **no registered `TypeScheme`** — `check.rs` special-cases
//! each with a bespoke `infer_*_prime` arm (`:4049` `:4061` `:4069` `:4176`
//! `:4188` → `infer_send_prime` / `infer_try_send_prime` / `infer_recv_prime`
//! / `infer_select_prime` / `infer_poll_prime`) because the types are
//! **projective**: `I` flows from a held `peer<I,O>` into the payload arg
//! (`send`/`try-send`); `O` flows from `peer<I,O>` into the return
//! (`recv`/`select`/`poll`) — no fixed-arity scheme can express that.
//! `doc_arg_ret_types_match_checker_scheme` begins `None => continue, //
//! not yet in checker — skip`, so it SKIPS all five and goes green — that
//! green proves nothing about them
//! (`DESIGN-STONE-255.1c-kernel-message.md`, "★★ THE POINT").
//!
//! **No stub `TypeScheme`s were minted to manufacture gate coverage.** Each
//! row's `@arg`/`@ret` below is declared from what its `infer_*_prime` fn
//! actually produces, and each row carries a `//` (not `///` — see
//! `kernel_stdio.rs`'s note on which comments `render-doc` prints)
//! maintainer comment naming that fn as the real authority.
//!
//! ## `poll`'s purity — derived from the body, not assumed for symmetry
//!
//! `send`/`try-send`/`recv`/`select` unambiguously move or consume a
//! payload. `poll` was the one this stone deliberately left open: if it
//! only REPORTED READINESS and consumed nothing, it would have no
//! observable effect and would be `Pure` + `Nondeterministic` — a fifth
//! census entry beside the four `:Ambient` readers.
//!
//! Reading the body says otherwise. `eval_poll_prime` (`runtime.rs:33232`)
//! blocks on `sel.select()` (`runtime.rs:33463`), which for the thread tier
//! is `comms::thread::Select::select()`; whichever arm fires, that fn's
//! last move is `let result = selected_op.recv(ch)…` (`comms/thread.rs:400`)
//! — the SAME crossbeam consuming-receive primitive `recv'`/`select'`
//! themselves use, which DEQUEUES the value from the channel (an effect the
//! sending end can observe — e.g. it clears backpressure). The process tier
//! (`sel.select_raw()`, `runtime.rs:33615`) does the analogous consuming
//! read off the io_uring ring. The listener arm doesn't just notice a
//! pending connection either — it completes the accept and mints a new
//! `Peer'` (`crate::kernel::message::wrap_connect_request`, arc 109 Stone B).
//! None of poll's three arms (self-peer,
//! listener, client) is idle observation; every one drains something.
//!
//! So `poll` declares `@Purity Effectful` — the SAME answer as its four
//! siblings, but reached independently from `sel.select()` →
//! `selected_op.recv(ch)`, not assumed for symmetry with them. Because the
//! declaration agrees with `effectful_by_prefix`'s `:wat::kernel::` guess,
//! `poll` does NOT add a fifth entry to
//! `declared_purity_vs_effectful_by_prefix_census` — the census stays at
//! its four `:Ambient` entries from home #4.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::kernel::send peer payload)` → `:wat::kernel::SendOutcome`. Sends
/// `payload` (type `I`, projected from the held `peer<I,O>`) across the
/// peer/channel boundary. Never raises on a gone peer — the outcome is a
/// matchable `Sent`/`Closed`/`Lost` value (`SendOutcome`).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Message
/// @arg     peer (:wat::kernel::Peer :- [I O]) the peer/channel handle to send across
/// @arg     payload :I the payload; must unify with the peer's held I
/// @ret     :wat::kernel::SendOutcome Sent / Closed / Lost — never a raise
/// @example-norun (:wat::kernel::send my-peer "hi") #=> #wat.kernel/SendOutcome.Sent{}
// `//` not `///` — maintainer rationale, not user-facing prose (see the
// `render-doc` goldens note in `kernel_stdio.rs`'s `readln'` block).
//
// No registered `TypeScheme` for `send` — `check.rs`'s `infer_send_prime` (~10703)
// is the real type authority: `I` is projected out of `args[0]`'s peer<I,O> via
// `project_peer_io`, `payload` must unify with it, and the return is the fixed
// path `:wat::kernel::SendOutcome` (not projective on the return side, unlike
// `recv'`). The `@arg`/`@ret` above document that inference, not a scheme.
//
// Deciding line for `@Purity Effectful`: `runtime.rs:31092`,
// `Some(peer) => match peer.send(payload_val) { … }` — an attempted enqueue
// onto the peer's channel is the observable effect, whatever the outcome.
#[wat_intrinsic(":wat::kernel::send")]
pub(crate) fn eval_peer_send_prime(
    peer: &WatAST,
    payload: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::kernel::message::eval_peer_send_prime(&[peer.clone(), payload.clone()], list_span, env, sym)
}

/// `(:wat::kernel::try-send peer payload)` → `:wat::kernel::TrySendOutcome`.
/// Best-effort, NON-BLOCKING twin of `send` — same `(peer<I,O>, payload<-I)`
/// contract, but a full channel or a gone peer is a silent
/// `WouldBlock`/`Lost`, never a block and never a raise.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Message
/// @arg     peer (:wat::kernel::Peer :- [I O]) the peer/channel handle to send across
/// @arg     payload :I the payload; must unify with the peer's held I
/// @ret     :wat::kernel::TrySendOutcome Sent / WouldBlock / Closed / Lost
/// @example-norun (:wat::kernel::try-send my-peer "hi") #=> #wat.kernel/TrySendOutcome.Sent{}
// `//` not `///` — maintainer rationale (see `readln'`'s note in `kernel_stdio.rs`).
//
// No registered `TypeScheme` for `try-send` — `check.rs`'s `infer_try_send_prime`
// (~10788) is the real authority: same projective-I shape as `infer_send_prime`,
// its own `:wat::kernel::TrySendOutcome` return (NOT a reuse of SendOutcome —
// `WouldBlock` is a real outcome `send'` structurally cannot return).
//
// Deciding line for `@Purity Effectful`: `runtime.rs:31266`/`31276`,
// `peer.try_send_wire(wire)` / `peer.try_send(payload_val.clone())` — an
// attempted enqueue, same DOING as `send`, non-blocking semantics only.
#[wat_intrinsic(":wat::kernel::try-send")]
pub(crate) fn eval_peer_try_send_prime(
    peer: &WatAST,
    payload: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::kernel::message::eval_peer_try_send_prime(&[peer.clone(), payload.clone()], list_span, env, sym)
}

/// `(:wat::kernel::recv peer)` → `:wat::kernel::RecvOutcome<O>`. Blocks for
/// one payload (type `O`, projected from the held `peer<I,O>`) across the
/// peer/channel boundary. Never raises on close/crash — the outcome is a
/// matchable `Message`/`Closed`/`Lost`/`Shutdown` value.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Message
/// @arg     peer (:wat::kernel::Peer :- [I O]) the peer/channel handle to receive from
/// @ret     (:wat::kernel::RecvOutcome :- [O]) Message(O) / Closed / Lost(Failure) / Shutdown
/// @example-norun (:wat::kernel::recv my-peer) #=> #wat.kernel/RecvOutcome.Message{msg: "hi"}
// `//` not `///` — maintainer rationale (see `readln'`'s note in `kernel_stdio.rs`).
//
// No registered `TypeScheme` for `recv` — `check.rs`'s `infer_recv_prime` (~10865)
// is the real authority: `O` is projected out of `args[0]`'s peer<I,O> and flows
// into the wrapped `RecvOutcome<O>` return — the `<O>` above documents that
// projection, not a scheme (the checker also rejects any `-> :T` ascription here,
// killed arc 258.5b: the type flows from the consumer or the self-describing wire).
//
// Deciding line for `@Purity Effectful`: `runtime.rs:31508`,
// `Some(peer) => Ok(match peer.recv() { … })` — a consuming receive that
// dequeues the value from the channel, observable to the sender.
#[wat_intrinsic(":wat::kernel::recv")]
pub(crate) fn eval_peer_recv_prime(
    peer: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::kernel::message::eval_peer_recv_prime(std::slice::from_ref(peer), list_span, env, sym)
}

/// `(:wat::kernel::select peers)` → `:wat::spawn::ServiceEvent<I,O,A>`.
/// Blocks until ONE peer in the (non-empty, same-tier) `peers` vector is
/// ready, and returns its outcome as a matchable `ServiceEvent`. Fan-in
/// over homogeneous peers only — the 3-arg service multiplexer is `poll`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Message
/// @arg     peers (:wat::core::Vector :- [(:wat::kernel::Peer :- [I O])]) non-empty, same-tier peers to fan in over
/// @ret     (:wat::spawn::ServiceEvent :- [I O A]) Message[idx,O] / Closed[idx] / Lost[idx,Failure] — `A` is a free, unconstrained tyvar (select' has no self-peer/admin channel, so :Admin can never fire from it)
/// @example-norun (:wat::kernel::select [peer-a peer-b]) #=> #wat.spawn/ServiceEvent.Message{idx: 0, msg: "hi"}
// `//` not `///` — maintainer rationale (see `readln'`'s note in `kernel_stdio.rs`).
//
// No registered `TypeScheme` for `select` — `check.rs`'s `infer_select_prime`
// (~11483) is the real authority: `I,O` project out of the Vector element's
// peer<I,O>; `A` is `fresh.fresh()` (never constrained) because select' has no
// self-peer to source an admin-receive type from — the `<I,O,A>` above
// documents that projection, not a scheme.
//
// Deciding line for `@Purity Effectful`: `runtime.rs:32388`–`32389`,
// `match sel.select() { crate::comms::SelectOutcome::Recv { index, result } =>
// … }`, which bottoms out in `comms/thread.rs:400`'s `selected_op.recv(ch)` — a
// consuming receive that dequeues the fired peer's value.
#[wat_intrinsic(":wat::kernel::select")]
pub(crate) fn eval_peer_select_prime(
    peers: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::kernel::message::eval_peer_select_prime(std::slice::from_ref(peers), list_span, env, sym)
}

/// `(:wat::kernel::poll self-peer listener peers)` → `:wat::spawn::ServiceEvent<I,O,A>`.
/// The 3-arg service multiplexer: blocks on the owner/admin link
/// (`self-peer`), the connection listener, AND every connected client peer
/// at once, returning whichever fires first as a matchable `ServiceEvent`
/// (`Admin`/`Shutdown`/`Connection`/`Message`/`Closed`). Same locus-delivery
/// DOING as `recv`/`select` — see the module doc's "poll's purity" section
/// for why it is `Effectful`, not `Pure`: every fired arm performs a real
/// consuming receive (or, on the listener arm, completes an accept and
/// mints a new peer), never a bare readiness check.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Message
/// @arg     self_peer (:wat::kernel::Peer :- [S A]) the owner/supervisor link (self-peer); `A` (its receive type) becomes the Admin payload type
/// @arg     listener (:wat::kernel::Listener :- [S R]) the connection listener; inferred permissively, not further constrained
/// @arg     peers (:wat::core::Vector :- [(:wat::kernel::Peer :- [I O])]) the connected client peers
/// @ret     (:wat::spawn::ServiceEvent :- [I O A]) Admin[A] / Shutdown / Connection[Peer<I,O>] / Message[idx,O] / Closed[idx]
/// @example-norun (:wat::kernel::poll self listener clients) #=> #wat.spawn/ServiceEvent.Message{idx: 0, msg: "hi"}
// `//` not `///` — maintainer rationale (see `readln'`'s note in `kernel_stdio.rs`).
//
// No registered `TypeScheme` for `poll` — `check.rs`'s `infer_poll_prime` (~11618)
// is the real authority: `I,O` project out of `peers`'s Vector<Peer'<I,O>>
// element type; `A` projects out of `self-peer`'s Peer'<S,A>/ThreadSelfPeer'<S,A>
// second type-arg. The `<I,O,A>` above documents that three-way projection, not
// a scheme.
//
// Deciding line for `@Purity Effectful` — the axis this stone deliberately left
// undecided: `runtime.rs:33463` `let event_value = match sel.select() { … }`
// bottoms out in `comms/thread.rs:400`'s `selected_op.recv(ch)` on the thread
// tier (`runtime.rs:33615`'s `sel.select_raw()` on the process tier does the
// analogous consuming read) — a real dequeue, not a readiness probe. Declared
// Effectful because the body says so, not for symmetry with its siblings.
#[wat_intrinsic(":wat::kernel::poll")]
pub(crate) fn eval_poll_prime(
    self_peer: &WatAST,
    listener: &WatAST,
    peers: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::kernel::message::eval_poll_prime(
        &[self_peer.clone(), listener.clone(), peers.clone()],
        list_span,
        env,
        sym,
    )
}
