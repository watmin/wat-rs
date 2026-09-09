# DESIGN — the consumer is idempotent

Drawn 2026-09-03. **Not struck.** Side quest S13, promoted: it is a precondition for main-line
item 3, not an optional cleanup.

## Why — and it is worse than "add a dedupe"

The circuit asserts **exactly-once** on an **at-least-once** system. When a topic-worker's
`Queue/send` + ack exceeded its visibility window under a loaded floor, the row reappeared, a second
worker sent the same message, and the floor went red at `total: 26` against `24`.

Widening the window (200 ms → 5 s) is correct SQS configuration and **does not remove the class** —
the assertion still rests on a timing margin, and this repo's doctrine has no room for a test that
reds without a bug.

★ **But the sharper finding is that our duplicate detector cannot see this duplicate.**

`send` mints a fresh uuid per call (`sk = edn::write(uuid::v4)`), so a re-sent message becomes a
**new envelope with a new id**. `distinct` keys on `queue/envelope-id` (`key-of`, `circuit.wat:441`).
So on a redelivery **`total` and `distinct` rise together and `dup` stays 0.**

The invariant this arc has quoted all day — `total=8000; distinct=8000; dup=0` — detects *the same
envelope acked twice*, which is nearly tautological. It does **not** detect *the same message
delivered twice*, which is the only kind at-least-once actually produces. The red was caught solely
because a small-fixture test compares `total` to a constant; the 2000×4×3 run asserts nothing and
has been read by eye.

★ `:fanout::body-key` — keyed on the body, defined in two files, **called nowhere** (side quest S9)
— was written for exactly this and never wired. The instrument existed and was never connected.

## What it delivers

1. A **stable message identity**, assigned at publish and carried end to end.
2. **Consumers that dedupe on it** — the same message delivered twice produces one outcome.
3. An invariant that **counts message identities, not envelope ids**, so it can see what it claims
   to see.
4. As a consequence: the visibility window can be tightened back, and the test stops depending on a
   timing margin.

## The one contract decision: identity is assigned by the PUBLISHER, not the queue

An envelope id is a *transport* fact — minted per `send`, different for every retry. A message id is
a *domain* fact — one per published message, stable across every redelivery and every subscriber.

They are different things and the arc has been conflating them. **Dedupe on the domain identity.**

The trace already carries one informally: `seq` is the first field of the body
(`FINDING`/`the message carries its trace`), and it was deliberately placed first so body identity
survived on the prefix. **Make it explicit rather than inventing a second scheme.**

## Out of scope = REJECTED

- **Deduping in the store or the queue.** At-least-once is the queue's *correct* contract; the
  consumer is where idempotence belongs, and that is what "a good SQS consumer always does this"
  means.
- **Exactly-once delivery.** Not on offer, not being attempted. Effectively-once *processing* via an
  idempotent consumer is the real thing.
- **Re-tightening the visibility window.** It becomes safe to, and it is a separate change with its
  own measurement.
- **`wat/`, `src/`.** Neither changes.
