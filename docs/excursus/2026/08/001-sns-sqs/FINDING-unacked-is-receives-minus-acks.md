# FINDING — `unacked` is receives-minus-acks, and it leaks permanently

**Found 2026-09-05 on `sns-sqs`, arc 278, by chaos.** Measured, not reasoned.
Repro: `wat-scripts/scratch-pad/probe-stats-sees-an-expired-unacked.wat`.

## THE MEASUREMENT — one message, one queue, no workers

```
sent=[1/0];held=[0/1];EXPIRED-NO-RECEIVER=[0/1];came-back=same-id;after-receive=[0/2];AFTER-ACK=[0/1]
```

`[visible/unacked]`, read through `Queue/stats` — the *only* thing the drain reads
(`circuit.wat:779`, `fanout::depth-of`).

| cell | reads | verdict |
|---|---|---|
| after send | `[1/0]` | correct |
| after receive, 200 ms visibility, not acked | `[0/1]` | correct |
| **350 ms later — past the window, NO receiver** | **`[0/1]`** | ⛔ **reclaim is LAZY** |
| receive again | `same-id` | the receive path is fine |
| after that receive | **`[0/2]`** | ⛔⛔ **one message, `unacked=2`** |
| **after acking it — queue now EMPTY** | **`[0/1]`** | ⛔⛔⛔ **permanent leak** |

## WHAT IT MEANS

**`unacked` counts receives minus acks. It does not count messages.** A message received
twice and acked once leaves `unacked = 1` on an empty queue, forever.

Two distinct defects, and the second is the severe one:

1. **Reclaim is lazy — it happens inside `receive`, never in `stats`.** A queue nobody is
   receiving from reports `visible=0` with `unacked>0` indefinitely, however long the
   visibility window has been expired.
2. **The counter leaks on every redelivery.** Each extra receive of the same message adds
   one that no ack will ever remove, because there is only ever one ack.

## WHY THIS IS THE `drained-never`

`fanout::queue-drained?` (`circuit.wat:789`) is `visible == 0 AND unacked == 0`.

**After any redelivery, that condition is unsatisfiable.** The drain then spins its full
4000 attempts and reports:

```
drained-never: last=[0/5] outbox=0 attempts=4000 elapsed=63565      circuit.wat:1318
```

★ **`[0/5]` was never 5 stranded messages. It is 5 leaked lease counts.** Every reading
of that arm as "five messages are stuck" — including mine, earlier today — was wrong.

## WHY THE FLOOR NEVER SAW IT

At rate 0 the circuit sets `vis = 1000000000000` (1000 s, `circuit.wat:1181`), so **no
message is ever redelivered** and the counter never leaks. The defect is unreachable in the
default configuration. **A green floor is not evidence about it.**

★ And the existing probe proves the *adjacent* thing: `probe-visibility-redelivers.wat`
establishes that an expired message comes back — **on the receive path**, which it exercises
by receiving. The drain does not receive. The mechanism everything rests on was verified
through the one door the drain never uses.

## SCOPE

`wat-scripts/queue/sqs.wat` — the queue's `unacked` accounting. Not the circuit, not
`wat/service.wat`, not the chaos work. The chaos work is what made it reachable.

## NOT FIXED HERE

This note characterizes; it does not repair. The repair is its own stone, and it must decide
what `unacked` *means* before changing how it is counted — a per-message in-flight flag and a
receives-minus-acks counter are different quantities, and the drain wants the first.
