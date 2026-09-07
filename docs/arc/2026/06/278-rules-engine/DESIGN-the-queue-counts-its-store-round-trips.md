# DESIGN — the queue counts its store round trips

**`Queue::StatsResponse` gains one counter: how many round trips this queue has made to its store.**
`wat-scripts/queue/sqs.wat` + `wat-scripts/fanout/circuit.wat`.

An **instrument**, not a fix. It does not explain the slope; it makes the slope decomposable.

## WHY — the slope is real, and every cheap explanation is already dead

The drain curve, measured at the same `m=4 j=3 sub-cap=8192`, two runs agreeing within 2 %:

| n | pairs/receive-call | worker proc | drain ms | **ms per receive call** |
|---|---|---|---|---|
| 500 | 9.43 | `<1ms` ×1973, max 1 ms | 510 | **2.41** |
| 1000 | 9.64 | `<1ms` ×3979, max 1 ms | 1266 | **3.05** |
| 2000 | 9.60 | `<1ms` ×7955, max 1 ms | 3230 | **3.88** |

★★★ **Batching is flat** (9.4–9.6 of a possible 10), **worker processing is flat** (`<1 ms`, max
1 ms at every depth), and **receive calls scale linearly** (0.106 / 0.104 / 0.104 per pair). The
entire slope sits in **cost per receive call — up 61 % from n=500 to n=2000.**

And two candidates died on the disk before this was drawn:

- **"the scan walks more rows at depth."** It does not. The receive scans
  `isk ∈ [at-nanos 0, at-nanos now]` (`sqs.wat:166-170`) while a claim sets
  `hide-at = now + vis-ns` (`:178`) — which moves the row **outside** the scanned range. Claimed
  rows are never walked, and the SQL carries `ORDER BY isk ASC LIMIT ?5`
  (`sqlite-store.wat:438`).
- **"the poller causes it."** `poll-calls` per pair rises (0.048 → 0.060 → 0.079), but that is a
  *consequence* of the drain lengthening — the poller does `m+1` per iteration regardless. It is
  correlated, and correlation is what this arc has already lost seven mechanisms to.

## ⛔ WHY THIS IS AN INSTRUMENT AND NOT A FIX

The obvious next sentence is *"so store ops must be getting dearer."* Writing it would mean
**dividing wall-clock by an assumed op count** — 833 receive calls × (1 scan + 10 claim-puts) plus
8000 ack-deletes — and that arithmetic is the **exact shape of the "5.7 ms per call" mechanism this
arc already buried**, which attributed blocked time to calls that were merely waiting.

★ The assumption in question — *ten puts per receive, ten deletes per ack* — has never been
measured. It is inferred from `:limit 10`. **Counting the round trips tests it directly.**

## ⛔ THE ONE CONTRACT DECISION — a count, not a timing

```wat
:Ok [receive-calls  ticks  visible  unacked  store-calls]
```

One `i64` on `Queue::StatsResponse`, incremented at each of the **eight** `Store/*` call sites in
`sqs.wat` (`:168 :204 :256 :290 :384 :729 :990 :1039`). The circuit sums it across the m subscriber
queues and reports `store-calls=` on the phases line.

★★ **Deliberately a count and not a wall-clock timing.** Timing every store call means two
`time::now` reads per op — ~34 000 extra clock reads at n=2000, inside the phase being measured.
**A zero-overhead instrument answers the first question**, and the first question is the fork:

- **store-calls per pair FLAT** → the op count is linear and **cost per op is rising**. Then the
  next stone times them, and it will know which one to time.
- **store-calls per pair RISING** → we are issuing more work per message than we thought, and the
  slope is a count problem, not a cost problem. Cheaper to fix and nobody suspected it.

`receive-calls` and `ticks` are already pure counters on this response (`sqs.wat:73-75`); this
follows that precedent exactly and adds no policy.

## WHAT THIS IS AND IS NOT WORTH — stated before measuring

⚠ **It does not explain the slope, and must not be judged on doing so.** It is judged on the
counter existing, being correct, and costing nothing measurable.

⚠ **No prediction is offered about which branch of the fork fires.** A stone that only "passes" when
the number confirms a hunch is a stone measuring its own hope.

⚠ **`sqs.wat` is not harness-local.** This touches a file maturing toward `wat/queue.wat`. Adding a
counter to a stats response is the smallest possible change there and follows the two counters
already present — but the blast radius is honestly larger than the last several stones.

## OUT OF SCOPE — REJECTED

- **Timing the store calls.** The next stone, once the fork above is resolved. Doing both at once
  means the timing overhead lands before we know whether timing is the question.
- **A per-op breakdown** (scans / puts / deletes separately). Four counters answer a question we have
  not earned yet; one counter answers the fork. If the count is surprising, the breakdown is the
  follow-up.
- **`collect`.** Verified this session to be a *separate phase* — `drain` is `t-drain0 → t-collect0`
  and contains only `poll-until-drained`. It does **not** contaminate the drain, so it is not a
  prerequisite. Still open on its own terms (15.7 s measured today vs 5.9 s in the tracker).
- **The `Lost`/`Closed` → `Exhausted 0` conflation.** Real, small, unrelated to the slope.
