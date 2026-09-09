# DESIGN — the benchmark has more than one publisher

**`run-with` gains a publisher count.** `wat-scripts/fanout/circuit.wat`. Until this lands, every
perf number this arc has produced is a statement about a topology we do not ship.

## WHY — the producer side never got the treatment the consumer side has

```
run-with [n m j rate seed drop-* …]
          │ │ │
          │ │ └─ workers per queue
          │ └─── queues
          └───── messages
grep publishers|npub|n-pub  →  0
```

**One publisher, hardcoded, with no way to configure otherwise.** `m` and `j` scale consumers; the
producer side is a single sequential fold.

★★★ Every conclusion this session rests on that. In particular this one, which I asserted and
which the benchmark cannot test: *"full jitter buys nothing because there is no herd."* True at
N=1 **by construction** — I measured a configuration with no herd and concluded herd-mitigation is
worthless.

At N publishers a fixed 25 ms is the **worst** policy available: every publisher bounces off
`Accepted 0` and wakes in lockstep. That is exactly the thundering herd full jitter exists to
break.

⚠ So `docs/excursus/2026/08/001-sns-sqs/the-client-backs-off-intelligently/SCORE.md`'s headline is wrong in its framing, and this
DESIGN corrects it: the strike was sound. **Adaptive backoff beat the shipped value by 2.0 s; its
de-correlation property was untested, because the harness cannot express more than one client.**
The 18472 ms "optimum" it lost to is an artifact of N=1, and tuning toward it optimises for a
cardinality we do not ship.

## ⛔ THE ONE CONTRACT DECISION — each publisher draws from its OWN seed

`:wat::rand::int-from` is threaded, so a shared seed would make P publishers produce **identical**
delay sequences — a synthetic herd that is an artifact of the harness rather than of the policy.
Each publisher carries its own seed, derived from the run seed and its index.

★ Without this the experiment cannot answer the question it exists to ask.

## ⛔ THE INTEGRITY GATE — p=1 must reproduce today

Adding the parameter must not move the p=1 numbers. If it does, the harness changed the
measurement and **no cross-stone comparison in this arc is valid any more** — including every
baseline the last six stones were graded against. That row matters more than the new numbers.

## THE SHAPE — mirror the worker

`Worker` already fires with `start` and reports with `disrupts`. A `Publisher` service does the
same: `start` publishes its share, `stats` returns `(calls, retries)`. The parent spawns P, then
joins; `publish` is spawn → last publisher done.

Message ids must stay globally distinct across publishers, so `distinct=8000` remains the
invariant that catches a partition bug.

## THE EXPERIMENT THIS UNLOCKS

Three policies × two cardinalities, same box, same run:

```
                       p=1                p=3
adaptive (shipped)   20379 (known)         ?
fixed 25 ms          18472 (known)         ?     ← predicted to degrade: lockstep wakes
fixed  1 ms          22395 (known)         ?
```

★★ The prediction on the record: **fixed 25 ms degrades at p=3 and adaptive does not.** If it does
not degrade, then jitter is unjustified in this system at any cardinality we care about, and that
is worth more than the stone.

## OUT OF SCOPE — REJECTED

- **`Wait :UpTo` on `send`.** Parking is next, and it has its own N-dependence this stone will
  expose: P parked senders all woken when room appears is a wake storm unless the queue wakes only
  as many as it has room for. **Do not draw parking until this lands.**
- **Choosing a backoff variant.** No policy change here. This stone only makes the question
  askable.
- **`setup` / `stop`.** Still after publish.
