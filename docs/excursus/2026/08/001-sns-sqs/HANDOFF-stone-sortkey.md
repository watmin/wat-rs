# HANDOFF → grok — excursus 001 stone SORTKEY

Same branch, `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-sortkey.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-sortkey.md`
- `docs/excursus/2026/08/001-sns-sqs/NOTE-journal-loses-metrics-on-sqlite-because-sk-is-time-only.md`
  — **including its ⛔ CORRECTED section.** The table at its top is half a measurement; the bug
  is NOT sqlite-specific.

**This is the excursus's largest stone, and the first expected to produce a FULLY GREEN floor.**
Every prior stone ran against a known red. This one fixes it. `FLOOR=0` is the target.

`journal`'s base key is `(namespace + kind, time-ns)` — nothing in it says WHICH event. A span's
close emits three Metrics at one instant, they share a key, `put` replaces, two are lost.
Measured on **both** backends: `span_macros` gets 1 where it asserts 3.

**The ruling is option C** (the four questions, all four YES): a telemetry event carries its own
id. `Scope` is a `defsurface` spliced into `Metric` and `Log`, so **one field reaches both**.
The user-facing surface — `(:wat::telemetry::log span :Info "…")`, `incr`, `timed` — does not
change at all; users never construct these records.

★ **Two things will decide whether this works:**

**Row 4** — the three Metrics of one `close` share `now`. That sharing IS the bug. If they end
up sharing an event id too, nothing is fixed and `span_macros` still returns 1.

**Row 7 / STOP-2** — the range bounds. A `SortKey` record renders longer than a bare timestamp,
so a row at exactly `time-hi` must still fall inside `sk-hi`. If the maximal sentinel is not
truly maximal, `query-metrics` silently drops the newest data **and every existing fixture
still passes**, because none queries a boundary. **Demonstrate it; do not argue it.** Same
class as the `#inst` width bug — an ordering property nothing asserted.

The BRIEF's census is **known-incomplete and says so**. Re-derive it and report your number.

Verify in the FOREGROUND; read the Summary line. On a NEW red: do NOT re-run, capture the arm
whole, name the exact assertion.
