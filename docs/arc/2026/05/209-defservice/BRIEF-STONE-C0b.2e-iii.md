# BRIEF — Stone C0b.2e-iii: make `Address` a proper entity (retire `SocketAddress'` + the thread raw-`Sender` fiction)

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/` (verify
`pwd`; operate only here; `git -C /home/watmin/work/holon/wat-rs`). Design:
`DESIGN-STONE-C0b.2e-iii-address-proper-entity.md` (read it fully). This mirrors C0b.2e-ii
(the Listener proper-entity stone) almost exactly. Do NOT commit — the Inquisitor weighs.

## The work in one paragraph

Today there is no `Address` entity: the thread-tier `Address'` is a raw
`comms::thread::Sender` with a checker-name pasted on; only `SocketAddress'` is real (an
opaque wrapping a String name). Make `Address` a proper first-class entity, exactly as ii
did for Listener. Introduce a kernel trait `CommAddress` (method `connect(&self, sym, span)
-> Result<Peer, EvalBreak>` — an address is dialed, never poll'd, so this is the ONLY method;
no `reactor_class`/`as_any`) with two impls — `ThreadAddress{tx}` (the `connect'` thread arm
body moved in) and `SocketAddress{name}` (the `connect'` socket arm body moved in) — and an
`Address{inner: Box<dyn CommAddress>}` entity under one new `ADDRESS_TYPE_PATH`. `connect'`
collapses to one arm (`inner.connect(...)`); `socket-address'` produces an `Address`;
`listener'` (thread) wraps its rendezvous `Sender` as an `Address` for the Address' tuple
slot. Retire `SOCKET_ADDRESS_TYPE_PATH`/`SocketAddress'` and the raw-`Sender` fiction. No
behavior change — the connect bodies move verbatim.

## Read in order (the rooms)

1. `src/kernel/listener.rs` — the EXACT pattern to mirror (`CommListener` trait + `Listener`
   entity + two impls; new home `src/kernel/address.rs` mirrors it).
2. `src/kernel/spawn.rs:151` `SOCKET_ADDRESS_TYPE_PATH` (→ replace with `ADDRESS_TYPE_PATH`).
3. `src/runtime.rs` ~17964 `eval_socket_address_prime` (String name → opaque); ~18244 the
   `connect'` thread arm (raw `Sender` → mint pairs + `typed_send` connect-request → `Peer::from_thread`);
   ~18194 the `connect'` socket arm (`SocketAddress'` → `connect_addr` → `Peer::from_socket`);
   the `listener'` (thread) Address' tuple slot (~18170 region, the `sender_from_comms`/second-element).
4. `src/check.rs` ~9916 `infer_socket_address_prime` + ~10133 `infer_connect_prime` +
   `listener_tuple` — the `SocketAddress'` head strings → `Address'`; `Address'` becomes a real type.

## Implementation sketch (fill the shape — mirror listener.rs)

**(A) `src/kernel/address.rs` (NEW):**
```rust
use crate::kernel::peer::Peer;
pub trait CommAddress: Send {
    fn connect(&self, sym: &SymbolTable, span: &Span) -> Result<Peer, EvalBreak>;
}
pub struct ThreadAddress { pub(crate) tx: crate::comms::thread::Sender<Value> }
pub struct SocketAddress { pub(crate) name: String }
impl CommAddress for ThreadAddress { fn connect(&self, sym, span) -> … {
    // MOVE the connect' thread arm body: mint req/resp pairs, Peer::from_thread,
    // typed_send(self.tx, connect_req) → return the client Peer
}}
impl CommAddress for SocketAddress { fn connect(&self, sym, span) -> … {
    // MOVE the connect' socket arm body: SocketAddr::from_abstract_name(self.name) →
    // UnixStream::connect_addr → sender_receiver_from_fd::<Value> → Peer::from_socket
}}
pub struct Address { pub(crate) inner: Box<dyn CommAddress> }
```
Register `mod address;` in `src/kernel/mod.rs`. Add `pub const ADDRESS_TYPE_PATH: &str =
":wat::kernel::Address'";` in spawn.rs (replacing `SOCKET_ADDRESS_TYPE_PATH`).

**(B) runtime.rs:** `socket-address'` → `Address{inner: Box::new(SocketAddress{name})}` opaque
(ADDRESS_TYPE_PATH). `connect'` → ONE arm: downcast the `Address` opaque → `addr.inner.connect(sym, list_span)`.
`listener'` (thread) Address' tuple slot → `Address{inner: Box::new(ThreadAddress{tx})}` opaque.

**(C) check.rs:** `infer_socket_address_prime` → `Address'`; `infer_connect_prime` accepts
`Address'` (one head, both tiers) → `Peer'`; `listener_tuple` keeps `Tuple[Listener'<S,R>, Address'<S,R>]`.
Every `"wat::kernel::SocketAddress'"` → `"wat::kernel::Address'"`.

Then `cargo build` and follow the compiler.

## Blast radius

`src/kernel/address.rs` (new), `src/kernel/mod.rs`, `src/kernel/spawn.rs`, `src/runtime.rs`
(socket-address'/connect'/listener'-thread-slot), `src/check.rs`. No `comms` change. No
`poll'`/`Listener`/`Peer` change. No new wat surface (the verbs are unchanged; the entity
behind the opaque becomes real).

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** the `connect'` thread arm body needs `env`/context the trait method can't carry
   (it takes `&SymbolTable` + `&Span`) — STOP, report.
2. **STOP-2:** `kernel::address` importing `comms` + `kernel::Peer` + `channel` (typed_send)
   hits a layering/cycle wall — STOP, report (expected clean: kernel→comms/channel).
3. **STOP-3:** a non-connect consumer reads `SOCKET_ADDRESS_TYPE_PATH` the collapse can't subsume — STOP, report.

## The gate

```
grep -rn "SOCKET_ADDRESS_TYPE_PATH\|wat::kernel::SocketAddress'" src/        # EMPTY after
grep -rn "ADDRESS_TYPE_PATH" src/kernel/spawn.rs                            # present (the new const)
cargo build --release
cargo test --release -p wat --test nursery connection_primitive -- --test-threads=1
cargo test --release -p wat --test nursery probe_arc209_c0b2c -- --test-threads=1
cargo test --release -p wat --test probe_arc209_c0b2d_named_cross_process
cargo test --release -p wat --test probe_arc209_c0b3aii_process_service_loop -- --test-threads=1
cargo test --release -p wat --test nursery -- --test-threads=1            # 895 passed / 4 failed (baseline)
cargo test --release --workspace --no-run
```
(NOTE: cargo test takes one positional filter — run each filter as its own invocation.)
Report each exact `test result:` line + the grep outputs + any STOP/honest delta. Do NOT commit.

## Prior comparable (copy the shape)

`BRIEF-STONE-C0b.2e-ii.md` + `src/kernel/listener.rs` — the Listener proper-entity stone is
the same pattern (new kernel trait `CommX` + `Box<dyn>` entity + two impls with the verb
bodies moved in + a new TYPE_PATH; structural-grep gate). `CommAddress` is even simpler (just
`connect`, no `reactor_class`/`as_any`).
