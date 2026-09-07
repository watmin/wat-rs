# DESIGN — a caller chunks to the declared limit

**`<op>-all`, generated.** `wat/service.wat` + `wat-scripts/queue/sqs.wat` +
`wat-scripts/topic/sns-fanout.wat`. The first piece of a **userland layer** over the rigid
surface/service tooling.

## WHY — the topic honours its own contract and DoS-es its downstream

```
topic   declares  :max-entries [msgs 10]        ← honours its inbound contract ✓
queue   declares  (nothing)                     ← removed in `the server manages its own capacity`
topic   sends     10 x nsubs bodies in ONE call ← violates the outbound one
```

Measured, at constant total work (8000 pairs both sides):

```
n=2000 m=4   publish 18472   retries  540   receives  4687
n=1000 m=8   publish 61431   retries 1652   receives 10387    ← 3.3x
```

★★ The mechanism is not fan-out cost. `10 msgs × 8 subs = 80 bodies > inbox cap 64`, so **every
publish takes the prefix path** and fragments — the regime already measured at ~3× when partial
admission landed.

⚠ And the flaw is mine twice over: I removed `Queue::send`'s `:max-entries` because it "rejected
what admission would take partially", which treated the symptom. The caller was sending an
oversized request; **the fix was never to delete the callee's limit.**

## ⛔ THE RULE — limits are contract; nobody raises one to fit a caller

SNS 10, SQS 10, DynamoDB 25. These are declared bounds that protect a service. **The instinct to
raise one means the flaw is elsewhere.** The generic rule this stone installs:

> **A service that fans a bounded request into a larger internal batch must chunk to the
> downstream's declared limit.**

And the limit itself is *derivable*, not chosen:

> **A queue's batch limit cannot exceed its capacity** — a batch larger than `cap` can never land
> whole, so declaring one promises something impossible.

★ That is why `[bodies 64]` on a `cap 64` queue is principled where the `10` I copied from a
public HTTP API was cargo-cult.

## ⛔ THE ONE CONTRACT DECISION — the caller must NOT know the limit

The naive form — the topic reads `:queue::Queue::SEND-MAX-ENTRIES` and chunks — puts a
downstream's number inside upstream business logic. Instead the **generated method** chunks, using
the limit it already resolves at expand time (exactly as `:max-request-bytes` is already enforced
before the send). The topic writes:

```wat
(:queue::Queue/send-all q (:queue::Queue::SendRequest :bodies <80 bodies> …))
```

and never learns whether the queue said 15, 64 or 10. **Change the limit; not one line upstream
changes.**

## WHY IT COMPOSES — a payoff of the prefix decision

`Accepted [count]` means *"the first count landed"*. So `send-all` sends chunks **in order**, sums
the counts, and **stops at the first short chunk** — the sum is still a true prefix. One
`Accepted n` comes back whether it took one round trip or six.

★★ Had we taken SQS's `Successful`/`Failed` id lists, chunking would need id-list merging. The
prefix choice makes it arithmetic.

## ⛔ WHERE THE HELPER IS EMITTED — and where it is NOT

Emit `<op>-all` **iff** the op declares `:max-entries [field N]` **and** its response declares
`:Accepted [count <- i64]`.

```
Queue::send          :max-entries ✓   Accepted ✓   →  send-all     emitted
Topic::publish       :max-entries ✓   Accepted ✓   →  publish-all  emitted
Store::put           :max-entries ✗   :Success []  →  nothing      emitted
```

★★★ `Store::put` is genuinely all-or-nothing and must **not** get a chunker. The two declarations
that make chunking sound are exactly the condition for emitting it — the helper cannot exist where
its contract does not.

## THE LAYERS, NAMED

| layer | what it is |
|---|---|
| **surface + service** | the wire contract. Declares limits, **rejects** violations, one round trip, no policy. Rigid on purpose — this is what protects against a DoS. |
| **userland** (`-all`) | makes the bound invisible without violating it. Opt-in **by name**, so a call site shows which one you took. |

⚠ `send` still rejects over-limit with `RequestTooManyEntries`. **The wall stays a wall.** Only the
named helper chunks — the same split AWS uses between a raw API and its higher-level clients.

## OUT OF SCOPE — REJECTED, not deferred

- **The read side** (`scan-all` over `next-cursor`, yielding `:wat::stream::Stream`). The symmetric
  half, and its payoff is correctness not speed: **`sqs.wat:169` passes `:cursor None` and nothing
  in the tree ever follows a cursor.** Its own stone.
- **Migrating `circuit.wat`'s eight hand-rolled helpers** (`publish-until-accepted!*`,
  `drop-first`, `backoff-delay`, …) onto this layer. That is the *prefix-following and backoff*
  face; this stone is the *chunking* face. Named because that code has **already been duplicated
  once** into the publisher child, which is why its bugs kept surfacing.
- **Init-validating `N <= cap`.** Real — a limit above capacity re-creates the livelock — but a
  separate invariant with a separate mechanism.
- **Raising any cap or any limit.** The whole point.
