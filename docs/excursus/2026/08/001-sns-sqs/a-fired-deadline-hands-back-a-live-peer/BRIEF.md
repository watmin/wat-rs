# BRIEF — a fired deadline hands back a live peer

**Read `DESIGN.md` beside this first.** It carries the one contract decision (three outcomes, all from
existing variants), the gap the orchestrator put into stone 1 that this stone finishes, three rejected
alternatives, and six trap-doors — the first of which is the reason a hidden mutation is acceptable
here and nowhere else.

## The work, in one paragraph

When `call-by-deadline`'s timer wins, it abandons the in-flight reply on the wire and returns the
**same** peer, so every later reply on that connection is shifted — permanently, silently, on both
tiers. Make the deadline path re-dial: read `dialed-from`, `connect` it, and **replace the peer inside
its cell** so every holder of that handle is working again. `DeadlineFired` keeps its shape and gains a
promise; a failed redial answers `Lost [cause]` instead. Then finish stone 1 by teaching the
**thread** dialer to remember its address, so the repair is uniform rather than half.

## The rooms

| where | why you are going there |
|---|---|
| `wat/service.wat:4210` `call-by-deadline` | the primitive. Its three `DeadlineFired` constructions are `:4233`, `:4237`, `:4241`. |
| `wat/service.wat:2733` | the **generated client method's** `DeadlineFired` arm → `RecvOutcome::TimedOut`. ⭑ Read it to see why a carried peer could never work: `TimedOut` is nullary. You are not changing this line. |
| `src/kernel/spawn.rs:142` | `pub type PeerCell = Arc<ThreadOwnedCell<Option<Peer>>>` — the fact the whole design rests on. |
| `src/runtime.rs:25845` | ⭑ THE PRECEDENT for mutating a peer cell: `with_mut(OP, span, \|opt_peer\| opt_peer.take())`. |
| `src/rust_deps/custodia.rs:71` `with_mut` · `:84` `with_ref` | the cell API. `dialed-from` used `with_ref`; the swap needs `with_mut`. |
| `src/kernel/address.rs:97` `ThreadAddress` · `:106` its `connect` | ⛔ the gap stone 1 baked in. A thread dial HAS an address. |
| `src/kernel/address.rs:249`–`:255` | how the socket dialer already does it — `Some(self.clone())`, with the comment. Mirror it for threads. |
| `src/kernel/peer.rs:340` `from_thread` | gains the address argument, required — not defaulted. Stone 1's required-argument latch found a site its census missed; keep that latch. |
| `src/kernel/address.rs:303` | `Address { inner: Box::new(ThreadAddress { tx }) }` — `Address` is transport-blind, so `dialed-from`'s type needs no change. |
| `wat-scripts/scratch-pad/probe-a-slow-peer-desyncs-the-next-call.wat` · `…-thread-tier.wat` | ⭑ THE TWO ACCEPTANCE FIXTURES. Both print the shift today; both must stop. |
| `wat-scripts/fanout/circuit.wat:1000` | the hand-written redial that has been right all along — the disposition this generalizes. |

## Implementation sketch

```wat
;; in call-by-deadline, where the timer wins
(:wat::core::match (:wat::kernel::dialed-from peer)
  ((:wat::core::Some addr)
    (:wat::core::match (:wat::kernel::connect addr)
      ((:wat::kernel::ConnectOutcome::Connected fresh)
        <swap `fresh` into the peer's cell>            ;; every holder is repaired
        (:wat::service::CallOutcome::DeadlineFired))
      ;; the peer is GONE, not merely slow — say so
      (_ (:wat::service::CallOutcome::Lost <cause>))))
  ;; after the thread-dialer fix this arm is reachable only for an ACCEPTED or
  ;; self peer, which a client never calls through. Decide it deliberately and
  ;; say what you chose — see STOP-3.
  (:wat::core::None <…>))
```

The swap itself is Rust-side, beside `dialed-from`: `with_mut`, replacing `Some(old)` with
`Some(fresh)`. ⭑ **The old peer must be dropped, not leaked** — it holds an fd.

## Blast radius

`wat/service.wat` (one function), `src/kernel/peer.rs`, `src/kernel/address.rs`, plus the `from_thread`
call sites and one new intrinsic for the swap. **The 13 existing `DeadlineFired` match arms: 0.**
Confirm that rather than inheriting it.

## STOP triggers

1. ⛔ **STOP-1 — if the swap cannot be made observable**, STOP and report it. A corruption traded for
   an invisible reconnect is a bad trade, and trap-door 1 accepts hidden mutation only on the
   condition that a reader can see it happened.
2. ⛔ **STOP-2 — if a failed redial cannot be distinguished from a successful one**, STOP. `Lost` vs
   `DeadlineFired` carries the entire new contract; collapsing them rebuilds the defect one level up.
3. **STOP-3 — decide the `dialed-from == None` arm deliberately and say what you chose.** After the
   thread fix it should be unreachable from a client call. If it is reachable, it must not be
   `DeadlineFired` — that variant now promises a live handle.
4. **STOP-4 — if the swap must happen off the cell's owning thread**, STOP. `ThreadOwnedCell` is
   thread-owned by construction.
5. **STOP-5 — the publisher and `recv-by-deadline` are OUT OF SCOPE.** The publisher is stone 3.
   `recv-by-deadline` may share the defect and is **unmeasured** — say so, do not fix it here and do
   not claim it is fine.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run`.
- `./scripts/floor.sh`, read the **Summary line**, never a piped exit code.
- ⭑⭑ **BOTH acceptance fixtures, and both must flip:**
  `probe-a-slow-peer-desyncs-the-next-call.wat` (process) and `…-thread-tier.wat` (thread).
  Today each prints `Answered(tag=11) sent=22`. After: **every call gets its own tag back.**
- ⭑ **The negative control:** kill the service, then fire a deadline against it — the answer must be
  `Lost`, never `DeadlineFired`. A redial that cannot succeed must not be reported as a repair.
- ⭑ **The original handle works after the swap** — that is the whole point; the caller never learns a
  new name for its peer.
- Happy path unchanged: `printf '#fanout/Input {…}' | … circuit.wat` → `distinct=8000;dup=0`.
- ⚠ Report the redial's **cost** on the deadline path (trap-door 3).

## Shape to copy

`a-peer-remembers-its-address/` — stone 1, immediately prior: the required-argument latch, the
`with_ref` peek, and `infer_dialed_from`'s parametric threading. And `circuit.wat:1000` for the
disposition this generalizes out of one hand-written place into the primitive.
