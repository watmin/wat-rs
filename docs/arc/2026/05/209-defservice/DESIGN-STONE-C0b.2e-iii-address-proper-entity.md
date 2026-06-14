# DESIGN-STONE C0b.2e-iii — make `Address` a proper entity (retire `SocketAddress'` + the thread raw-`Sender` fiction)

> The last connection-surface fiction. Like `Listener` before it (C0b.2e-ii), there is no
> `Address` ENTITY: the thread-tier `Address'` is a raw `comms::thread::Sender` with the
> checker-name `Address'<S,R>` pasted on; only `SocketAddress'` is real (an opaque wrapping a
> String name). Make `Address` a proper, transport-blind first-class entity — mirroring
> Peer/Listener. After this, the whole connection surface (Peer, Listener, Address) is unified
> + properly-named + first-class.

## The decision (four-questions, pinned)

`CommAddress` trait + `Box<dyn CommAddress>` `Address` (open trait, mirror Peer/Listener),
NOT a closed `ServiceAddress` sum (`:Thread/:Process/:Remote` with a `:Remote`
forcing-function-error variant). The closed sum bakes a closed transport set on the growing
remote axis — the exact pattern rejected for `PeerTransport` ("remote is N things"). A remote
address = a new `CommAddress` impl (its `connect` dials TCP+mTLS), organic, zero central edit.
(The "ServiceAddress sum" framing in the old C0b.2e design + task #229 is STALE — superseded
by the open-trait decision the Peer/Listener stones established.) One stone (CommAddress has
no consumer until the entity).

`CommAddress` is the SIMPLEST of the three connection traits — an address is *dialed*, never
*poll'd*, so it needs only `connect`; NO `reactor_class`, NO `as_any` (connect' just calls it).

## Grounded this session (HEAD `a7060c32`)

