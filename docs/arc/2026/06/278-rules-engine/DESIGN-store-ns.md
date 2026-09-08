# DESIGN — `store-ns`

**`store-ns` becomes a named field on `TakeAcc` and on the queue's `:ephemeral` state, reported as
`store-ms=`.** `sqs.wat` + `circuit.wat` + the six ripple files.

The instrument that splits the drain into **time in the store** and **everything else**. Third
attempt; the first two failed to parse, and the shape that blocked them is gone.

## WHY — the fork is still open and unchanged

`the queue counts its store round trips` measured:

| n | store-calls/pair | drain ms/pair |
|---|---|---|
| 500 | 1.239 | 0.2515 |
| 1000 | 1.247 | 0.3125 |
| 2000 | 1.266 | 0.3999 |

**Flat count, +59 % wall time per message.** Two candidates were then eliminated *by reading*:

- **store contention is structurally impossible** — each subscriber store is granted to exactly one
  pid (`circuit.wat:2101`) and the queue is a serializing `defservice`, so at most one store call is
  ever in flight.
- **the count explanation is dead** — measured above.

⚠ What remains is **two** branches, not one. A store round trip is *bracketed by the queue's own
work* — state reconstruction, the outbox, the waiters vector. The 59 % can live in the **queue** as
easily as in the **store**. The previous DESIGN named a winner inside its own fork and was wrong to.

## ⛔ WHY THIS PARSES NOW

Two strikes died on the same edit. grok: *"UnclosedParen with Continue=4; UnexpectedRParen with
Continue=5. **No integer works.**"* The cause was `Tuple`'s deliberate three-element limit —
threading a fourth value forced `(Tuple sc ns)` into an already-nested accumulator.

`the accumulator is declared where the child can see it` removed that (`cd3c5b220`):

```wat
;; sqs.wat:95, inside :messages so the process child sees it
(:wat::core::defstruct :queue::TakeAcc
  [store <- Peer   keep <- PersistentVector<Waiter>   calls <- i64])
```

★★★ **`store-ns` is now a named field.** No nesting, no paren depth change at any use site. The
edit that could not be made twice is a one-line addition.

## ⛔ THE OVERHEAD IS PRICED

`probe-what-a-clock-read-costs.wat`, three runs: **322 / 321 / 359 ns**, so 20 000 reads is
**6.4–7.2 ms against a 3 200 ms drain — 0.20 %**, against an effect of **59 %**. A 0.2 % instrument
cannot masquerade as that.

⚠ Not `await-timer-ms`, measured at **1269 µs** earlier in this arc — that is a timer arm + fire +
select wake. A bare clock read is ~4 000× cheaper.

## ⛔ THE ONE CONTRACT DECISION

```
TakeAcc          [store  keep  calls  ns]           ← a named field, sqs.wat:95
:ephemeral       … store-calls  store-ns …
StatsResponse::Ok [receive-calls ticks visible unacked store-calls store-ns]
phases            store-ms=   (nanos on the wire, millis at the format)
```

Two clock reads per store round trip, at the **same eight sites** `store-calls` already counts
(`sqs.wat:177 213 269 303 413 773 1075 1124`). Every site that counts also times — a site that
counts without timing is a compile-visible asymmetry.

## THE FORK IT RESOLVES — stated so it cannot be retrofitted

- **`store-ms`/pair grows in step with `drain`/pair** → the time is inside the store; the next
  question is the SQL and the blast radius becomes the **stdlib**.
- **`store-ms`/pair flat while `drain`/pair grows** → **the store is exonerated**; the time is in
  the queue's own processing or in scheduling between actors, and the search moves somewhere nobody
  has looked.

★ **No prediction.** Both are real outcomes.

## WHAT THIS IS AND IS NOT

⚠ **It does not explain the slope.** It halves the search space — the third halving, after the two
eliminations above.

⚠ **`store-ns` is cumulative over the whole run**, no per-op breakdown, no fill/drain split — the
same limits as `store-calls`. `store-ms` **exceeding `drain` is expected and correct**, because the
fill's puts are in it.

⚠ **Eight files, fifteen sites.** `Queue::StatsResponse::Ok` ripples exactly as before — grepped
**before** this DESIGN was written.

## OUT OF SCOPE — REJECTED

- **Timing inside the store service** (`wat/query/sqlite-store.wat`, stdlib). Only worth the rebuild
  if the fork points there — which is why we ask first.
- **A per-op breakdown.** One number resolves the fork.
- **`circuit.wat`'s 36 nested-Tuple chains**, the `Lost`/`Closed` → `Exhausted 0` conflation, and
  `collect`. All open, none on the slope's path.
