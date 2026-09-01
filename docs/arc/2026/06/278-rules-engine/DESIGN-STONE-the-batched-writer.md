# DESIGN STONE — item (b): the batched writer

**Commissioned 2026-09-01.** Fixes the finding from stone C: after a failed size-triggered flush the
span's buffer can exceed the server's cap and become **permanently unflushable**, growing with every
subsequent log.

The asymmetry that produces it, both sides read this session:

```
the span triggers at  >=    span.wat:75, 123, 172
the server rejects at  >    service.wat:1779, 2132
```

An over-cap buffer is exactly what fragmentation exists for, and the arc named it: item (b),
*"writers FRAGMENT an oversized batch into ≤budget submissions."*

## ★ Scope ruling: build (b) over a Vector. Do NOT invent `Stream`.

The design defines (b) as sugar over item (a): `(write-logs-batched journal items)` ≡
`(write-logs-stream journal (stream-from items))`. Measured this session:

- **`Stream` does not exist** — no `defsurface`/`typealias`/`defrecord` for it anywhere in `wat/`.
- **`:wat::query::WriteResult` does not exist** either.
- **No chunker exists** anywhere in the stdlib.

So (a) as written needs two new types plus the fold. **The span does not need any of that**: it holds
its whole buffer in hand, which is precisely (b)'s "have-it-all-in-hand" case.

Building `Stream` to satisfy an equivalence, with no lazy producer anywhere to consume it, is
building an abstraction before its second user. **(a) waits for a consumer that actually streams.**
The equivalence stays true the day (a) lands; it just is not what makes (b) correct today.

## The rule

> Fragment a batch into submissions that each fit the op's declared cap, write them in order, and
> report **exactly how many items were written**.

Sizing is item (a)'s own settled decision, transcribed: *"fold accumulating each item's encoded
byte-length; cut a batch when the next item would cross the budget. Exact `edn::write` length beats
an estimate (the encode is needed anyway)."* The span already measures exactly this way.

The cap is read from the contract (`Journal::WRITE-{LOGS,METRICS}-MAX-REQUEST-BYTES`), never a
literal — as stone A established.

★ **Cut at `>`, not `>=`.** The server rejects at `>` (`service.wat:1779`). A chunk sized to exactly
the cap is legal, and the span's `>=` trigger is a *when-to-flush* heuristic, not a *what-fits* rule.
Conflating them is what produced the unflushable buffer in the first place.

## ★ THE CONTRACT DECISION: partial progress must be exact

One write is all-or-nothing. **Chunked writes are not.** If chunks 1–3 land and chunk 4 fails, the
caller must learn that **exactly** — because the span will drop the written prefix and retain the
rest:

- report **fewer** written than actually landed → those items are re-sent → **duplicate logs**
- report **more** than landed → those items are dropped → **silently lost logs**

So the writer returns the **count of items written** alongside the outcome, and the span resets to
the un-written suffix. That is stone A's reset-only-on-success discipline applied per chunk instead
of per flush — the same rule, finer grained.

**An off-by-one here is a data bug in both directions**, which is why it is the stone's load-bearing
gate rather than a detail.

## The edge that must not loop: one item over the cap

A single item whose encoded length alone exceeds the cap can never be placed in any chunk. It must be
**surfaced as its own failure**, never skipped (silent loss) and never retried forever (a livelock
that would hang the flush).

The vocabulary already exists and is the honest one: `RequestTooLarge{bytes, cap}` — the same
variant the server would return, reported by the client that can see it first. This is the
io-budgets model's own line: *"Client tooling to fit the budget"* — a client that can measure can
also refuse.

## Where it lands

`wat/telemetry.wat` — `write-logs-batched` and `write-metrics-batched`, the two the span needs.
`flush-logs`/`flush-metrics` call them instead of `Journal/write-*` directly, and reset to the
un-written suffix rather than to empty-on-success.

## Out of scope = REJECTED

- **`Stream`, `stream-from`, item (a)** — see the scope ruling above.
- **`WriteResult` as a new named type** — the count plus the existing response says everything the
  caller needs; a new type is warranted when a second consumer wants it.
- **Retry policy.** A refused chunk stays buffered and the next flush retries it by construction.
- **Backpressure and drop policy.** Still the builder's ruling, and still unmade — though this stone
  removes the *unbounded* growth that made it urgent.