- **No `ADDRESS_TYPE_PATH`** (only `SOCKET_ADDRESS_TYPE_PATH`, spawn.rs:151).
- Thread `Address'` = a raw `Value::wat__kernel__Sender` (the rendezvous tx); produced by
  `listener'` (thread) → `Tuple[Listener', Address']` (Address' = the second element). The
  checker names it `Address'<S,R>` via `listener_tuple` — not a real entity.
- `connect'` thread arm (runtime.rs ~18244): downcast `Value::wat__kernel__Sender`, mint
  `comms::thread::pair::<Value>()` ×2 (req/resp), `client = Peer::from_thread(req_tx, resp_rx)`,
  `connect_req = Tuple[receiver_from_comms(req_rx), sender_from_comms(resp_tx)]`,
  `typed_send(addr_sender, connect_req)` → return `client` Peer.
- `connect'` socket arm (runtime.rs ~18194): `SocketAddress'` opaque → name String →
  `UnixStream::connect_addr` → `wrap_stream_as_socket_peer`-equiv → `Peer::from_socket`.
- `socket-address'` (runtime.rs ~17964, `eval_socket_address_prime`): `(name :S :R)` →
  `SOCKET_ADDRESS_TYPE_PATH` opaque wrapping the String name.
- Checker: `infer_socket_address_prime` → `SocketAddress'<S,R>`; `infer_connect_prime`
  `Address'`/`SocketAddress'` → `Peer'` (i-b); `listener_tuple` → `Tuple[Listener'<S,R>, Address'<S,R>]`.
- `CommListener` (C0b.2e-ii, kernel/listener.rs) is the exact pattern to mirror.

## The contract decision (pinned)

**`CommAddress` is a KERNEL trait** (`src/kernel/address.rs`, new home — mirrors listener.rs;
a proper entity gets a home):
```rust
pub trait CommAddress: Send {
    /// Dial this address; return the connected client-side Peer.
    fn connect(&self, sym: &SymbolTable, span: &Span) -> Result<Peer, EvalBreak>;
}
pub struct ThreadAddress { tx: comms::thread::Sender<Value> }   // dial = mint pairs + typed_send the connect-request
pub struct SocketAddress { name: String }                       // dial = UnixStream::connect_addr(abstract name)
pub struct Address { inner: Box<dyn CommAddress> }              // ADDRESS_TYPE_PATH; one entity
```
- `ThreadAddress::connect` = the `connect'` thread arm body MOVED here (mint req/resp pairs,
  `Peer::from_thread`, `typed_send` the connect-request via `self.tx`).
- `SocketAddress::connect` = the `connect'` socket arm body MOVED here (`connect_addr` + wrap → `Peer::from_socket`).
- `connect'` collapses to ONE arm: downcast the `Address` opaque → `inner.connect(sym, span)`.
- `socket-address'` produces `Address{ inner: Box::new(SocketAddress{name}) }` (ADDRESS_TYPE_PATH,
  was SOCKET_ADDRESS_TYPE_PATH).
- `listener'` (thread) wraps its rendezvous `Sender` as `Address{ inner: Box::new(ThreadAddress{tx}) }`
  for the Address' tuple slot (was the raw `Sender`).
- **Checker:** `Address'<S,R>` becomes a REAL type (one `ADDRESS_TYPE_PATH`); every
  `"wat::kernel::SocketAddress'"` head → `"wat::kernel::Address'"`; `listener_tuple` keeps
  `Tuple[Listener'<S,R>, Address'<S,R>]`; `infer_socket_address_prime` → `Address'<S,R>`;
  `infer_connect_prime` accepts `Address'` (one head, both tiers) → `Peer'`.

## The gate (proper-entity refactor — structural disconfirm)

1. **Structural:** `ADDRESS_TYPE_PATH` exists; `SOCKET_ADDRESS_TYPE_PATH` /
   `wat::kernel::SocketAddress'` grep-EMPTY; the thread `listener'` Address' slot is an
   `Address` opaque, not a bare `Value::wat__kernel__Sender`.
2. **Regression (existing connect flow via the proper entity):** `connection_primitive`
   (thread peer-pair, unaffected), `c0b1` (thread listener'/connect'/accept'), `c0b2c`
   (process same-process), `c0b2d` (cross-process connect-by-name), `c0b3aii` (the process
   service loop — uses `socket-address'`+`connect'`) — all GREEN.
3. Nursery serial **895/4** (baseline only) + full workspace compiles.

## Files touched

`src/kernel/address.rs` (NEW — `CommAddress` + `Address` + `ThreadAddress`/`SocketAddress`),
`src/kernel/mod.rs` (mod address), `src/kernel/spawn.rs` (`ADDRESS_TYPE_PATH` replaces
`SOCKET_ADDRESS_TYPE_PATH`), `src/runtime.rs` (`socket-address'` → `Address`; `connect'` one
arm via `inner.connect`; `listener'` thread Address' slot → `Address`; move the two connect
bodies into the impls), `src/check.rs` (`SocketAddress'`→`Address'`; `Address'` is now a real
type). No `comms` change. No `poll'`/`Listener`/`Peer` change.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** moving the `connect'` thread arm body into `ThreadAddress::connect` needs
   `env`/context the trait method can't carry (it takes `&SymbolTable` + `&Span`) — STOP, report.
2. **STOP-2:** `kernel::address` importing `comms` + `kernel::Peer` + `channel` (for `typed_send`)
   hits a layering wall — STOP, report (expected clean: kernel→comms/channel).
3. **STOP-3:** a non-connect consumer reads `SOCKET_ADDRESS_TYPE_PATH` the collapse can't subsume — STOP, report.

## Out of scope (rejected — NOT deferred)

- SO_PEERCRED allow-set = **C0b.3b**. Remote (`AF_INET`+mTLS) = a later `CommAddress`/`CommListener`/`Comm*` impl.
- After iii: thread+process **parity** is reached for the connection+service surface → **Stone C**
  (the defservice defmacro) is unblocked (modulo C0b.3b's identity gate).

## The deadlock contract carries

Each `connect` impl preserves its transport's exact dial semantics (crossbeam rendezvous-send;
socket `connect_addr`). A representation/home move, no deadlock-surface change.
[[feedback_vended_primitives_never_deadlock]] [[feedback_optional_is_a_smell]]
