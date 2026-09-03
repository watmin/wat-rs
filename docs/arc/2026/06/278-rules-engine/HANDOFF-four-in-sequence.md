# HANDOFF — four stones, in sequence, measured between each

Drawn 2026-09-03. Independent work, deliberately bundled. **The sequence is the point.**

## Why in this order, and why measured between

All four are independent — a red in one does not block the others. But if they land together, no
result can be attributed to anything. This arc has already lost time to exactly that (a 2×2 with one
variable uncontrolled produced numbers worse than none).

**Five runs and a recorded number after each stone, before starting the next.** If you run out of
road, stopping after stone 1 leaves the main line advanced; stopping after stone 4 with one combined
measurement leaves us with nothing we can attribute.

Current baseline, mem, `2000×4×3`: **149–161 deliveries/s**, e2e max ~700 ms, `distinct=8000`.

---

### Stone 1 — the consumer is idempotent  ·  `BRIEF-the-consumer-is-idempotent.md`

**Main line. Precondition for packet-loss injection.** Fully drawn; read that brief.

In one line: our duplicate detector keys on `queue/envelope-id`, `send` mints a fresh uuid per call,
so a redelivery raises `total` and `distinct` together and `dup` stays 0. **The invariant cannot
witness the duplicate at-least-once produces.**

**Do this one first** — its throughput baseline (149–161/s) is the one the other three will move.

---

### Stone 2 — the topic-worker batches  ·  NEW, drawn here

`sns-fanout.wat`'s topic-worker takes up to 10 inbox rows per tick, then does, **per row**: one
`Queue/send` carrying a **single-element `bodies` vector**, and one `Queue/ack`.

The batching surface exists — `bodies` is a `Vector`, `Store::PutRequest` takes many — and it is
being used with one element. The wire-batching win did not survive the adapter's deletion; the
capability did.

**Group the tick's rows by subscriber**, one `Queue/send` per distinct subscriber carrying its
messages, and ack the batch. Ten rows spanning four subscribers becomes ~4 sends + acks instead of
10 + 10.

- **STOP** if grouping changes delivery order *within* a subscriber. Order within a queue is the
  queue's business, and a batch must preserve it.
- **STOP** if `distinct=8000` breaks. Grouping is where a batched pipeline drops its tail.
- Expect the per-message CPU not to amortise (`uuid::v4`, `edn::write`, row build), as measured
  before: the round trips collapse, the work does not.

---

### Stone 3 — the circuit runs on sqlite  ·  side quest S2

`mem-store` writes are O(table); `sqlite-store` is linear. Measured **1.68×** once durability made
the store hot — up from 1.29× before it.

The codemod exists, is idempotent, and its diff is verified store-only:

```
cp wat-scripts/fanout/circuit.wat wat-scripts/scratch-pad/probe-circuit-sqlite.wat
printf '["wat-scripts/scratch-pad/probe-circuit-sqlite.wat"]\n' \
  | ./target/release/wat ./wat-scripts/scratch-pad/fix-circuit-to-sqlite.wat
```

**The decision is whether the circuit itself should move**, not whether the codemod works.
`mem-store` remains the differential **oracle** either way; this is about which backend the fixture
demonstrates. If you move it, the sqlite probe stops being a variant and becomes redundant — say so.

---

### Stone 4 — the trace stamp stops round-tripping  ·  side quest S10, corrected

`sqs.wat:253` appends a timestamp whenever the body contains `|`. The topic-worker then **splits the
body on `|` and re-joins it minus that segment**, per message, to keep subscriber traces well-formed.

Production code adds a field; other production code removes it. That is worse than the original wart
this replaced, and it is string surgery on every message.

**Last, deliberately** — stones 1 and 2 may change the body's shape, and fixing this against a
format that is about to move would be wasted.

- **STOP** if removing it costs the trace. The histogram is our primary instrument now; losing a
  stage is worse than the wart.

---

## Floor and method, all four

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — capture,
name the arm, do not re-run.

**Check `ps` before any timing.** **Five runs, report the spread**, and report **deliveries/s**, not
wall — `setup`+`stop` is process lifecycle and is the builder's separate concern.

One SCORE per stone. If a stone stops, say so and start the next: they are independent by
construction.
