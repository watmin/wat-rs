# DESIGN — the store can fail

Builder: *"draw #1"* — ranked first in
`the-crash-surface-is-enumerated/FINDING-the-crash-surface.md` §5.

**Drawn 2026-09-12. NOT STRUCK.**

## Why

Six arms of `wat-scripts/queue/sqs.wat` are live code that **has never executed**:

```
queue.send  store put     Lost :780   Closed :814   TimedOut :845
queue.ack   store delete  Lost :1215  Closed :1248  TimedOut :1279
```

They are §2d — *the unknowable write*: the rows **may** have landed, the handler cannot know. Their own
comment says *"Do not claim Accepted n — the put is unknowable."* Measured zero on my healthy **and** my
chaos run: `inbox-lost=0 inbox-closed=0 inbox-timedout=0`. The queue's only fault knobs
(`drop-recv-bp`/`drop-ack-bp`, `sqs.wat:202–203`) suppress **the queue's reply to its own caller** — they
cannot fail a store call.

This blocks two things: the **`:Unknown` A/B ruling** the builder deferred, and the banked **−17.1 %
`rt-store`** patch behind it.

## ⭑⭑ THE SHAPE: A FAULTING PROXY, NOT A KNOB IN THE STDLIB

`sqs.wat:201` declares `store-addr <- (Address :- [Store::Op Store::Reply])` and `connect`s to it
(`:248`, `:251`). **The queue takes an ADDRESS.** So a userland service that satisfies
`:wat::query::Store` and forwards to the real store can be substituted **with zero changes to `sqs.wat`
and zero changes to the stdlib.**

★ That is the whole design. `wat/query.wat`, `wat/query/mem.wat` and `wat/query/sqlite-store.wat` are
**untouched** — no chaos knob enters the stdlib store, which would be a permanent fault surface in a
production type for the benefit of one harness.

Cost: the proxy must forward **six** features — `ensure-schema · put · delete · count-index · scan ·
scan-index` (counted from `wat/query.wat`'s `defsurface`).

## ⭑ The fault must LOSE THE REPLY, never skip the call

This is the one thing that makes the stone honest, and it is easy to get backwards.

> §2d is *"the write may have landed and I cannot know."* A fault that **skips** the store call produces
> the opposite state — the write definitely did **not** land. That exercises the arm while
> **contradicting** the condition the arm exists for.

So the proxy **forwards the op to the real store, waits for the real reply, and then destroys its own
answer.** The write lands. The caller cannot know. That is exactly the shape `drop-recv-bp` already uses
one tier up — the FINDING's words: *"it performs the delete and suppresses the reply, which is precisely
what makes the caller retry."*

## Two knobs, because three arms need two mechanisms

| knob | proxy behaviour | queue observes | cost |
|---|---|---|---|
| `store-drop-reply-bp` | forward, get the real reply, **suppress it**, keep serving | **`TimedOut`** | ⚠ **10 s per fault** — see below |
| `store-die-bp` | forward, get the real reply, then **exit** | **`Lost`** / **`Closed`** | prompt (socket close is an event) |

Together they reach all three §2d arms. `die` is terminal for that tier — which is a real failure mode,
not a defect of the knob — so it is for small runs, while `drop-reply` is survivable and repeatable.

## ⛔ THE COST IS FIXED AT 10 s AND I PROBED WHY

I assumed the proxy could declare a short deadline. **It cannot.**
`wat-scripts/scratch-pad/probe-deadline-ms-is-callee-derived.wat` — a `:satisfies`-mode service declaring
`:deadline-ms 300`, whose handler parks on a 1-hour timer:

```
outcome=TimedOut;elapsed-ms=10000;declared=300
```

**A callee's `:deadline-ms` does not change what its caller waits.** The generated client method uses the
default 10 000 ms (`wat/service.wat:2442` — *"Optional :deadline-ms on the service. Default 10000. Never
off"*), and neither store overrides it (`grep`: no `:deadline-ms` in `query.wat`, `mem.wat`,
`sqlite-store.wat`).

★ **That is a finding beyond this stone:** a declared clause that silently does nothing in
`:satisfies` mode. It is **not** this stone's job to fix — but it must be filed, because the next person
to reach for `:deadline-ms` to bound a call will find it inert. Where the clause *is* honoured is unmeasured
and deliberately not guessed at here.

**Consequence for the harness:** size the run around 10 s per `drop-reply` fault. At `n=50` with one or
two faults that is fine; at `n=2000` with a 5 % rate it is not a harness, it is a hang.

## The one contract decision

> **The proxy is a pass-through with a fault rate, never a reimplementation.** Every non-faulting op
> forwards and returns the real store's real reply, byte-for-byte. The proxy holds **no state** except
> its counters and its RNG.

A proxy that answers from its own store would be a second store implementation and a second source of
truth — the exact defect `mem-store`-vs-`sqlite-store` differential testing exists to catch.

## The four questions

**Obvious?** YES — a pass-through that sometimes eats its own reply. **Simple?** YES — six forwards, one
dice roll, two counters; no stdlib change, no new outcome type, no Rust. **Honest?** YES, and it is the
crux: the write really lands, so the unknowable state is genuine rather than simulated. **Good UX?** YES —
substituted by address, so any existing harness can point at it without edits.

## Scope

**IN:** the proxy service in `wat-scripts/` satisfying `Store`, forwarding all six features · the two
knobs, each with a **fire counter at the suppression site, not at the dice roll** · a probe that makes
**each of the three §2d arms fire and prints which** · the arms' counters reported.

**OUT = REJECTED:**
- ⛔ **Ruling `:Unknown`.** This stone makes the measurement possible; the A/B ruling stays the builder's.
- ⛔ **Landing `PATCH-measure-variant.diff`.** Still behind `:Unknown`.
- ⛔ **Migrating the six arms off `assertion-failed!`.** They are among the 61 placeholders. This stone
  makes them *fire*; what they should then *do* is the next ruling. ⚠ **They will raise when fired — that
  is the expected, correct result of this stone, not a failure.**
- ⛔ **A chaos knob in `wat/query/*`.** Rejected on the reasoning above.
- **Fixing `:deadline-ms`'s inert clause.** Filed, not fixed.

## Trap-doors

1. ⛔ **Skipping the call instead of losing the reply** — inverts the condition under test (§"must LOSE
   THE REPLY").
2. ⛔ **10 s per `drop-reply` fault.** Size the run for it. A 2000-message run with a live rate will look
   like a hang, and someone will call it a deadlock.
3. **Count at the suppression site, not the dice roll.** The chaos-rate stone earlier in this campaign
   shipped an injector whose counter could not distinguish *never fired* from *fired and did nothing* —
   and `disrupt-hits` still cannot (`ec3ea95c8`). Do not add a third.
4. **Six features, not one.** A proxy that forwards `put`/`delete` and stubs `scan` will pass a shallow
   probe and corrupt a real run.
5. **The proxy adds a hop** — +2 crossings per store op. **Never measure `rt-store` through it.** It is a
   chaos instrument, not a perf one.
6. **The six arms raise today.** When they fire, the tier dies. Expected (see Scope). Report *which* arm
   and *what it printed* — that output is the stone's product.
