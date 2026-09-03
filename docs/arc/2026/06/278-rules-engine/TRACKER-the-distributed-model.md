# TRACKER — the distributed model, and the side quests

Opened 2026-09-02. **Living document — update in place, do not append a second copy.**
No arc number is minted here; this tracks work inside arc 278 until the builder rules otherwise.

## Baseline as of 2026-09-02 (measure against this, five runs, quiet box)

```
2000×4×3, mem-store, topic cap 16, queue cap 32, K=10
  publish+drain 8.3–8.7 s   →  921–954 deliveries/s   (±1.8%)
  e2e max 152–197 ms        →  t3→t4 >1 s count = 0
  total=8000; distinct=8000; dup=0
  wall 22.6 s, of which ~12.5 s is process spawn/reap (builder's boot-time item, NOT this arc)
```

Same fixture this morning: **109 deliveries/s, e2e ~12 s, non-deterministic.**

---

## THE MAIN LINE

### 1. Prove redelivery works — ✅ DONE 2026-09-02
`probe-visibility-redelivers.wat`, gated by `tests/services/probe_queue_visibility.rs`:
`first=got;while-inflight=none;after-expiry=got;same=yes`.

### 1-historical. Why it needed proving — NOT STARTED, probe, small
`sqs.wat:62` states the intent (*"the message stays invisible until its visibility timeout"*) but
**nothing exercises redelivery after visibility expiry**, and the circuit sets `visibility-ns` to
10¹² ns precisely so it never happens.

Everything below assumes retry-on-no-ack works. **Establish it before building on it.**

### 2. Topic durability via its own queue-service — ✅ STRUCK 2026-09-03
Costs **6.1× throughput** (921–954/s → 149–161/s, e2e 152–197 ms → ~700 ms), and ~3.5× of that
survives a linear store. See `FINDING-durability-is-store-op-bound.md`.

### 2-historical. The gap it closed — NOT DRAWN, stone
`publish` returns `Ok` for a message held in **`:ephemeral` state** (`sns-fanout.wat`, `outbox`).
The queue puts to a store before replying `Ok`; the topic does not. The two services disagree about
what acceptance means and only one is honest.

The shape, composed entirely from existing parts:

```
topic-service = publish surface + ONE queue-service instance + J internal workers
```

- `publish` writes **N rows, one per subscription**, to the internal queue, then replies `Ok`
- internal workers consume it, call `Queue/send` on the subscriber queue, **ack on success**
- subscriber `Full` → **don't ack** → visibility expires → retried

Per-subscription tracking is the point: SNS is **at-least-once per subscription**, not best-effort —
retry policies and DLQs attach per subscription. The `(message, subscriber)` row is the unit; a
message-level unit would re-deliver to everyone on one subscriber's retry.

★ **Do not write a second delivery engine.** A durable topic with background retry *is* a queue with
subscribers as consumers, and the queue already has level-triggered wakeup, a depth bound, parked
waiters and batching.

### 3. Packet loss injection — NOT DRAWN, probably its own arc (builder's ruling)
The one fault domain never simulated. IPC has been giving us reliable networking for free.

★ **The acceptance criterion is that it BREAKS something:** `dup=0` is a property of the transport,
not of the design. At-least-once under loss produces duplicates by definition — a lost ack means
redelivery. So the honest invariant becomes `distinct=8000; dup ≥ 0`, and the choice is idempotent
consumers or store-side dedup by message id.

**If loss is injected and `dup` stays 0, the loss is not being modelled.**

---

## SIDE QUESTS — found in passing, none chased

