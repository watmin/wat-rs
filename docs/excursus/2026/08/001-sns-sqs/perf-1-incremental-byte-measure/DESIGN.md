# DESIGN STONE — perf 1: the incremental byte measure

**Commissioned 2026-09-01.** A specification error of mine from item (c) stone A, measured rather
than argued.

## The defect, measured

`log`, `incr` and `timed` each decide the size trigger by encoding the **whole would-be batch**:

```wat
would (:wat::core::conj logs0 l)
bytes (:wat::string::length (:wat::edn::write (…WriteLogsRequest would)))
```

`wat-scripts/scratch-pad/probe-span-log-cost.wat`, logging into a span whose caps are large enough
that no flush fires:

```
250 logs  ->  224ms
500 logs  ->  618ms   (2.76x per doubling)
1000 logs -> 1848ms   (2.99x per doubling)
```

Linear predicts 2× per doubling, quadratic 4×. The ratio is **climbing** — 2.76 → 2.99 — which is the
quadratic term taking over as the buffer grows. At 1000 logs that is **1.85 ms per `log` call**, for
an operation whose useful work is a `conj`.

## Why it is wrong, in the design's own terms

`DESIGN-service-io-budgets.md` item (a) justified exact measurement:

> *"Exact `edn::write` length beats an estimate (**the encode is needed anyway**)."*

The encode **is** needed anyway — **once, at flush time**. I turned that into an encode *per
accumulation*, which is a different claim and one the justification does not support. Every log pays
to re-encode every log before it.

## The rule

> Carry the accumulated encoded size in the Record. Each arriving item adds **its own** encoded
> length. One encode per item, not one per item per call.

O(n) to fill a buffer of n, instead of O(n²).

## ★ The contract decision: the running total must stay EXACT, not approximate

A sum of per-item encodings is not identically the encoding of the whole request — there is
container framing (the request record, the vector's delimiters and separators). So the running total
must account for that framing, not ignore it.

**And the direction of any residual error is not negotiable.** The trigger exists to keep a batch
under the server's cap; an under-count ships an over-cap batch and the server refuses it
(`RequestTooLarge`). So:

- the accounted size must be **≥** the true encoded size, never below it, **or**
- it must be exact.

A conservative over-count costs an occasional early flush. An under-count costs a refused write.
Those are not symmetric, and the stone must state which it achieves and prove it.

★ **The gate is a differential, not an assertion:** for a range of batch shapes, the running total
must equal (or exceed, stated) `string::length(edn::write(request))` computed the old way. That is
the only check that cannot be satisfied by a plausible-looking arithmetic that drifts.

## What must not change

- **The trigger's behaviour.** Same cap, same `>=` comparison, same flush points for the same input.
  This is a cost change, not a semantics change.
- **`write-*-batched`'s chunker.** It measures at flush time, where the encode is genuinely needed,
  and it cuts at `>`. Untouched.
- The bound (stone D) counts ITEMS and is unrelated.

## Out of scope = REJECTED

- The store's `sort-by`-per-read (`mem.wat:190, 217`). Pre-existing, documented as *"correct, not
  fast"*, and its own stone — perf 2.
- Caching encoded bytes per `Log` for reuse at flush time. Plausible, unmeasured, and a second
  change riding on the first.
