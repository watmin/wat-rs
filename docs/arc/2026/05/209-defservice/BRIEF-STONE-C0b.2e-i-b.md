# BRIEF — Stone C0b.2e-i-b: the unified `Peer` collapse (retire `SocketPeer'`)

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/`
(verify `pwd` first; operate only here; `git -C /home/watmin/work/holon/wat-rs`). Design:
`DESIGN-STONE-C0b.2e-i-b-unify-peer-collapse.md` (read it fully). Foundation already
shipped (i-a `aac27fb5`): `ReactorClass`, `CommReceiver::reactor_class`/`as_any`. Do NOT
commit — the Inquisitor weighs.

## The work in one paragraph

Collapse the two connection-peer types into ONE. `Peer<Value,Value>` (crossbeam) and
`SocketPeer<String,String>` (socket) both become the single non-generic
`Peer { tx: Box<dyn CommSender<Value>>, rx: Box<dyn CommReceiver<Value>> }`, built via
`Peer::from_thread` / `Peer::from_socket`. The socket side switches `String`→`Value`
(`process::Sender<Value>` encodes internally now), so `send'`/`recv'` DELETE their
arm-level `value_to_edn` encoding. Every `SocketPeer'` / `SOCKET_PEER_TYPE_PATH` /
`SocketPeer` is retired — the wat type path is always `:wat::kernel::Peer'`. The compiler
cascade is your guide: this is one uniform substitution (`SocketPeer' → Peer'`) repeated
across ~70 sites, plus deleting the now-redundant socket codec. The fail-count is the
progress meter.

## Read in order (the rooms)

1. `src/kernel/peer.rs:182` `Peer<S,R>` + `:335` `SocketPeer<I,O>` — redefine `Peer`
   (boxed, non-generic) with `from_thread`/`from_socket` + `send`/`recv` calling the box;
   DELETE `SocketPeer` and its impls.
2. `src/kernel/spawn.rs:142` `SOCKET_PEER_TYPE_PATH` + the `SocketPeerCell` alias + the
   `PeerCell` alias — DELETE the socket ones; point `PeerCell` at the non-generic `Peer`.
