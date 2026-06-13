# DESIGN-STONE C0b.2e — the HEAVY connection unification (engineer for remote)

> Make thread / process / remote look IDENTICAL at the surface, by making the transport axis a
> CLOSED, COMPILER-ENFORCED enum. The four-questions chose HEAVY (full runtime merge) over LIGHT
> (type-only) because the transport set is KNOWN to grow — remote is designed-in ("process correct ⟹
> remote correct"). Builder: *"heavy it is — we engineer for remote's existence before it exists."*
> HEAVY's buy: adding a transport (remote) is ONE enum variant + the compiler forces an arm at every
> `send'`/`recv'`/`select'` site (exhaustiveness); serialized transports (socket, remote) share the
> EDN wire. The failure class "added a transport, forgot a dispatch site" becomes unrepresentable.

## The end-state (what "unified" means)

ONE transport enum is the source of truth for "how does this peer move bytes":
```rust
enum PeerTransport {
    Crossbeam { tx: comms::thread::Sender<Value>,   rx: comms::thread::Receiver<Value>  }, // Value-direct (shared mem)
    Socket    { tx: comms::process::Sender<String>, rx: comms::process::Receiver<String> }, // EDN-String (separate mem)
    // Remote { … }  ← the forcing function: declared shape, added when :remote ships
}
```
- **`Peer<I,O>`** (one struct, one `PEER_TYPE_PATH` opaque) wraps a `PeerTransport`. `send'`/`recv'`
  match the transport (`Crossbeam` → `Value` direct; `Socket` → EDN-encode/decode). **`SocketPeer` /
  `SOCKET_PEER_TYPE_PATH` are RETIRED** — there is one connection peer.
- **`Listener<I,O>`** (one `LISTENER_TYPE_PATH` opaque) wraps a `ListenerTransport`
  (`Rendezvous`(crossbeam Receiver) | `Uds`(UnixListener) | `Inet` later). `accept'` + the
  `select'`-3arg listener-arm match the transport. **`SocketListener'` + the naked thread
  `Receiver`-as-listener RETIRED.**
- **`ServiceAddress<S,R>`** — a tagged sum (the ONE tier-explicit client value):
  ```
  (defenum :wat::kernel::ServiceAddress<S,R>
    :Thread  [addr <- :wat::kernel::Address'<S,R>]   ;; same-process only (the in-mem Sender)
    :Process [name <- :wat::core::String]            ;; abstract UDS name (from socket-address')
    :Remote  [host <- :wat::core::String  port <- :wat::core::i64])  ;; forcing function
  ```
  `connect'` takes a `ServiceAddress<S,R>` → `Peer<S,R>`, dispatching on the variant; the `:Remote`
  arm is a clean "remote tier not yet built" error until `:remote` ships (the variant declares the
  shape; the arm names it unbuilt — [[feedback_dont_build_the_forcing_function]]).
- **`SelectEvent<I,O>`** — `:Connection` carries the unified `Peer<I,O>`. **No `SocketSelectEvent`**
  is ever created (the C0b.3a-ii plan is simplified by this).

What stays tier-DISTINCT (honestly): the **spawn handles** `Thread'`/`Process'` (an *owner's handle
to a spawned program* — lifecycle/`close'`/pidfd, a different abstraction from a *connection*), and
the **spawn host** + **spawn-prog form** (closure vs forms) + **self-acquisition** (fn-arg vs
`(self-peer)`) — because those are where shared-vs-separate-memory genuinely differs and the choice
is the author's. The tier is explicit at the **choice points** (host, `ServiceAddress` variant);
unified for the **connected peer's use**.

## Why HEAVY (four-questions, LOCKED)

- **Obvious — YES.** One `Peer`, one `Listener`, a `ServiceAddress` sum. The wat surface is identical
  across tiers; the transport is data.
- **Simple — YES (end-state).** `send'`/`recv'`/`select'` go from a 4-way `type_path` match to
  `{Thread', Process', Peer}` + an inner transport match — the transport localized to one enum.
- **Honest — YES.** The transport enum names the real transports; the "can't cross a boundary" guard
  lives at `ServiceAddress` construction (a `:Thread` address is same-process-only). Compiler
  exhaustiveness makes transport-completeness STRUCTURAL, not hand-discipline (extirpare top rung).
- **Good UX — YES.** Adding remote = one variant + compiler-forced arms; callers tier-blind once
  connected; `defservice` generates the per-tier glue.

Cost (priced + accepted): a one-time wide refactor (~50 sites; bulk in `runtime.rs` + `check.rs`)
revising shipped C0b.2b/2c/2d runtime. Cheapest NOW — before remote and before C0b.3a-ii / Stone C
build on the representation.

## Decomposition (ordered; each its own RED/regression-gated strike)

- **C0b.2e-i — unify the connection `Peer`.** Merge `Peer`+`SocketPeer` structs → one `Peer<I,O>`
  with `PeerTransport {Crossbeam, Socket}`; one `PEER_TYPE_PATH` (retire `SOCKET_PEER_TYPE_PATH`).
  Collapse the `send'`/`recv'`/`select'`(×3)/`connect'`/`accept'`/`peer-pair'`/`socket-pair'`/
  `self-peer` connection-peer arms to the unified peer + inner transport match. Checker: one
  `Peer<I,O>` head; `project_peer_io` → `{Thread',Process',Peer}`; `infer_connect'/accept'/
  socket-pair'/peer-pair'/self-peer` → `Peer<I,O>`; `SelectEvent.:Connection` → `Peer<I,O>`.
  **Gate (refactor — structural disconfirm):** `SOCKET_PEER_TYPE_PATH` grep-empty; every arc-209 peer
  probe (c0b1b/c0b2b/c0b2c/c0b2d/c0b3a0) green via the unified `Peer`; full surface compiles.
- **C0b.2e-ii — unify the `Listener`.** Merge the thread rendezvous Receiver + `SocketListener'` →
  one `Listener<I,O>` with `ListenerTransport {Rendezvous, Uds}`; one `LISTENER_TYPE_PATH`.
  `listener'`/`accept'`/the `select'`-3arg listener-arm dispatch on the listener transport. **Gate:**
  c0b1b (rendezvous) + c0b2c/c0b2d (uds) green; `SocketListener'` grep-empty.
- **C0b.2e-iii — `ServiceAddress<S,R>` sum.** The `defenum` + `connect'(ServiceAddress)` dispatch
  (`:Thread`/`:Process`/`:Remote`); `:Remote` arm = forcing-function error. The thread `Address'` +
  `socket-address'` fold into the variants. **Gate:** a caller connects via a `ServiceAddress` on
  thread AND process, identically (one code path, two variants).

Then C0b.3a-ii emits the UNIFIED `SelectEvent` (no `SocketSelectEvent`), and Stone C / defservice
build on the unified surface.

## Gate-shape note (i + ii are refactors)

i and ii are type-MERGES, not feature additions — there is no pre-committed wat RED (the disconfirming
fact is structural: the retired `type_path` is still grep-present at HEAD). The gate is
regression-green across the arc-209 probes + grep-empty of the retired path + full-surface compile.
iii IS a feature (the `ServiceAddress` sum) → a normal RED probe (a caller connects via a
`ServiceAddress` — fails at HEAD because the type doesn't exist).

## Out of scope = rejected (named)

- **Merging the spawn handles `Thread'`/`Process'` into `Peer`** — REJECTED: ownership/lifecycle
  (`close'`/pidfd) is a different abstraction from a connection. A separate question, not this.
- **Building the `:Remote` transport** — declared (the enum variant + the error arm), built when
  `:remote` ships. The whole point: the shape exists before the implementation.
- **`SO_PEERCRED`** — C0b.3b.

## The deadlock contract carries

The unified `Peer`/`Listener` preserve every transport's existing semantics (non-blocking UDS listen,
io_uring socket recv, crossbeam rendezvous). No deadlock-surface change — a representation merge.
[[feedback_vended_primitives_never_deadlock]]
