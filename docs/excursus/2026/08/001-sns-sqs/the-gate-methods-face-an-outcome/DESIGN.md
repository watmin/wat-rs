# DESIGN — the gate methods face an outcome

Builder: *"let's draw grant"*, closing the owner-method class this track opened.

**Drawn 2026-09-11. NOT STRUCK.** Stone 3 of `a-momentary-failure-is-not-fatal`.

## Why

`the-owner-faces-an-outcome` gave `stop` and `hibernate` an outcome and a bounded wait. It
**deliberately scoped `grant` out** on the grounds that it *"returns `nil` and matches
`peer-process`, a different shape — measure before assuming it moves."* Measured: the shape is
`stop`'s, wrapped in a tier check.

```
grant-method-body = (match (peer-process handle)
  ((Some _)  ;; PROCESS tier — the Admin::AllowPeer round-trip
    send AllowPeer[pids]; recv:
      Message(Status::PeersAllowed) → nil
      Message(other)                → raise  (protocol violation — correctly stays)
      Lost(cause)                   → raise
      Stopped                       → raise "…the service was ALIVE…"      ← says alive, then dies
      Closed                        → raise "service peer closed during grant"
      TimedOut                      → raise "the peer is alive and silent" ← says alive, then dies)
  ((None) nil))  ;; THREAD tier — "the handle IS the grant", no admin message at all
```

Four non-`Message` arms raise; **two announce the peer is alive in the message that kills the
process**, exactly as `stop` did this morning.

## ⭑ AND IT HAS A TWIN, ASKED BEFORE THE STRIKE THIS TIME

`revoke` (the `Admin::DenyPeer` mirror, `wat/service.wat:3106`) is **the identical shape** — same
`peer-process` check, same four raises, `Status::PeersDenied` instead of `PeersAllowed`. **Both are in
scope.** This campaign found five sibling-pairs in one day, every one of them *after* a floor red or a
wrong claim; this is the first time the question was asked first.

With these two, the owner-method class closes: `stop` ✓ `hibernate` ✓ (stone 2) · `grant` · `revoke`.

## ⛔⛔ THE CONTRACT DECISION — RE-RECV, NOT RE-ASK, AND THE REASON IS NOT SYMMETRY

Stone 2's mechanism was forced: `Admin::Stop` **terminates** the service, so a re-send reaches a dead
peer. **`AllowPeer` is the opposite** — read from the serve arm (`wat/service.wat:2330`):

> *"fold `(allow' l pid)` over the vec … ack `PeersAllowed` up the lineage peer … then **CONTINUE
> serving (recur — no state change)**"*

So the service survives, and `allow` is a set-add over pids — **idempotent**. `grant` genuinely *can*
be re-asked. Inheriting stone 2's shape by symmetry would have been an unexamined assumption, which is
this campaign's most expensive habit.

**And a re-ask is still the wrong choice.** Two facts decide it:

1. **The serve loop never retries its ack.** Its `send PeersAllowed` faces `Closed`/`Lost` and
   *continues serving regardless*. So a lost ack means **no ack will ever arrive**, and a re-recv-only
   loop burns its whole budget and reports `GaveUp` on a case a re-send would have recovered. That is a
   real cost of re-recv, and it is the honest argument *for* re-ask.
2. ⛔ **But a re-ask can leave a surplus ack queued on the lineage peer.** If the first ack was merely
   *slow* rather than lost, re-sending produces two, the loop reads one, and the second stays on the
   wire. A later `<S>/stop` then does `send Admin::Stop; recv` and picks up the stale `PeersAllowed` →
   `Message(other)` → `"defservice stop: expected Status::Stopped"`. **That is character-for-character
   the frame-desync class that produced today's crash**, and it is silent until it detonates somewhere
   else.

**Ruling: re-recv.** A reported `GaveUp` is strictly better than a silent desync — it is named, faced,
and local, where the surplus ack is none of those. ⚠ The cost is real and stated: **a lost ack becomes
`GaveUp`, not a recovered grant.** If that proves to matter in practice, the fix is to make the serve
loop's ack retryable — *not* to make the client re-ask.

## The outcome type

One enum for both methods, payload-free:

```
(:wat::core::defenum :wat::service::GateOutcome :wat::enum::Pure
  :Applied []                                              ;; the gate changed; ack received
  :Gone    [cause <- :wat::kernel::LociDiedError]           ;; Closed / Lost — the peer is gone
  :GaveUp  [waited-ms <- :wat::core::i64  last <- :wat::core::String])
```

★ **Not `StopOutcome :- [nil]`.** Reusing it would name a grant's success arm `Stopped`, which is a
type lying about what happened — the defect this whole track exists to remove. `Applied` is honest for
both `grant` and `revoke` (both apply a change to the accept gate), so the two methods share one enum
rather than minting two.

**The THREAD arm returns `Applied` immediately.** On the thread tier the handle *is* the grant — there
is no message, nothing to wait for, and nothing that can fail. It must not pretend to have waited, and
it must not return a distinct "not applicable" variant: the gate genuinely is in effect.

## Out of scope = REJECTED

- **Generalizing `StopOutcome` + `GateOutcome` into one `OwnerOutcome :- [T]`.** Tempting — the shapes
  are `Ok | Gone | GaveUp` twice — and rejected for now: it churns `StopOutcome`'s 45 freshly-migrated
  call sites, and `:Ok` loses the domain naming (`Stopped` / `Applied`) that makes a match readable.
  **Stated here so the builder can overrule cheaply**, because it is a real simplification if he wants
  the churn.
- **Making the serve loop's ack retryable.** Named in the contract decision above as the correct fix
  *if* lost acks prove to matter. Not this stone.
- **`owner-recv-loop`'s dead `TimedOut` arm.** Tracked at
  `docs/arc/2026/04/109-kill-std/NOTE-an-outcome-variant-no-primitive-can-construct.md`, sequenced
  after this stone by the builder. This stone inherits the same limitation and **must not claim
  otherwise**: `GaveUp` will be reachable only while the peer keeps emitting outcomes.

## Blast radius, measured across all three carriers

```
(:ns/grant  …)    68 occurrences / 33 files    (top: circuit.wat 18, sns-fanout.wat 5)
(:ns/revoke …)     4 occurrences
.rs / .jsonl       0
generated bodies   2   wat/service.wat grant-method-body :3056, revoke-method-body :3106
```

⚠ My earlier figure of "45 sites" for `grant` was from a looser pattern and was wrong; **68** is the
counted number. Third time this session a census of mine grew on re-measurement — count with
`grep -o … | wc -l`, never `grep -c`.

## Trap-doors named up front

1. **`AllowPeer` is re-askable and `Stop` is not.** Do not copy stone 2's mechanism *for its reason*;
   copy it for the desync argument above.
2. **The thread arm has no round-trip.** It must return `Applied`, not fall through the loop.
3. **`revoke` is the twin.** Changing one and not the other is the sixth sibling-pair of the day.
4. **Three carriers of wat source.** Two floor reds came from missing `tests/**/*.wat` and wat inside
   `.rs`/`.jsonl`. Here both are 0 — *verified*, not assumed.
5. **`cargo build --release` does not compile tests.** Use `cargo nextest run --release --no-run`.
