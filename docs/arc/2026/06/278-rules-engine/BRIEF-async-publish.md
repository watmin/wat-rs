# BRIEF — accept, then fan out

Make `publish` return once the message is accepted, and deliver to subscribers on the topic's own
tick. The publisher's critical path goes from **13 sequential blocking round-trips to one**.

Read `DESIGN-STONE-the-async-publish.md` beside this first — two contract decisions, and the second
one (the drain condition) is what keeps the invariant true.

## Read in order, and why you are being sent there

1. **`wat-scripts/topic/sns-fanout.wat`, the `publish` arm** — the `foldl` over subscriber peers
   calling `Sub/deliver` synchronously. **This is the defect**: the publisher waits for every
   subscriber's queue write, and each of those waits on a store write.
2. **`wat-scripts/queue/sqs.wat`** — the queue's outbox and its `-tick`/`-flush-outbox` arms. The
   topic's outbox is the same pattern; copy its shape rather than inventing one.
3. **`wat/telemetry/span.wat`, the `log` arm** — arming on the **empty→non-empty transition** with no
   stored flag, and the bounded accumulator with a distinct refusal. Both apply here.
4. **`wat-scripts/fanout/circuit.wat`, `wait-drained`** — the drain condition on `pending` and
   `in-flight`. **You are adding a third term**, and the reason is in the DESIGN.

## The work

**1. An outbox on the topic.** `publish` appends and replies **accepted**; it no longer delivers.

**2. A `-deliver` tick** that takes the head, fans out to subscribers, and re-arms while the outbox
is non-empty. Armed on the empty→non-empty transition.

**3. Bound the outbox.** A `publish` that cannot be accepted returns a **distinct refusal** — not a
silent drop. See the DESIGN for why dropping is wrong here specifically.

**4. Expose outbox depth**, and add it to the circuit's drain condition.

**5. The circuit's invariant is unchanged**: `total=8000; distinct=8000; dup=0`.

## Blast radius

`wat-scripts/topic/sns-fanout.wat`, `wat-scripts/fanout/circuit.wat`. **No substrate: no
`service.wat`, no `Outcome`, no runtime, no stdlib.**

## STOP triggers

**STOP-1 — the drain condition must cover the outbox.** An accepted-but-undelivered message rests in
a place the old condition cannot see. `pending = 0 AND in-flight = 0` now stops *before the message
arrives*. If you cannot expose outbox depth, STOP — do not compensate with a sleep or a retry count.

**STOP-2 — a refused publish is not a dropped publish.** The caller is synchronous and holding the
message; hand the refusal back. A silent drop is data loss the caller could have prevented.

**STOP-3 — the invariant is not negotiable.** `distinct=8000` is what proves nothing was lost in the
new asynchrony. If it moves, the change lost messages and that is the finding.

**STOP-4 — no substrate changes.** `ReplyTo` and self-scheduling are finished. If you find yourself
in `wat/service.wat`, the design has gone somewhere it should not.

## The gates to write

- **★ publish returns before delivery** — measure it: a publish into a topic whose subscriber is slow
  must return promptly, not at subscriber speed. **RED today.**
- **★ nothing is lost** — `total=8000; distinct=8000; dup=0`, and with the outbox term removed from
  the drain condition this must **FAIL**. (Same discipline as the sane circuit's row 2: prove the
  term is load-bearing by removing it.)
- **the outbox bound refuses rather than drops** — fill it, and the refusal reaches the caller.
- **an idle topic never ticks** — the transition arming.
- **the phase split** — re-run with publish/drain instrumentation and **report** the numbers against
  `publish=24.3 s, drain=0.02 s`. Reported, not promised.

## Prior comparable result

`SCORE-the-sane-circuit.md` — its row 2 is the model for proving a drain term load-bearing, and its
two substrate findings are why the timer values here should be non-zero.