3. `src/runtime.rs` — the verb arms (DESIGN's site map has every line):
   `send'` (22995+23033 → one Peer arm), `recv'` (23288+23327 → one), `select'` 1-arg
   Peer' arm (23821), `select'` 3-arg (23971/24062), `connect'` (18178), `accept'` (18418),
   `peer-pair'` (17865), `socket-pair'` (17908), `wrap_stream_as_socket_peer` (18009),
   `eval_program_self_peer` (17577), `wrap_connect_request` (24164), error strings (23072,
   23400).
4. `src/process/verbs.rs:388` — the self-peer install (SOCKET_PEER_TYPE_PATH → PEER_TYPE_PATH).
5. `src/check.rs` — `project_peer_io` (10540), `infer_connect_prime` (10168),
   `infer_accept_prime` (10227), `infer_socket_pair_prime` (~9959), `infer_program_self_peer`
   (~9993): every `"wat::kernel::SocketPeer'"` → `"wat::kernel::Peer'"`.

## Implementation sketch (fill the shape; the compiler cascade guides the rest)

**(A) `peer.rs` — the unified struct:**
```rust
pub struct Peer {
    tx: Box<dyn crate::comms::CommSender<crate::value::Value>>,
    rx: Box<dyn crate::comms::CommReceiver<crate::value::Value>>,
}
impl Peer {
    pub fn from_thread(tx: crate::comms::thread::Sender<crate::value::Value>,
                       rx: crate::comms::thread::Receiver<crate::value::Value>) -> Self {
        Self { tx: Box::new(tx), rx: Box::new(rx) }
    }
    pub fn from_socket(tx: crate::comms::process::Sender<crate::value::Value>,
                       rx: crate::comms::process::Receiver<crate::value::Value>) -> Self {
        Self { tx: Box::new(tx), rx: Box::new(rx) }
    }
    pub fn send(&self, v: Value) -> Result<(), SendError<Value>> { self.tx.send(v) }
    pub fn recv(&self) -> Result<Value, RecvError>              { self.rx.recv() }
}
```
**Doc-honesty (intueri):** the existing `Peer` doc (peer.rs:172) says "Pipes-only
bidirectional **worker self-peer**" — that is a stale Level-1 lie: `Peer` is the general
crossbeam-or-socket bidirectional connection endpoint, used as BOTH a worker self-peer AND
a connection handle (peer-pair'/connect'/accept'). Rewrite the doc to say what it now is:
the unified, transport-blind (`Box<dyn CommSender/Receiver<Value>>`) connection/self peer;
the self-vs-connection role is positional at the call site (e.g. `select'`/`poll'` arg 0),
not a type distinction. Do NOT rename it to `SelfPeer` — `Peer` is the correct unified name.

**(B) `send'`/`recv'` — collapse the two connection arms into one (Thread'/Process' stay):**
```rust
// send' Peer' arm (was: Peer' direct + SocketPeer' value_to_edn). Now ONE arm:
Some(peer) => peer.send(payload_val) ...   // box encodes internally for socket; NO value_to_edn
// recv' likewise: peer.recv() → Value (decode is internal to the boxed process::Receiver)
```

**(C) `select'` 1-arg `Peer'` arm — recover the concrete receiver via `as_any`:**
```rust
match peer.rx.as_any().downcast_ref::<crate::comms::thread::Receiver<Value>>() {
    Some(rx) => { /* register rx with thread::Select, exactly as the old Peer' arm did */ }
    None => return Err(/* clean MalformedForm: non-crossbeam connection peer in 1-arg select'
                          — socket/remote connection select' is Stone C0b.3a-ii */),
}
```
(3-arg `select'` self-peer + client downcasts use the same `as_any` recovery; crossbeam.)

**(D) constructors — build the unified `Peer`, switch socket to `Value`:**
- `peer-pair'`: `Peer::from_thread(a_tx, b_rx)` / `from_thread(b_tx, a_rx)`, tag PEER_TYPE_PATH.
- `socket-pair'`: `comms::process::socket_pair::<Value>()` (was `::<String>()`) →
  `Peer::from_socket(...)`, tag PEER_TYPE_PATH (was SOCKET_PEER_TYPE_PATH).
- `wrap_stream_as_socket_peer`: `sender_receiver_from_fd::<Value>` → `Peer::from_socket`.
- `connect'`/`accept'`: socket arms → `from_socket`; thread arms → `from_thread`.
- self-peer: `from_socket`, PEER_TYPE_PATH.

**(E) `check.rs`** — replace every `"wat::kernel::SocketPeer'"` head string with
`"wat::kernel::Peer'"`; in `project_peer_io` drop the now-duplicate `SocketPeer'` arm and
fix the `expected:` string (drop the `| SocketPeer'<S,R>` alternative).

Then `cargo build` and follow the cascade. For a wide mechanical pass, edit in place
surgically (read → targeted replace → write); do not whole-file-rewrite runtime.rs/check.rs.

## Blast radius

`src/kernel/peer.rs`, `src/kernel/spawn.rs`, `src/runtime.rs`, `src/check.rs`,
`src/process/verbs.rs`. No new wat surface, no new probe (the gate is structural + the
existing arc-209 probes). `comms/` is untouched (i-a already shipped the foundation).

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** `as_any` downcast of a unified `Peer`'s `rx` to `&thread::Receiver<Value>`
   yields `None` in a `select'` arm exercised by an existing test — STOP, report (would
   contradict the i-a probe; the live connection peers are crossbeam).
2. **STOP-2:** `socket_pair::<Value>()` / `sender_receiver_from_fd::<Value>` don't type —
   STOP, report (would contradict i-0).
3. **STOP-3:** a non-connection consumer reads `SOCKET_PEER_TYPE_PATH` / `SocketPeer` in a
   way the collapse can't subsume — STOP, report.
4. **STOP-4:** any existing test reaches a `select'` connection arm with a *socket* peer
   (reactor_class `Fd`) — STOP, report (means socket connection `select'` is reachable now;
   do NOT improvise a `process::Select` branch — that is C0b.3a-ii).

## The gate

```
# structural disconfirm — these must be EMPTY after:
grep -rn "SOCKET_PEER_TYPE_PATH\|wat::kernel::SocketPeer'\|struct SocketPeer\|SocketPeerCell" src/
cargo build --release
# regression — every arc-209 peer probe via the unified Peer:
cargo test --release -p wat --test nursery connection_primitive probe_arc209_c0b1b probe_arc209_c0b2b probe_arc209_c0b2c probe_arc209_c0b2d probe_arc209_c0b3a0 -- --test-threads=1
cargo test --release -p wat --test nursery -- --test-threads=1        # 895 passed / 4 failed (baseline)
cargo test --release --workspace --no-run                             # full surface compiles
```
Report each exact `test result:` line + the grep output (must be empty) + any STOP/honest
delta. Do NOT commit.

## Prior comparable (copy the shape)

`BRIEF-STONE-C0b.2c-*` (the process connection verbs — same files: peer.rs/spawn.rs/
runtime.rs/check.rs, same opaque-cell + downcast + type-path patterns) and i-a
(`BRIEF-STONE-C0b.2e-i-a.md`, the comms foundation this consumes).
