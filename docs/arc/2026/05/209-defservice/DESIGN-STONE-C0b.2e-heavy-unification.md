# DESIGN-STONE C0b.2e — the HEAVY connection unification (engineer for remote)

> Make thread / process / remote look IDENTICAL at the surface, by making the transport axis a
> CLOSED, COMPILER-ENFORCED enum. The four-questions chose HEAVY (full runtime merge) over LIGHT
> (type-only) because the transport set is KNOWN to grow — remote is designed-in ("process correct ⟹
> remote correct"). Builder: *"heavy it is — we engineer for remote's existence before it exists."*
> HEAVY's buy: adding a transport (remote) is ONE enum variant + the compiler forces an arm at every
> `send'`/`recv'`/`select'` site (exhaustiveness); serialized transports (socket, remote) share the
> EDN wire. The failure class "added a transport, forgot a dispatch site" becomes unrepresentable.

## The end-state (what "unified" means)

> ⚠️ REVISED 2026-06-13 (builder): "remote" is a CLASS (N kinds — verifications, resumption,
> protocols), not one thing, and growing it must be ORGANIC. So the seam is NOT a closed
> `enum PeerTransport {Crossbeam, Socket, Remote}` (which would force central surgery + per-site arms
> per remote). The seam is the comms TRAIT you already built — **operations closed, transports open.**

The source of truth is the existing comms abstraction (`comms/mod.rs:620,641`):
`trait CommSender<T> { fn send }` + `trait CommReceiver<T> { fn recv }`, already impl'd by
`comms::thread` (crossbeam, Value-direct) AND `comms::process` (socket, EDN). "Value over crossbeam
LOOKS LIKE value over EDN-over-socket" — true at the trait.

- **`Peer<I,O>`** (one struct, one `PEER_TYPE_PATH` opaque) holds the **boxed comms trait**:
  `tx: Box<dyn CommSender<Value>>`, `rx: Box<dyn CommReceiver<Value>>`. `send'`/`recv'` call the
  trait — **NO per-transport arms.** Encoding moves INTO the comms impl (crossbeam sends `Value`
  direct; socket EDN-encodes internally) so `Peer::send(Value)`/`recv()→Value` is uniform. **A new
  transport (each remote) = a new `CommSender`/`CommReceiver` impl + a connector — `send'`/`recv'`/
  `select'` never change.** That is the organic growth: add impls, not arms. **`SocketPeer` /
  `SOCKET_PEER_TYPE_PATH` are RETIRED** — there is one connection peer over the open trait.

## The real axis: in-memory (capture) vs socket (no-capture)

The genuinely-distinct split is NOT thread/process/remote — it is **in-memory vs socket**:
- **In-memory** — `thread` (crossbeam). The ONE env that ALLOWS CAPTURE (closures); that is why it is
  separately named and gets different SPAWN treatment (closure prog, fn-arg self-peer). At the
  CONNECTION layer it is just the crossbeam-backed `Peer`.
- **Socket / out-of-memory, no-capture** — `process` (same-machine UDS) + the **remote CLASS**
  (off-machine; N kinds, growing). Process-IPC-is-already-sockets PAVES THE ROAD: every remote is "an
  out-of-memory socket endpoint not on this machine," sharing the socket transport + the forms-over-
  wire (no-capture) model. The remote class grows in the **connector** layer (how the endpoint is
  established + its verification/resumption), each a plug-in, never central surgery.
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
- **Simple — YES (end-state).** `send'`/`recv'` collapse from a 4-way `type_path` match to
  `{Thread', Process', Peer}`, and the `Peer` arm is a single trait call (`tx.send`/`rx.recv`) — no
  per-transport sub-arms. The transport is behind the comms trait, not enumerated at the call site.
- **Honest — YES.** The trait names the operation contract; each transport impl names its real wire;
  the "can't cross a boundary" guard lives at `ServiceAddress` construction (a `:Thread` address is
  same-process-only). Transport-completeness is structural by POLYMORPHISM (the sites call the trait;
  there is no arm to forget) — the open-set version of the extirpare top rung.
- **Good UX — YES.** Adding a remote = a new `CommSender`/`CommReceiver` impl + a connector, with
  `send'`/`recv'`/`select'` untouched (organic); callers tier-blind once connected; `defservice`
  generates the per-tier glue.

Cost (priced + accepted): a one-time wide refactor (~50 sites; bulk in `runtime.rs` + `check.rs`)
revising shipped C0b.2b/2c/2d runtime. Cheapest NOW — before remote and before C0b.3a-ii / Stone C
build on the representation.

## Decomposition (ordered; each its own RED/regression-gated strike)

- **C0b.2e-i — unify the connection `Peer`.** Merge `Peer`+`SocketPeer` structs → one `Peer<I,O>`
  holding the boxed comms trait (`tx: Box<dyn CommSender<Value>>`, `rx: Box<dyn CommReceiver<Value>>`);
  one `PEER_TYPE_PATH` (retire `SOCKET_PEER_TYPE_PATH`). Move EDN encoding INTO the socket comms impl
  (use a `CommSender<Value>`/`CommReceiver<Value>` over the socket that encodes internally) so the
  arm stops encoding. Collapse the `send'`/`recv'`/`select'`(×3)/`connect'`/`accept'`/`peer-pair'`/
  `socket-pair'`/`self-peer` connection-peer arms to ONE `Peer` arm = a trait call. Checker: one
  `Peer<I,O>` head; `project_peer_io` → `{Thread',Process',Peer}`; `infer_connect'/accept'/
  socket-pair'/peer-pair'/self-peer` → `Peer<I,O>`; `SelectEvent.:Connection` → `Peer<I,O>`.
  ⚠️ Verify whether `comms::process::Sender<Value>` encodes internally or the arm must (read
  `comms::process::Sender::send`, `process.rs:245`) — settle encoding placement at draw time.
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
- **Building any `:Remote` member** — the remote CLASS is open; each member (a verification /
  resumption / protocol flavor) is a future `CommSender`/`CommReceiver` impl + connector, added
  organically. C0b.2e builds the SEAM (the trait-held `Peer` + the connector layer) so that adding a
  remote is additive; it builds zero remote members. The road exists before the cars.
- **`SO_PEERCRED`** — C0b.3b.

## Loopback-TCP unlock + remote-readiness invariants (the pressure test)

The honest test of this seam: can two wat programs later talk over **loopback TCP** by *adding*, not
*rebuilding*? Traced end-to-end — yes, because the machinery is fd-generic and trait-seamed:
- **Peer transport is reused.** A TCP socket is `AF_INET SOCK_STREAM`; C0b.2c proved io_uring
  (`PollAdd POLLIN|POLLHUP` + `Read`) is address-family-agnostic on stream sockets, and
  `sender_receiver_from_fd(fd: OwnedFd)` takes any fd. `TcpStream → OwnedFd → Socket Peer` — same
  comms impl, zero new transport code.
- **Reactor is reused.** `select'`'s listener-arm + read-arms (C0b.3a-i) poll any fd; a TCP listen fd
  signals POLLIN on an incoming connection like a UDS one.
- **Address slot exists.** `ServiceAddress::Remote[host port]` (declared). Loopback TCP = fill its
  `connect'`/`listener'` arm with `TcpStream::connect`/`TcpListener::bind`+accept.
- **mTLS rides the SAME shape** — plain TCP reuses the raw-fd `Socket` comms; mTLS is a NEW
  `CommSender`/`CommReceiver` impl (TLS-wrapped; can't ride raw io_uring `Read`), admitted by the
  TRAIT seam as a new impl + connector. This is precisely why the seam is the trait, not a closed enum.

So the only new code for a future loopback-TCP member is a **connector** (TCP connect + bind/accept)
wired to the `:Remote` arm (+ for mTLS, a TLS comms impl). Everything downstream is reused.

**Remote-readiness invariants — C0b.2e MUST honor these so the build does not weld the seam to UDS:**
1. **`Peer` stays fd-source-agnostic** — it holds the boxed comms trait over an `OwnedFd`; it never
   names UDS. (Already true via `sender_receiver_from_fd`.)
2. **`Listener` (C0b.2e-ii) accepts through an ABSTRACTION, not a hardcoded `UnixListener`** — a
   "thing that accepts → `OwnedFd`" so a `TcpListener` plugs in beside the UDS one. The current UDS
   hardcode is fine *as the `:Process` member*; the seam must allow a `:Remote` member beside it.
3. **`connect'`/`listener'` dispatch on the `ServiceAddress` variant** — `:Remote` routes to a (future)
   TCP/TLS connector without touching the `:Thread`/`:Process` paths.

**Named future gate:** when the first `:Remote` connector ships, the validation is a **loopback-TCP
wat-to-wat round-trip** (two wat programs, one binds `:Remote 127.0.0.1:port`, the other dials it,
EDN round-trips). NOT built now — the road is what C0b.2e guarantees; the car is a later stone.

## The deadlock contract carries

The unified `Peer`/`Listener` preserve every transport's existing semantics (non-blocking UDS listen,
io_uring socket recv, crossbeam rendezvous). No deadlock-surface change — a representation merge.
[[feedback_vended_primitives_never_deadlock]]
