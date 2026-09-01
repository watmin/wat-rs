# BRIEF — long polling in `wat-queue`

Let `receive` wait for a message instead of returning empty, so a worker stops spending a network
round-trip to be told "nothing yet". Then let the circuit ask for more than one message at a time.

Read `DESIGN-STONE-the-queue-long-poll.md` beside this first — the contract decision is that there
must be **one receive path, not two**, and that is the thing that fails silently.

## Read in order, and why you are being sent there

1. **`wat-scripts/queue/sqs.wat`** — `ReceiveRequest {queue, now-ns, visibility-ns, limit}` and the
   `receive` arm. **The arm's body is what you are extracting into a shared function**, not
   rewriting: `scan-index` for visible messages, the visibility re-put, the envelope shape.
2. **`wat/service.wat:69-81`** — `Directed {conn-id, reply}` and `Outcome::ReplyTo`. Landed this
   evening; `wat-tests/service-deferred-reply.wat` is the worked exemplar, including a **timer**
   waking a client at both loci.
3. **`wat/telemetry/span.wat`, the `log` arm** — arming on the **empty→non-empty transition** with no
   stored flag. Copy that pattern for the expiry tick; the reasoning is identical.
4. **`tests/services/probe_arc278_self_scheduling.wat`** — `(-tick [s ctx])`. Internal arms take
   **no request**, which is why there is one scanning expiry tick rather than a timer per waiter.
5. **`wat-scripts/fanout/circuit.wat`**, the worker's `drain` — `:limit 1` and a fold that runs `cap`
   times regardless. This is the consumer that demonstrates the win.

## The work

**1. `ReceiveRequest` gains `wait-ns <- i64`.** `wait-ns = 0` is **byte-identical to today**.

**2. Extract the receive body into a shared function** called by the `receive` arm, the `send` arm,
and the expiry tick. One path (see the contract decision).

**3. Park on empty + wait.** Store `{conn-id (from ctx), queue, limit, visibility-ns, deadline}` in
`:ephemeral`; `NoReplyAndArm` the expiry tick **on the empty→non-empty transition** of the waiter set.

**4. `send` wakes waiters** for that queue, FIFO, running the shared receive path and `ReplyTo`-ing
each satisfied waiter.

**5. The expiry tick** uses `ctx`'s `start-ns` as now, `ReplyTo`s empty to any past deadline, drops
them, and re-arms only if waiters remain.

**6. The circuit asks for more.** `wait-ns > 0` and `limit > 1` in the worker's `drain`. Re-run it and
**report the wall time — do not promise one.**

## Blast radius

`wat-scripts/queue/sqs.wat` and `wat-scripts/fanout/circuit.wat`. **No `service.wat`, no `Outcome`,
no runtime, no stdlib.** The substrate is finished.

## STOP triggers

**STOP-1 — one receive path.** If the `send` arm or the tick builds its own reply instead of calling
the shared function, STOP. They will drift, and they will drift silently: the immediate path is
covered by every existing queue gate and the wake path by none of them.

**STOP-2 — `wait-ns = 0` must not move.** Every existing queue gate must pass **unedited**. If one
needs editing, behaviour changed and long polling stopped being additive.

**STOP-3 — waiters do not go in `:durable`.** A `conn-id` does not survive a fork or a resume;
persisting one is persisting a promise to a client that cannot exist.

**STOP-4 — an idle queue must never wake.** Arm on the empty→non-empty transition, re-arm only while
waiters remain. A queue with no waiters that still ticks is a wake per queue per interval, and it
scales with queue count.

## The gates to write

- **★ a parked receive is woken by a send** — the waiter gets the message, with the visibility re-put
  applied, exactly as an immediate receive would. **RED today.**
- **★ a parked receive times out** — `wait-ns` elapses, the waiter gets empty, and the queue keeps
  serving.
- **no empty round-trip** — with long polling, a drain that would have spun N times makes far fewer
  receive calls. Count them.
- **`wait-ns = 0` is unchanged** — every existing queue gate passes unedited.
- **an idle queue is silent** — no waiters, no ticks.
- **the circuit** — same output string (`total=8000; distinct=8000; dup=0`), wall time **reported**
  against 88.6 s.

## Prior comparable result

`SCORE-deferred-reply.md` — the substrate this consumes, and a no-delta stone whose gate had to prove
a **timer** waking a client, not just a client waking a client.
