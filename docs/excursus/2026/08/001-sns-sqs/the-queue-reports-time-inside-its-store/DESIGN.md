# DESIGN — the queue reports time inside its store

**`Queue::StatsResponse` gains a second counter: nanoseconds spent inside `Store/*` calls.**
`sqs.wat` + `circuit.wat` + the same six ripple files.

The instrument that splits the drain into **time in the store** and **everything else**.

## WHY — count is dead, contention at the store is impossible, and one suspect was nearly skipped

`the queue counts its store round trips` measured:

| n | store-calls/pair | drain ms/pair |
|---|---|---|
| 500 | 1.239 | 0.2515 |
| 1000 | 1.247 | 0.3125 |
| 2000 | 1.266 | 0.3999 |

**Flat count, +59 % wall time per message.** So the same round trips take longer.

★★★ **Contention at the store is structurally impossible** — established by reading, not measuring.
Each subscriber store is granted to exactly **one** pid (`circuit.wat:2101`, inside the owning
queue's `post-spawn-fn`) and its address reaches only that queue's Record (`:2102`). The queue is a
serializing `defservice`, so **at most one store call can be in flight**. No queueing at the store.

⚠ **And the suspect I nearly skipped.** A store round trip is bracketed by the *queue's own* work —
state reconstruction, the outbox, the waiters vector. The 59 % can live in the **queue** as easily as
in the **store**, and the previous stone's fork never named that branch. Declaring "so the store's
SQL is getting dearer" would be the third time this arc named a winner it had not earned.

## ⛔ THE OVERHEAD OBJECTION IS PRICED AND DISSOLVES

`the queue counts its store round trips` deferred timing under STOP-5: *"two `time::now` reads per
store op is ~20 000 extra clock reads inside the phase under measurement."* **The objection was never
priced.** It is now — `wat-scripts/scratch-pad/probe-what-a-clock-read-costs.wat`, three runs:

```
per-read = 322ns / 321ns / 359ns        20,000 reads = 6.4ms / 6.4ms / 7.2ms
```

★★★ **6.4 ms against a 3 200 ms drain is 0.20 %.** The effect under measurement is **59 %**. A 0.2 %
instrument cannot masquerade as that, and the objection that deferred this stone is answered rather
than assumed away.

⚠ Not to be confused with `await-timer-ms 1`, measured at **1269 µs** earlier in this arc. That is a
timer arm + fire + select wake. A bare clock read is ~4 000× cheaper.

## ⛔ THE ONE CONTRACT DECISION — accumulate nanos, report milliseconds

```wat
:Ok [receive-calls  ticks  visible  unacked  store-calls  store-ns]
```

`store-ns <- :wat::core::i64` on the queue's `:ephemeral` state beside `store-calls`, accumulated as
`(now_after - now_before)` around **each of the eight `Store/*` call sites**, summed across the m
subscriber queues by the circuit and reported as `store-ms=` on the phases line.

★ Two clock reads per store round trip, at the **same** eight sites `store-calls` already counts —
the increment and the timing are one edit per site, and any site that counts but does not time is a
compile-visible asymmetry.

## THE FORK IT RESOLVES — stated so it cannot be retrofitted

With `store-ms` beside `drain` at each depth:

- **store-ms/pair grows in step with drain/pair** → the time is inside the store. The next question
  moves into `wat/query/sqlite-store.wat` and the SQL, and the blast radius becomes the **stdlib**.
- **store-ms/pair flat while drain/pair grows** → **the store is exonerated.** The time is in the
  queue's own processing, or in scheduling between actors — and the search moves somewhere nobody
  has looked.

★ **No prediction is offered.** Both are real outcomes. The previous stone's DESIGN named a winner
inside one branch of its own fork and was wrong to; this one states both and picks neither.

## WHAT THIS IS AND IS NOT WORTH

⚠ **It does not explain the slope.** It halves the search space, twice over — once by measurement
here, once by the structural elimination of store contention above.

⚠ **`store-ns` is cumulative over the whole run**, with no per-op breakdown and no fill/drain split —
the same limits `store-calls` has. It answers the fork and nothing else. The previous grading records
what happens when that kind of counter is over-read.

⚠ **Eight files again.** `Queue::StatsResponse::Ok` is matched at 14 sites; appending a field ripples
to the same six extra files. Verified by
`grep -rn 'queue::Queue::StatsResponse::Ok' --include=*.wat .` — **this time the grep was run before
the brief, not after.**

## OUT OF SCOPE — REJECTED

- **Timing inside the store service.** `wat/query/sqlite-store.wat` is stdlib, frozen into the binary.
  Only worth the rebuild if the fork points there — which is the point of asking first.
- **A per-op breakdown** (scan vs put vs delete). One number resolves the fork; a breakdown answers a
  question we have not earned.
- **`collect`** (15.7 s measured today vs 5.9 s in the tracker), the **`Lost`/`Closed` → `Exhausted 0`**
  conflation, and the **2.4× at m=8**. All open, none on the slope's path.
