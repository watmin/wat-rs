# DESIGN-STONE C0b.2e-i — unify the connection `Peer` (the boxed peer-link seam)

> ⚠️ SUPERSEDED-IN-PART (2026-06-13, before build): the `PeerLink` trait below was a workaround for
> `HolonRepresentable`'s conflation. Builder surfaced the root: "we've been using holon as a crutch …
> we just need edn compliance." The real seam is **`EdnRepresentable`** (a prerequisite stone,
> **C0b.2e-i-0**): once `Value: EdnRepresentable` and the comms wire is bound on `EdnRepresentable`,
> the unified `Peer` is `Box<dyn CommSender<Value>>` over the EXISTING comms — no `PeerLink`. Read the
> "encoding decision" + "the seam" sections below as the *problem statement*; the *solution* is
> `EdnRepresentable` + `Box<dyn CommSender<Value>>`, drawn in C0b.2e-i-0 then this stone. The merge
> plan (collapse the dispatch arms, retire `SocketPeer'`, one `Peer<I,O>` checker head, the refactor
> gate) STILL HOLDS verbatim.

> First strike of the HEAVY unification (`DESIGN-STONE-C0b.2e-heavy-unification.md`). Merge the two
> connection-peer structs (`Peer` crossbeam / `SocketPeer` socket) into ONE `Peer` holding a boxed
> **peer-link trait** — operations closed, transports open. Retire `SocketPeer'`. After this a remote
> is a new link impl, not a new dispatch arm.

## The encoding decision (grounded, LOCKED)

`Value` is NOT `HolonRepresentable` (`comms/mod.rs:110`; the socket comms bound), and that trait's
default `to_wire` is *holon-tagged* EDN — using it as the wire would re-commit the
holon-tagging-as-transport abuse ([[feedback_contract_not_encoding]]: the wire is PLAIN EDN). So we
do NOT make `Value: HolonRepresentable`. Instead: a **Value-level peer-link trait** whose socket impl
encodes explicitly with the existing `value_to_edn` / EDN-decode (plain EDN), wrapping the existing
`comms` underneath. The encoding MOVES from the runtime arm INTO the socket link impl, so the arm
just calls `link.send(value)`.

## The seam

In `src/kernel/peer.rs`:
```rust
pub trait PeerLink: Send {
    fn send(&self, v: Value) -> Result<(), PeerSendError>;   // Crossbeam: direct; Socket: value_to_edn then send
    fn recv(&self) -> Result<Value, PeerRecvError>;          // Crossbeam: direct (+crash chan); Socket: recv then EDN-decode
    fn poll_fd(&self) -> Option<RawFd>;                       // Crossbeam: None (crossbeam-select); Socket: Some(read fd)
}
pub struct Peer { link: Box<dyn PeerLink> }                  // ONE connection peer; PEER_TYPE_PATH; retire SocketPeer/SOCKET_PEER_TYPE_PATH
```
Impls (this stone ships two; a remote later is a third, organically):
- **`CrossbeamLink { tx: comms::thread::Sender<Value>, rx: comms::thread::Receiver<Value>, crash: … }`**
  — `send`/`recv` direct (Value over crossbeam); `recv` surfaces the crash-channel on EOF exactly as
  today's `Peer` does; `poll_fd → None`.
- **`SocketLink { tx: comms::process::Sender<String>, rx: comms::process::Receiver<String> }`** —
  `send`: `value_to_edn(v) → tx.send(String)`; `recv`: `rx.recv() → EDN-decode → Value` (the codec the
  C0b.2b `send'`/`recv'` socket arms run today, MOVED here); `poll_fd → Some(rx read fd)`.

"Operations closed" = the three `PeerLink` methods. "Transports open" = the impls; a remote =
`impl PeerLink for RemoteX` (+ its connector). `send'`/`recv'`/`select'` call the trait and never
change. The existing `comms` `CommSender`/`CommReceiver` live UNDER each impl (the byte transport);
`PeerLink` is the Value-level layer that makes them look identical.

## The merge (collapse the dispatch — Explore map has every site)

- **`send'` / `recv'`** (`runtime.rs` `eval_peer_send_prime`/`eval_peer_recv_prime`): the 4 arms
  (`Thread'`/`Process'`/`Peer'`/`SocketPeer'`) collapse to 3 — `Thread'`, `Process'`, and ONE `Peer`
  arm = `peer.link.send(v)` / `peer.link.recv()`. The arm no longer EDN-encodes (the `SocketLink`
  does). (Spawn handles `Thread'`/`Process'` stay distinct — ownership ≠ connection.)
- **`select'`** (1-arg + 3-arg): the `Peer'`/`SocketPeer'` arms collapse to a `Peer` arm. The reactor
  is chosen by the peers' `poll_fd` homogeneity: all `None` → `comms::thread::Select`; all `Some` →
  `comms::process::Select` (register `poll_fd`s + the listener arm). Mixed transports in one `select'`
  → a clean error (a service's clients are one tier — not a representable-good state). `SelectEvent`'s
  `:Connection` carries the unified `Peer`.
- **`connect'`/`accept'`/`peer-pair'`/`socket-pair'`/`self-peer`** produce `Peer { link: … }`
  (Crossbeam for the thread tier; Socket for the process tier) instead of distinct `Peer'`/`SocketPeer'`.
- **Checker** (`check.rs`): ONE `Peer<I,O>` head. `project_peer_io` → `{Thread', Process', Peer}`
  (3-way). `infer_connect'/accept'/socket_pair'/peer_pair'/self_peer` → `Peer<I,O>`. `SelectEvent`
  `:Connection` → `Peer<I,O>`. Retire the `SocketPeer'` head.

## Gate-shape (a refactor — structural disconfirm, not a wat RED)

No pre-committed wat RED: this is a type/struct merge; the disconfirming fact is structural —
`SOCKET_PEER_TYPE_PATH` / `SocketPeer'` still grep-present at HEAD. The gate:
1. `SOCKET_PEER_TYPE_PATH` + `wat::kernel::SocketPeer'` grep-empty after the merge.
2. Every arc-209 peer probe green via the unified `Peer`: `c0b1b` (thread select service loop),
   `c0b2b` (socket round-trip), `c0b2c`/`c0b2d` (process/named connection), `c0b3a0` (self-peer).
3. `select'` over both tiers intact (c0b1b thread + the c0b2b/c0b2c socket peers).
4. Full nursery serial 895/4 (baseline only) + full workspace test surface compiles.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1 (the delicate one):** if `select'`'s reactor cannot be chosen by `poll_fd` homogeneity
   without a structural change to `comms::thread::Select`/`comms::process::Select` — STOP, report. (The
   reactors stay as-is; `select'` picks between them by the peers' `poll_fd`.)
2. **STOP-2:** if the crossbeam crash-channel (`Peer.crash`, surfaced on EOF) cannot be carried inside
   `CrossbeamLink::recv` without losing the structured death — STOP, report.
3. **STOP-3:** if any non-connection consumer reads `SOCKET_PEER_TYPE_PATH` in a way the merge can't
   subsume — STOP, report.

## Out of scope = rejected

- **Listener unification** — C0b.2e-ii. **`ServiceAddress` sum** — C0b.2e-iii. **Any remote link
  impl** — a later organic addition. **Merging `Thread'`/`Process'` spawn handles into `Peer`** —
  rejected (ownership ≠ connection).

## The deadlock contract carries

Each link impl preserves its transport's exact semantics (crossbeam rendezvous + crash channel;
io_uring socket recv; non-blocking accept). A representation merge, no deadlock-surface change.
[[feedback_vended_primitives_never_deadlock]]
