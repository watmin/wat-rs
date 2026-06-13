# DESIGN-STONE C0b.2e-i-b — the unified `Peer` collapse (retire `SocketPeer'`)

> Second strike of the connection-`Peer` unification (consumes the i-a foundation:
> `as_any` for concrete recovery + `ReactorClass`, shipped `aac27fb5`). Collapses the two
> connection-peer types (`Peer` crossbeam / `SocketPeer` socket) into ONE boxed `Peer` and
> retires `SocketPeer'` / `SOCKET_PEER_TYPE_PATH`. A **pure structural collapse** — after
> it, a connection peer is transport-blind at the wat surface; a remote is a new
> `CommSender`/`CommReceiver` impl, not a new type.

## Scope correction (grounded `infer_select_prime`, check.rs:10828)

`select'` 1-arg's checker accepts **only** `Thread'`/`Process'` (spawn handles) — NOT
connection peers. So there is no checked "select' 1-arg over connection peers" capability
to "complete over both tiers." This stone therefore does **not** add socket `select'`
routing. The runtime `select'` `Peer'` arm (Stone C0, reached only by crossbeam connection
peers) keeps feeding `thread::Select`; because the unified `Peer`'s receiver is now boxed,
it recovers the concrete `&thread::Receiver<Value>` via `as_any` (the i-a foundation).
A socket-backed connection `select'` (the 3-arg service multiplexer) is **C0b.3a-ii** — a
distinct capability, not a deferral from this collapse. (`reactor_class`, the other half of
the i-a foundation, is consumed there.) No speculative socket branch is built here
([[feedback_dont_build_the_forcing_function]]).

## The runtime representation (pinned)

Today (grounded): the crossbeam connection peer is `Peer<Value,Value>` (peer.rs:182,
`tx: thread::Sender<Value>`, `rx: thread::Receiver<Value>`); the socket connection peer
is `SocketPeer<String,String>` (peer.rs:335, `tx: process::Sender<String>`, `rx:
process::Receiver<String>`) with **arm-level** `value_to_edn` encoding in `send'`/`recv'`.

Unified (non-generic, transport-erased to `Value`):
```rust
pub struct Peer {
    tx: Box<dyn crate::comms::CommSender<crate::value::Value>>,
    rx: Box<dyn crate::comms::CommReceiver<crate::value::Value>>,
}
impl Peer {
    pub fn from_thread(tx: comms::thread::Sender<Value>,  rx: comms::thread::Receiver<Value>)  -> Self;
    pub fn from_socket(tx: comms::process::Sender<Value>, rx: comms::process::Receiver<Value>) -> Self;
    pub fn send(&self, v: Value) -> Result<(), SendError<Value>> { self.tx.send(v) }
    pub fn recv(&self) -> Result<Value, RecvError>              { self.rx.recv() }
}
```
The socket side switches `String` → `Value`: `process::Sender<Value>` encodes internally
via `value.to_wire()` (process.rs:172, the i-0 payoff), so `send'`/`recv'` **delete** their
arm-level `value_to_edn`/`wat_edn::write` codec — encoding lives in the comms impl, not the
runtime arm ([[feedback_contract_not_encoding]], made structural). `SocketPeer` is deleted;
`Peer<Value,Value>` becomes the non-generic `Peer`.

## The one arm that uses `as_any` — `select'` connection-peer downcast

`select'` 1-arg `Peer'` arm (runtime.rs:23821) + 3-arg self-peer/client downcasts
(runtime.rs:23971/24062) currently downcast the opaque to `Peer<Value,Value>` and read
`.rx` (a concrete `thread::Receiver`) to feed `thread::Select`. After the collapse `.rx`
is `Box<dyn CommReceiver<Value>>`; recover the concrete `&thread::Receiver<Value>` via
`rx.as_any().downcast_ref::<thread::Receiver<Value>>()` (proven by the i-a probe), then
feed `thread::Select` exactly as today. These connection peers are crossbeam (the C0 /
c0b1b paths — the live `probe_arc209_connection_primitive` 1-arg `select'` test exercises
this exact arm). On `as_any` → `None` (a non-crossbeam connection peer — not reachable by
any current test) the arm returns a **clean error** pointing at C0b.3a-ii, never a panic;
no `process::Select` branch is built here (no live caller — [[feedback_dont_build_the_forcing_function]]).

## The site map (grounded — Explore + this-session reads)

**peer.rs:** redefine `Peer` (boxed, non-generic, + `from_thread`/`from_socket`); DELETE
`SocketPeer` (peer.rs:335) + impls.
**spawn.rs:** DELETE `SOCKET_PEER_TYPE_PATH` (:142) + the `SocketPeerCell` alias; keep
`PEER_TYPE_PATH`; update the `PeerCell` alias to the non-generic `Peer`.
**runtime.rs:**
- `send'` (22903): collapse `Peer'` (22995) + `SocketPeer'` (23033) arms → ONE `Peer'` arm
  = `peer.send(payload_val)` (no encoding). Keep Thread'/Process' arms.
- `recv'` (23090): collapse Peer'(23288)+SocketPeer'(23327) → ONE `peer.recv()`.
- `select'` 1-arg `Peer'` arm (23821): downcast → unified `Peer` cell → `as_any` rx →
  `&thread::Receiver<Value>` → `thread::Select` (crossbeam; unchanged behavior).