| # | what | status |
|---|---|---|
| S1 | ~~Adapter's 1 ms retry poll~~ **OBSOLETE — `:fanout::adapter` was deleted by the durable-topic stone.** No adapter, no poll. | dissolved |
| S14 | **The topic-worker sends and acks ONE ROW AT A TIME** — `bodies` is a `Vector` used with a single element, and one `Queue/ack` per row. The batching surface survived the adapter's deletion; its use did not. | ready |
| S1-old | ~~(superseded)~~ Its trigger has fired (bounding every stage worked). Same repair already made twice; short. | ready |
| S2 | **Store swap to sqlite in the circuit.** Measured 1.29× at cap 16, 1.75× batched. The codemod exists (`fix-circuit-to-sqlite.wat`), is idempotent, diff verified store-only. | ready |
| S2b | **The store is now 1.68×**, up from 1.29× before durability — the durable topic made it hotter exactly as predicted. Promotes S2 from convenience to load-bearing. | ready |
| S13 | **The circuit asserts exactly-once on an at-least-once system.** `probe_ex001_fanout` requires `total == distinct`; a visibility expiry during processing produces a legitimate duplicate and reds the floor (it did, 26 vs 24). Widening the window 200 ms → 5 s is correct SQS configuration and **does not remove the class** — the assertion still depends on a timing margin. Resolution is an idempotent consumer (dedupe by envelope id), and **item 3 forces it anyway**. | open |
| S3 | **`mem-store` writes are O(table).** 1000/2000/4000 rows → 6.5/20.8/90.0 s. At the current workload it is a **1.29× perf term**; it is also the differential **oracle**, so an O(n²) oracle slows every differential as the corpus grows. `SCORE-perf-3` claimed "writes go linear" and is corrected in place. | open |
| S4 | **`:wat::core::Vector`'s `conj` is O(n)**, `PersistentVector`'s is O(1) — 17× at n=4000, **no stated complexity contract anywhere**, and the name points the wrong way. **23 files** accumulate into a `Vector` via `conj` in a `foldl`. Already fixed once at one call site (`stream->vec`) and **regrew** in the topic's outbox. Rides the builder's corpus migration; wants a contract and possibly a lint, not just a migration. | builder's |
| S5 | **Substrate `ensure-alarm` outcome.** The level-triggered wakeup uses a hand-maintained `armed?` flag; an outcome meaning *"ensure an alarm exists for op X"* makes both armed-twice and armed-zero-times unrepresentable. Rung 3 for a class currently at rung 2. Needs `wat/`. | open |
| S6 | **Nonexistent stdlib enum variants are not resolve-checked.** `:wat::core::Option::Nope`, `:wat::kernel::RecvOutcome::Bogus` all pass `--check` and die at run time; a same-file enum is rejected. Cost real time twice today. | open |
| S7 | **Duration-0 `after` never fires at process tier.** Verified; thread fires, process is silent, no diagnostic. Locus transparency break. Untouched. | open |
| S8 | ~~The outbox cursor~~ **OBSOLETE — the topic's outbox was deleted by the durable-topic stone.** | dissolved |
| S8-old | ~~(superseded)~~ The rebuild was ~0.47 ms/delivery; K=10 tick-batching amortises it. **Its share needs re-measuring** before it is ruled on again — the last two rulings both expired for reasons I mis-dated. | re-measure |
| S9 | **`body-key` is dead code** in `circuit.wat` and the sqlite variant — defined, never called. So "the same body delivered twice" is checked by nothing. | trivial |
| S10 | **CORRECTED and WORSE.** Alive at `sqs.wat:253` (`contains? b`, which is why a `contains? body` grep missed it — token, not form). The queue appends a stamp; the topic-worker now **splits and re-joins the body per message to strip it back off**. Production code adds a field that other production code removes. | ready |
| S10-old | ~~(superseded)~~ — production code branching on payload content for instrumentation. Least-bad given the surface was frozen, but a wart. | wart |
| S11 | **Promotion ruling:** `wat-scripts/{topic,queue}` → `wat/`. | builder's |
| S12 | **The capstone** — telemetry instrumenting the circuit. Still unbuilt; the in-band trace has partly superseded its purpose. Worth re-scoping rather than building as originally drawn. | re-scope |

---

## METHOD RULES EARNED TODAY — these cost stones

1. **A perf row needs a distribution, not a sample.** Five runs minimum. Re-running a *timing*
   measurement is not the flake re-run the floor doctrine forbids — that rule protects **red
   assertions**, where a green re-run destroys evidence. For a timing row the spread *is* the evidence.
2. **A perf finding measured under one regime does not transfer when the regime changes.** The store
   was correctly ~1% with shallow queues and 5.6× with deep ones. I closed that lane and had to
   reopen it two stones later.
3. **Check `ps` before any timing measurement.** A contended box produced a false reading.
4. **`assert s.count(old)==1` on scripted edits catches false MEASUREMENTS, not just bad rewrites.**
   It stopped "50 ms fixes the variance" from shipping off three lucky samples.
5. **Three samples of a bimodal distribution look unimodal.**
6. **A 2×2 with one variable uncontrolled is not a 2×2**, and its numbers are worse than none.
