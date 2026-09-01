# BRIEF — perf 1: the incremental byte measure

Stop re-encoding the whole buffer on every `log`/`incr`/`timed`. Carry a running byte total; each
arriving item adds its own encoded length. **Measured**: 1.85 ms per `log` call at a 1000-entry
buffer, growing superlinearly (2.76× then 2.99× per doubling).

Read `DESIGN-STONE-the-incremental-byte-measure.md` beside this first — the contract decision is
about the *direction* of any residual error, and it is not symmetric.

## Read in order, and why you are being sent there

1. **`wat/telemetry/span.wat:214-217`** — the `log` arm's `would` / `bytes` pair. The defect, once.
   `incr` (`:64`-ish) and `timed` (`:135`-ish) carry the same shape against the metrics request.
2. **`wat-scripts/scratch-pad/probe-span-log-cost.wat`** — the measurement above. **Re-run it before
   and after**; it is the stone's own evidence and its numbers go in the SCORE.
3. **`wat/telemetry.wat`** — the Record (stone B/D added fields there); the running totals live
   beside them.
4. **`wat/telemetry.wat`'s `write-*-batched`** — measures at flush time and cuts at `>`. **Do not
   touch it**: that encode is genuinely needed and is not per-accumulation.

## The work

**1. Two running totals on the Record** — one for the logs request, one for the metrics request.

**2. Each arriving item adds its own encoded length.** One `edn::write` of the item, not of the
batch.

**3. Account for container framing** so the total tracks the real request encoding — the record
wrapper, the vector delimiters, the separators between items.

**4. Reset with the accumulator.** A flush that empties `logs` must reset its total; a partial drain
(item (b) leaves a suffix) must leave the total consistent with that suffix. **Getting this wrong is
a slow drift, invisible until a batch is refused.**

## Blast radius

`wat/telemetry.wat` (Record fields), `wat/telemetry/span.wat` (three arms + the flush reset paths).
**No runtime change, no surface change, no chunker change.**

## STOP triggers

**STOP-1 — never under-count.** The accounted size must be exact, or conservatively HIGH. An
under-count ships an over-cap batch that the server refuses; an over-count costs an occasional early
flush. State in the SCORE which you achieved.

**STOP-2 — the trigger's behaviour must not move.** Same input, same flush points. This is a cost
change. If a flush point shifts, say so and why.

**STOP-3 — partial drains.** Item (b) can leave an un-written suffix. The total must match the
suffix that remains, not the batch that was attempted. If you cannot keep them consistent, STOP —
a drifting total is worse than a slow one.

**STOP-4 — do not touch the chunker.** `write-*-batched` measures at flush time where the encode is
needed. Out of bounds.

## The gates to write

- **★ the differential:** across a range of batch shapes and sizes, the running total equals (or
  exceeds — state which) `string::length(edn::write(request))` computed the old way. **This is the
  only gate that cannot be satisfied by plausible-looking arithmetic that drifts.**
- **the drift after a partial drain:** flush against a sink that accepts one chunk and refuses the
  next, then re-check the differential against the remaining suffix.
- **the cost:** re-run `probe-span-log-cost.wat`. The per-doubling ratio must approach 2×, not 3×.
- **behaviour unchanged:** every stone A/B/C/D and item-(b) gate still passes, assertions unedited.

## Prior comparable result

`SCORE-item-b-batched-writer.md` — the first no-delta stone, and its note on why: decisions made
against measurements taken first.