- `select'` 3-arg (23951): self-peer (23971) + client (24062) downcasts → unified `Peer`,
  same `as_any` recovery; crossbeam service loop (c0b1b) preserved.
- `connect'` (18178): `SocketAddress'` arm's `wrap_stream_as_socket_peer` → unified `Peer`
  (from_socket); thread `Address'` arm's `Peer { tx, rx }` (18244) → `from_thread`.
- `accept'` (18418): `SocketListener'` arm → unified `Peer`; thread `Listener'` arm → unified.
- `peer-pair'` (17865): two crossbeam `Peer` via `from_thread`.
- `socket-pair'` (17908): `socket_pair::<Value>()` (was `::<String>()`) → two `Peer` via
  `from_socket`; opaque tag PEER_TYPE_PATH (was SOCKET_PEER_TYPE_PATH).
- `wrap_stream_as_socket_peer` (18009): `sender_receiver_from_fd::<Value>` → unified `Peer`.
- self-peer: `eval_program_self_peer` (17577) + `process/verbs.rs:388` install → unified
  `Peer` under PEER_TYPE_PATH (was SOCKET_PEER_TYPE_PATH).
- `SelectEvent` `:Connection` via `wrap_connect_request` (24164) → unified `Peer`.
- send'/recv' error strings (23072, 23400): drop the `SocketPeer'<S,R>` alternative.
**check.rs:** every `"wat::kernel::SocketPeer'"` → `"wat::kernel::Peer'"`: `project_peer_io`
(10540) drop the `SocketPeer'` head arm + fix the error string; `infer_connect_prime`
(10168) SocketAddress'→`Peer'`; `infer_accept_prime` (10227) SocketListener'→`Peer'`;
`infer_socket_pair_prime` (~9959) →`Peer'` pair; `infer_program_self_peer` (~9993) →`Peer'`.

## The contract decision (pinned)

One `Peer` = `Box<dyn CommSender<Value>>` + `Box<dyn CommReceiver<Value>>`, one
`PEER_TYPE_PATH`. `send'`/`recv'` call the boxed trait (encoding internal). `select'`'s
connection-peer arms recover the concrete `thread::Receiver` via `as_any`. Spawn handles
`Thread'`/`Process'` stay DISTINCT (ownership ≠ connection). No socket `select'` (C0b.3a-ii).

## The gate (structural refactor — no wat RED)

This is a type/struct merge; the disconfirming fact is **structural**.
1. **Structural disconfirm:** `SOCKET_PEER_TYPE_PATH`, `wat::kernel::SocketPeer'`, and the
   `SocketPeer` struct are grep-PRESENT at HEAD, grep-EMPTY after.
2. **Regression (every arc-209 peer probe via the unified `Peer`):** `c0b1` (thread
   connect), `connection_primitive` (**1-arg `select'` over a crossbeam `Peer'` — the
   survival proof for 1-arg connection `select'`**), `c0b1b` (thread service 3-arg select'),
   `c0b2b` (socket round-trip), `c0b2c`/`c0b2d` (process connect / by-name), `c0b3a0`
   (self-peer) — all GREEN. (c0b2b proves the unified socket `Peer` round-trips a `Value`;
   `connection_primitive` + c0b1b prove crossbeam `select'` via `as_any`.)
3. Nursery serial **895/4** (baseline only) + full workspace test surface compiles (a type
   collapse is a recompile cascade — every binary builds; fail-count is the progress meter,
   `docs/SUBSTRATE-AS-TEACHER.md`).

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** the `as_any` downcast of a unified `Peer`'s `rx` to `&thread::Receiver<Value>`
   does not yield `Some` in the `select'` arm — STOP, report (would contradict the i-a probe).
2. **STOP-2:** `socket_pair::<Value>()` / `sender_receiver_from_fd::<Value>` do not type
   (Value not a legal socket wire `T`) — STOP, report (would contradict i-0).
3. **STOP-3:** a non-connection consumer reads `SOCKET_PEER_TYPE_PATH` / `SocketPeer` in a
   way the collapse can't subsume (a lifecycle path distinct from `Peer'`) — STOP, report.
4. **STOP-4:** a `select'` connection arm is reached by a *socket* peer (reactor_class `Fd`)
   in any existing test — STOP, report (means socket `select'` is reachable now and is not
   C0b.3a-ii after all; do not improvise a socket branch).

## Out of scope (rejected — NOT deferred)

- Socket `select'` (the 3-arg process service multiplexer: socket self-peer + listener-arm
  + `SelectEvent` over `process::Select`, consuming `reactor_class`) = **C0b.3a-ii**.
- `Listener'`/`SocketListener'` unification = **C0b.2e-ii**.
- `ServiceAddress<S,R>` sum (:Thread/:Process/:Remote) = **C0b.2e-iii**.
- Any remote `CommSender`/`CommReceiver` impl = a later organic addition.

## The deadlock contract carries

Each construction path preserves its transport's exact semantics (crossbeam rendezvous;
io_uring socket recv; non-blocking accept). A representation merge, no deadlock-surface
change; the boxed `Peer`'s `Drop` drops its concrete inner, which runs the same `Drop`.
[[feedback_vended_primitives_never_deadlock]] [[feedback_contract_not_encoding]]
