# FINDING — durability is store-op bound, and the store got hotter exactly as predicted

Measured 2026-09-03, grading `the topic is durable`.

## The cost

| | publish+drain | deliveries/s | e2e max |
|---|---|---|---|
| ephemeral topic, mem | 8.3–8.7 s | **921–954** | 152–197 ms |
| **durable topic, mem** | 49.7–53.3 s | **149–161** | ~700 ms |
| **durable topic, sqlite** | 30.2–31.3 s | **255–264** | 418–543 ms |

`distinct=8000; dup=0` on all eight runs.

**Durability costs 6.1× on mem, and ~3.5× on a linear store.** The difference between those two is
the store's share, and it is now **1.68×** — up from the **1.29×** measured before durability.

★ **That was predicted, in writing, before the stone was drawn:** *"durability puts a store write on
the publish path, and makes the store the hot path, promoting the sqlite swap from a nice 1.3× to
load-bearing."* It is the second prediction this arc that held, and both held for the same reason —
they were decomposed from measured pieces rather than reasoned outward from a neighbour.

## Where the remaining 3.5× lives

A message now traverses **two** queues — the topic's internal inbox and the subscriber's — and each
queue costs roughly **three store operations per message**:

```
send      write the row
receive   REWRITE the row to set its visibility deadline
ack       delete the row
```

So ~6 store ops per delivery, ~48,000 for a 2000×4×3 run. At sqlite's measured ~0.9 ms per put that
is ~43 s against the 31 s observed — the same order. **The system is now store-op bound**, which is
a third regime, distinct from the hop-bound and buffer-bound ones this arc has already been through.

The visibility rewrite is the interesting one: it is a **write per receive**, purely bookkeeping,
and it is inherent to the SQS model rather than an artefact of this implementation. Real SNS→SQS
also writes twice for the same reason — durable at the topic, durable at the queue.

## What is NOT a finding

**`dup=0` here is not evidence of exactly-once.** At-least-once permits duplicates; reliable IPC
simply never generates one. The one time the system *did* produce duplicates — the red floor arm
below — it was correct behaviour, not a bug.

## ⚠ The circuit asserts exactly-once on an at-least-once system

The strike's first floor went red, was captured (`.floor/2026-09-03T04-22-46Z/ARM.txt`), named, and
not re-run:

```
probe_ex001_fanout::fanout_compute_is_complete_and_lossless
assertion `left == right` failed
  left: "26"   right: "24"
```

A topic-worker's `Queue/send` + ack exceeded its 200 ms visibility window under a loaded floor, the
row became visible again, a second worker took it, and the message was delivered twice. **That is
at-least-once working correctly.**

The fix — widening the topic-worker's window to 5 s (`circuit.wat:624`) — is **correct SQS
configuration**: visibility must exceed processing time. But it does not remove the class. The test
asserts `total == distinct`, i.e. exactly-once, and the only thing preventing a duplicate is a
timing margin. Under enough load 5 s can be exceeded too, and the floor would go red for something
that is not a bug — which this repo's doctrine has no room for.

**The structural resolution is an idempotent consumer** — dedupe by envelope id — so `total ==
distinct` holds regardless of redelivery, and the window can be tight again. **Main-line item 3
forces this anyway:** once packet loss is injected, redelivery stops being a rare race and becomes
routine.

Tracked as side quest **S13**.
