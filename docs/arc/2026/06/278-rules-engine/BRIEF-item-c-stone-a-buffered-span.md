# BRIEF — item (c) stone A: the buffered span

Make `span'` buffer its logs and emit metrics as **deltas**, so a flush can happen more than once
without double-counting. No timer in this stone. No new service, no new surface.

Read `DESIGN-STONE-the-buffered-span.md` beside this first — it carries the contract decision, which
is the only thing here that can be silently wrong.

## Read in order, and why you are being sent there

1. **`wat/telemetry/span.wat:44-62`** — `incr` and `timed`. These are the exemplar for the new `log`:
   they take a request, fold it into `:durable`, and return `Outcome::Reply` with the new state.
   `log` becomes the third of these.
2. **`wat/telemetry/span.wat:85-102`** — `log` today. It builds a `Log` and immediately calls
   `Journal/write-logs` with a one-element Vector. The `Log` construction is correct and stays; only
   its destination changes.
3. **`wat/telemetry/span.wat:104-165`** — `close`. **This is the flush you are extracting.** It
   folds `counters` into one Metric each (`Unit::Count`), folds `durations` into
   `<name>/count` + `<name>/duration` (`Count` + `Nanos`, the sum), ships one
   `WriteMetricsRequest`, and maps the sink's response onto `CloseResponse`. Keep the naming; keep
   the response mapping.
4. **`wat/telemetry.wat:24-30`** — `Samples` is `(Vector :- [i64])`. The span already holds full
   fidelity; only `close` discards it.
5. **`wat/telemetry.wat:119-210`** — `Journal`'s `write-logs`/`write-metrics` and their declared
   `:max-request-bytes`. The size threshold is **read from the contract**, never a literal.

## The work

**1. `logs` accumulator.** `:durable` gains `logs <- (Vector :- [:wat::telemetry::Log])`. `log`
builds its `Log` exactly as now, `conj`s it, and returns `Ok`.

**2. Extract the flush.** `close`'s metric-building becomes a reusable path that emits and **resets**
— counters zeroed, duration samples cleared, logs emptied. `close` calls it and reports the outcome
as it does today.

**3. Durations emit BOTH.** Keep `<name>/count` and `<name>/duration` exactly as they are; ADD one
`<name>/sample` Metric per sample, `Unit::Nanos`.

**4. Size trigger.** After the `conj` in `log`, if the accumulated batch is at/over the threshold,
flush now. Same for `timed` against the metrics budget. Derive the threshold from the op's declared
`:max-request-bytes` — the io-budgets arc made it discoverable for exactly this.

## Blast radius

`wat/telemetry/span.wat`, and `wat/telemetry.wat` ONLY if `:durable` gaining a field forces it.
**No change to `Journal`**, to `Numeric`, or to any surface. No `defservice` machinery.

## STOP triggers

**STOP-1 — the double-count.** If you find yourself writing one emission path for `close` and a
different one for a mid-life flush, STOP. They must be the same path with the same reset; two paths
is the double-count waiting to happen, and it is invisible until a dashboard is wrong.

**STOP-2 — no timer in this stone.** `Outcome::NoReplyAndArm`, `Alarm`, and `-flush-*` internal ops
are stone B. If you are arming anything, STOP.

**STOP-3 — the threshold is not a literal.** If the op's declared `:max-request-bytes` is not
reachable from the arm, STOP and report what is missing rather than hardcoding a number. A magic
constant here silently diverges from the contract the server enforces.

**STOP-4 — `LogResponse::Ok` now means "buffered", not "written".** That is a real weakening of a
promise. Do NOT invent a new response variant to paper over it in this stone; the honest surfacing
of a buffered write's failure is a contract question the builder has not ruled on. Note it in the
SCORE and leave `Ok` as it is.

## The gate to write

A span that: `incr :requests` ×3, flushes, `incr :requests` ×2, closes — must produce counter
metrics summing to **exactly 5**, never 8. That is the double-count, and it is the reason this stone
exists before the timers.

Plus: N logs under a small threshold produce **fewer than N** write-logs calls; and one `timed`
sample set emits `<name>/count`, `<name>/duration`, and one `<name>/sample` per sample.

## Prior comparable result

`docs/excursus/2026/08/002-handle-lifetime-wall/SCORE-stone-3-param-ownership.md` — the shape for a
SCORE, and its delta section shows the standard for reporting what the brief got wrong.
