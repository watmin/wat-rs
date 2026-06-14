# DESIGN-STONE C0b.2e-ii — make `Listener` a proper entity (retire `SocketListener'` + the thread raw-`Receiver` fiction)

> Builder reframe (grounded): there was never a listener ENTITY. The thread-tier
> `Listener'` is a raw `comms::thread::Receiver` with the checker-name `Listener'<S,R>`
> *pasted on* (no `LISTENER_TYPE_PATH`, no opaque); only `SocketListener'` is a real
> entity. This stone makes `Listener` a proper first-class entity — one real,
> transport-blind type both tiers produce — mirroring how Peer was unified.

## The decision (four-questions, pinned)

A real `Listener` entity = `Box<dyn CommListener>` (open trait, mirror Peer), NOT a closed
`enum Listener{Thread,Socket}` (that bakes a closed transport set on the growing
remote-listener axis — the dishonesty rejected for `PeerTransport`). A remote listener
(TCP/mTLS) is then a new `CommListener` impl. One stone (not split): `CommListener` is a
brand-new trait with no consumer until the entity — splitting the trait off would be an
unused-trait stone.

## Grounded this session (HEAD `4e865d20`)

- **No `LISTENER_TYPE_PATH`** (only `SOCKET_LISTENER_TYPE_PATH`, spawn.rs:146).
- Thread `listener'` (runtime.rs ~18170): `comms::thread::pair::<Value>()` → returns
  `Tuple[Receiver, Sender]` = a raw `Value::wat__kernel__Receiver` (the "Listener'") + a
  raw `Value::wat__kernel__Sender` (the "Address'"). The checker names them via
  `listener_tuple` (check.rs ~10020), but they are not real entities.
- Thread `accept'` (runtime.rs ~18497): downcast `Value::wat__kernel__Receiver`,
  `crate::channel::typed_recv` a connect-request, `wrap_connect_request(cr, span)` → `Peer`.
- Process `listener'` (runtime.rs ~18118): `UnixListener::bind_addr` + `set_nonblocking(true)`
  → opaque `SOCKET_LISTENER_TYPE_PATH`.
- Process `accept'` (runtime.rs ~18444): downcast `&UnixListener`, poll-driven non-blocking
  accept (C0b.3a-i, `process::Select` listener-arm), `wrap_stream_as_socket_peer`-equiv → `Peer`.
- `wrap_connect_request` (runtime.rs ~24164) already builds the unified `Peer` (i-b).
- `ReactorClass{InMemory,Fd}` (comms, i-a) exists. `CommSender<T>`/`CommReceiver<T>` are
  kernel-free generic traits — `CommListener::accept → kernel::Peer` is NOT, so it lives in
  kernel (layering: kernel→comms, never comms→kernel).

## The contract decision (pinned)

**`CommListener` is a KERNEL trait** (`src/kernel/listener.rs`, new home):
```rust
pub trait CommListener: Send {
    /// Block until a connection arrives; wrap + return the server-side Peer.
    fn accept(&self, sym: &SymbolTable, span: &Span) -> Result<Peer, EvalBreak>;
}
pub struct Listener { inner: Box<dyn CommListener> }   // LISTENER_TYPE_PATH; one entity
```
`accept` is the ONLY method this stone needs (standalone `accept'`). The poll'-loop
methods (`reactor_class`/`listen_fd`) are added by **C0b.3a-ii** — its consumer — NOT
here; adding them now would be unused-until-next-stone (don't-build-the-forcing-function).
Impls (this stone ships two; a remote listener is a third, organically):
- **`CrossbeamListener { rx: comms::thread::Receiver<Value> }`** — `accept` = `typed_recv`
  the connect-request + `wrap_connect_request` (the thread `accept'` logic, MOVED here).
- **`SocketListener { listener: UnixListener }`** — `accept` = the poll-driven non-blocking
  accept + wrap (the process `accept'` logic, MOVED here).

`accept'` collapses to ONE arm: downcast the `Listener` opaque → `inner.accept(sym, span)`.
`listener'` wraps its mechanism into a `Listener`: thread → `Tuple[Listener(Crossbeam{rx}),
raw Sender (Address', unchanged)]`; process → `Listener(Socket{listener})`.

**Checker:** `Listener'<S,R>` becomes a REAL type (one `LISTENER_TYPE_PATH`); every
`"wat::kernel::SocketListener'"` head → `"wat::kernel::Listener'"` (`infer_listener_prime`
process arm, `infer_accept_prime` listener arm); `listener_tuple` keeps the thread
`Tuple[Listener'<S,R>, Address'<S,R>]` shape (Address' raw until iii).

## Out of scope (rejected — NOT deferred)

- `Address'` proper entity (the raw-`Sender` fiction + `SocketAddress'`) = **C0b.2e-iii**.
- The poll' loop's listener-arm integration over the proper `Listener` + the
  `reactor_class`/`listen_fd` methods on `CommListener` = **C0b.3a-ii** (it both adds those
  methods and consumes them; not built here — no consumer yet).
- Any remote `CommListener` impl = later organic addition.

## The gate (proper-entity refactor — structural disconfirm)

1. **Structural:** `LISTENER_TYPE_PATH` exists; `SOCKET_LISTENER_TYPE_PATH` /
   `wat::kernel::SocketListener'` grep-EMPTY; the thread `listener'` no longer returns a
   bare `Value::wat__kernel__Receiver` as the listener slot (it's a `Listener` opaque).
2. **Regression (existing listener flow via the proper entity):** `connection_primitive`
   (thread peer-pair, unaffected), the thread connection probe `c0b1` (listener'/connect'/
   accept' crossbeam), `c0b2c`/`c0b2d` (process listener'/accept'/connect' socket) — all GREEN.
3. Nursery serial **895/4** (baseline only) + full workspace test surface compiles.

## Files touched

`src/kernel/listener.rs` (NEW — `CommListener` + `Listener` + the two impls), `src/kernel/mod.rs`
(mod listener), `src/kernel/spawn.rs` (`LISTENER_TYPE_PATH` replaces `SOCKET_LISTENER_TYPE_PATH`),
`src/runtime.rs` (`listener'` wraps → `Listener`; `accept'` one arm via `inner.accept`; move
the two accept bodies into the impls), `src/check.rs` (`SocketListener'`→`Listener'`; `Listener'`
is now a real type). No `Address'`/`connect'` change beyond the listener slot. No `comms` change.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** moving the thread `accept'` body (`typed_recv` + `wrap_connect_request`) into
   `CrossbeamListener::accept` needs `sym`/context the trait method can't carry — STOP, report
   (the method takes `&SymbolTable` + `&Span`; if it needs more, surface it).
2. **STOP-2:** the socket `accept'` poll-loop can't move into `SocketListener::accept` without
   a structural change to `process::Select` — STOP, report (it should move verbatim;
   `Select::listener` already exists).
3. **STOP-3:** a layering wall — `kernel::listener` importing `comms` + `kernel::Peer` — does
   not compile cleanly — STOP, report (expected clean: kernel→comms).

## The deadlock contract carries

Each impl preserves its transport's exact accept semantics (crossbeam rendezvous recv;
io_uring poll-driven non-blocking accept). A representation/home move, no deadlock-surface
change. [[feedback_vended_primitives_never_deadlock]] [[feedback_optional_is_a_smell]]
