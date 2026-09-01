# BRIEF — item (c) stone C: a size-triggered flush must speak

Three arms compute a write failure and throw it away. Make them report it. `Ok` keeps meaning
**"accepted"**; the failure becomes matchable.

Read `DESIGN-STONE-the-size-triggered-flush-must-speak.md` beside this first — it carries the ruling
against two success values and the one subtlety that turns this fix into a data-loss bug if missed.

## Read in order, and why you are being sent there

1. **`wat/telemetry/span.wat`, the `log` arm** — find `pair0`, then `s1 (:wat::core::first pair0)`.
   `(:wat::core::second pair0)` is never read. That is the defect, once. `incr` and `timed` have the
   identical shape against `flush-metrics`.
2. **`wat/telemetry/span.wat:170` and `:179`** — `flush` and `close`, the only two sites that DO read
   `(second pair)` and map it onto their response. **This is the exemplar**: copy how they map, do
   not invent a mapping.
3. **`wat/telemetry/span.wat:361-384`** — `flush-logs`. Note `(_ (:wat::core::Tuple s cresp))`: on
   failure it returns the ORIGINAL state, buffer intact. **Do not touch this.** It is why no data is
   lost today and why this stone is small.
4. **`wat/telemetry.wat`** — `Span::{Incr,Timed,Log}Response`, each `Ok`/`RequestTooLarge`/
   `RequestMalformed`. Each gains `Constraint`/`Transient`/`Fatal`, spelled exactly as
   `CloseResponse` and `FlushResponse` already spell them.

## The work

**1. Three variants on three responses.** Copy `CloseResponse`'s `Constraint`/`Transient`/`Fatal`
lines verbatim — same names, same `:wat::query::` payload types. These are the sink's failures
surfaced pass-through, never a new taxonomy.

**2. Read the second element.** In each of the three arms, match `(second pair0)`: `Done` → the arm's
`Ok`; each failure → the arm's matching variant, carrying the same `err`.

**3. ★ Keep the item that arrived.** The log or sample that triggered the flush is NOT part of the
failed batch — it is the one that made the batch too big. The arm must return the failure response
**with the accumulated state**, so the new item is still buffered. Report what failed; keep what
arrived.

**4. The cascade.** Adding variants makes existing matches non-exhaustive. The checker names each
site; census says exactly two `.wat` files match these enums (`span.wat` and
`probe_arc278_span_surface.wat`). Add real arms — never a `_` wildcard on a response enum, which is
the swallow this whole arc removed.

## Blast radius

`wat/telemetry.wat` (three enums), `wat/telemetry/span.wat` (three arms), and whatever the checker
names. **No change to `flush-logs`/`flush-metrics`. No runtime change. No new op.**

## STOP triggers

**STOP-1 — do not drop the arriving item.** If the failure path returns a state without the new log
or sample buffered, STOP: that trades a silent failure for silent data loss, which is strictly worse
than what is there now.

**STOP-2 — no `_` wildcard on a response enum.** If the cascade tempts you to wildcard a match arm,
STOP and add the real arms. A wildcard here re-creates the exact swallow this stone removes.

**STOP-3 — do not touch the flush functions.** `flush-logs`/`flush-metrics` reset only on success and
that is correct. If you find yourself editing them, the fix has gone somewhere it does not belong.

**STOP-4 — no second success value.** `Ok` stays one thing meaning "accepted". A `Buffered` variant
was four-questioned and rejected (see DESIGN); if the work seems to want one, STOP and report why
rather than adding it.

## The gates to write

- **the failure reaches the caller:** a span whose sink fails, driven past the size cap via `log` —
  the caller must receive `LogResponse::Fatal` (or the matching variant), **not `Ok`**. This gate is
  RED today and is the whole stone.
- **and the arriving item survives it:** after that failed flush, the buffered batch must still
  contain both the un-flushed logs AND the one that triggered it. Prove it by a later successful
  flush landing all of them.
- the same for `incr`/`timed` against the metrics cap.

## Prior comparable result

`SCORE-item-c-stone-b-two-clocks.md` beside this. Its Row 8 section records a row of MINE that was
the wrong test — worth reading before trusting any row in this brief literally.
