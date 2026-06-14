# BRIEF — Stone C0b.2e-ii: make `Listener` a proper entity (retire `SocketListener'` + the thread raw-`Receiver` fiction)

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/`
(verify `pwd`; operate only here; `git -C /home/watmin/work/holon/wat-rs`). Design:
`DESIGN-STONE-C0b.2e-ii-listener-proper-entity.md` (read it fully). Do NOT commit — the
Inquisitor weighs.

## The work in one paragraph

Today there is no `Listener` entity: the thread-tier `Listener'` is a raw
`comms::thread::Receiver` with a checker-name pasted on; only `SocketListener'` is real.
Make `Listener` a proper first-class entity. Introduce a kernel trait `CommListener`
(method `accept(&self, sym, span) -> Result<Peer, EvalBreak>`) with two impls — a crossbeam
one (the thread `accept'` logic moved in) and a socket one (the process `accept'` logic
moved in) — and a `Listener { inner: Box<dyn CommListener> }` entity under one new
`LISTENER_TYPE_PATH`. `listener'` wraps its mechanism into a `Listener`; `accept'` collapses
to one arm (`inner.accept(...)`). Retire `SOCKET_LISTENER_TYPE_PATH`/`SocketListener'` and
the raw-`Receiver`-as-listener fiction. `Address'` stays a raw `Sender` (that's iii). No
behavior change — the accept logic moves verbatim into the impls.

## Read in order (the rooms)

1. `src/kernel/peer.rs` — the unified `Peer` (the pattern to mirror) + `wrap_connect_request`'s
   home; `src/kernel/spawn.rs:146` `SOCKET_LISTENER_TYPE_PATH` (→ replace with `LISTENER_TYPE_PATH`).
2. `src/runtime.rs` ~18170 `eval_listener_prime` (thread: `thread::pair::<Value>()` → Tuple;
   process: `UnixListener::bind_addr` + `set_nonblocking` → `SOCKET_LISTENER_TYPE_PATH` opaque).
3. `src/runtime.rs` ~18418 `eval_accept_prime` — the socket arm (poll-driven non-blocking
   accept via `process::Select::listener`) + the thread arm (~18497: `typed_recv` +
   `wrap_connect_request`). These two bodies MOVE into the two `CommListener` impls.
4. `src/runtime.rs` ~24164 `wrap_connect_request` (builds the unified `Peer` — the crossbeam
   accept calls it).
5. `src/check.rs` ~10027 `infer_listener_prime` + ~10020 `listener_tuple` + `infer_accept_prime`
   (~10192) — the `SocketListener'` head strings → `Listener'`; `Listener'` becomes a real type.

## Implementation sketch (fill the shape)

**(A) `src/kernel/listener.rs` (NEW):**
```rust
use crate::kernel::peer::Peer;
pub trait CommListener: Send {
    fn accept(&self, sym: &SymbolTable, span: &Span) -> Result<Peer, EvalBreak>;
}
pub struct CrossbeamListener { pub(crate) rx: crate::comms::thread::Receiver<Value> }
pub struct SocketListener   { pub(crate) listener: std::os::unix::net::UnixListener }
impl CommListener for CrossbeamListener { fn accept(&self, sym, span) -> … {
    // MOVE the thread accept' body: typed_recv(self.rx, …) → wrap_connect_request(cr, span)
}}
impl CommListener for SocketListener { fn accept(&self, sym, span) -> … {
    // MOVE the socket accept' body: process::Select::listener(fd) poll-loop → wrap stream as Peer
}}
pub struct Listener { pub(crate) inner: Box<dyn CommListener> }
```
Register `mod listener;` in `src/kernel/mod.rs`. Add `pub const LISTENER_TYPE_PATH:
&str = ":wat::kernel::Listener'";` in spawn.rs (replacing `SOCKET_LISTENER_TYPE_PATH`).

**(B) runtime.rs `listener'`:** thread arm → wrap the minted `Receiver` as
`Listener{ inner: Box::new(CrossbeamListener{rx}) }` opaque (LISTENER_TYPE_PATH), keep the
raw `Sender` as the Address' tuple element; process arm → `Listener{ inner:
Box::new(SocketListener{listener}) }` opaque (LISTENER_TYPE_PATH, was SOCKET_LISTENER_TYPE_PATH).

**(C) runtime.rs `accept'`:** collapse to ONE arm — downcast the `Listener` opaque →
`peer = listener.inner.accept(sym, list_span)?` → Ok(peer). Delete the two inline bodies
(now in the impls).

**(D) check.rs:** `infer_listener_prime` process arm `SocketListener'` → `Listener'`;
`infer_accept_prime` `SocketListener'` head → `Listener'`; `listener_tuple` keeps
`Tuple[Listener'<S,R>, Address'<S,R>]`. Every `"wat::kernel::SocketListener'"` → `"wat::kernel::Listener'"`.

Then `cargo build` and follow the compiler.

## Blast radius

`src/kernel/listener.rs` (new), `src/kernel/mod.rs`, `src/kernel/spawn.rs`, `src/runtime.rs`
(listener'/accept'), `src/check.rs`. No `comms` change. No `connect'`/`Address'` change
beyond the listener slot. No new wat surface (the verbs are unchanged; the entity behind
the opaque becomes real).

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** the thread `accept'` body needs `sym`/context the trait method can't carry —
   STOP, report (method takes `&SymbolTable` + `&Span`; if it needs `env` or more, surface it).
2. **STOP-2:** the socket `accept'` poll-loop can't move into `SocketListener::accept` without
   changing `process::Select` — STOP, report (it moves verbatim; `Select::listener` exists).
3. **STOP-3:** `kernel::listener` importing `comms` + `kernel::Peer` hits a layering/cycle wall
   — STOP, report (expected clean: kernel→comms).

## The gate

```
grep -rn "SOCKET_LISTENER_TYPE_PATH\|wat::kernel::SocketListener'" src/        # EMPTY after
grep -rn "LISTENER_TYPE_PATH" src/kernel/spawn.rs                              # present (the new const)
cargo build --release
cargo test --release -p wat --test nursery connection_primitive probe_arc209_c0b2c probe_arc209_c0b2d -- --test-threads=1   # (run each filter separately)
cargo test --release -p wat --test nursery -- --test-threads=1                # 895 passed / 4 failed (baseline)
cargo test --release --workspace --no-run                                     # full surface compiles
```
(NOTE: cargo test takes one positional filter — run `connection_primitive`, `probe_arc209_c0b2c`,
`probe_arc209_c0b2d` as separate invocations, plus the thread-connection probe if one exists.)
Report each exact `test result:` line + the grep outputs + any STOP/honest delta. Do NOT commit.

## Prior comparable (copy the shape)

`BRIEF-STONE-C0b.2e-i-b.md` (the Peer collapse — same `Box<dyn>` transport-blind-entity
pattern, the opaque + downcast + type-path retire patterns, structural-grep gate).
