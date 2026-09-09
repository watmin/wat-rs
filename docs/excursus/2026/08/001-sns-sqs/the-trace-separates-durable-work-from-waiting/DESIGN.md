# DESIGN — the trace separates durable work from waiting

**Two new stamps, one phantom stage deleted.** `wat-scripts/topic/sns-fanout.wat` +
`wat-scripts/fanout/circuit.wat`. Telemetry only — no system behaviour changes.

## WHY — the current trace cannot answer the question being asked

The builder's hypothesis: *the durable parts are more expensive than interpreted code.* The circuit
already emits five histograms and **not one of them can settle it.**

```
outbox   50-250 = 7989/8000   max 236ms
t1->t2   <1ms   = 8000/8000   max   0ms
t2->t3   <1ms   = 7997/8000   max   1ms
t3->t4   10-50=5274 50-250=2232  max 116ms
e2e      50-250=7405 250-1000=588
```

### ⛔ `t1->t2` IS A PHANTOM

`sns-fanout.wat:481`:

```wat
stamped (:wat::core::format "{b}|{t1}|{t2}|{t3}" :b rest :t1 t1 :t2 t1 :t3 t3)
                                                          ^^^^^^^^^^^^^^
```

**`t2` is assigned `t1`.** `max=0ms` for all 8000 is not a fast stage — it is a subtraction of a
number from itself. A stage that reports zero by construction is worse than no stage: it reads as
evidence.

### AND BOTH EXPENSIVE HOPS FUSE TWO DIFFERENT THINGS

```
t0  publisher, before Topic/publish
      │  outbox 50-250ms = publish RPC + the topic's Store/put + INBOX QUEUEING + worker receive
t1  topic-worker receives from inbox
      │  t2->t3 <1ms = topic-worker processing only            ← genuinely free
t3  bucket built
      │  t3->t4 10-250ms = Queue/send + its Store/put + SUBSCRIBER QUEUEING + receive
t4  subscriber worker receives
```

★★ The one stage that isolates pure processing is free. The two expensive stages each contain
**one durable write and an unbounded wait**, welded together. So the trace says "50–250 ms" and
cannot say which part.

## ⛔ THE ONE CONTRACT DECISION — stamp the moment the write RETURNS

Two stamps, one per expensive hop, taken at the boundary between work and waiting:

- **`t0b`** — in `Topic::publish`, immediately after its `Queue/send-all` returns `Accepted`
- **`t3b`** — in the topic-worker, immediately after its subscriber `Queue/send` returns

Then:

```
t0  -> t0b   durable work on the publish path      (RPC + Store/put)
t0b -> t1    pure inbox QUEUEING
t1  -> t3    topic-worker processing               (already measured, free)
t3  -> t3b   durable work on the fanout path       (RPC + Store/put)
t3b -> t4    pure subscriber-queue QUEUEING
```

★ Five real stages replacing four real ones and a phantom. **`t2` is deleted, not repaired** —
there is nothing at that point to measure.

## WHAT IT WILL PROBABLY SHOW, WRITTEN DOWN SO IT CAN BE WRONG

`Store/put` measures **675 µs** in isolation, inside a hop of **50–250 ms**. If the stamps agree,
durability is **well under 1 %** of that stage and the rest is queueing — which would *contradict*
the hypothesis that prompted this stone.

⚠ That is an inference from two separate measurements, not a trace. I have been wrong six times
today inferring instead of measuring, so it is recorded here as a prediction to be tested, not a
conclusion to be confirmed.

★★ And if it is right, the consequence is the useful part: **the system is not durability-bound or
interpretation-bound — it is queue-bound**, and effort belongs on how long a message waits, not on
how fast a write is.

## OUT OF SCOPE — REJECTED

- **`wat/telemetry.wat`'s `Span`/`Journal`.** A real telemetry system exists and the circuit
  hand-rolls stamps instead. Adopting it is a bigger, separate question; this stone fixes the
  trace we already have rather than replacing the mechanism mid-investigation.
- **Acting on the result.** This stone measures. What to do about a queue-bound system is the next
  stone, and it should be chosen after the number exists.
- **`collect`'s 5.9 s.** Still the harness measuring itself, still unbounded responses, still its
  own stone.
