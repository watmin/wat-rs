# BRIEF — the instrument reports what happened

Split `stop` into `collect` + `stop`, and add a counter for total publish round trips.
`wat-scripts/fanout/circuit.wat` only. Read `DESIGN-the-instrument-reports-what-happened.md` first.

## READ IN ORDER

| room | why |
|---|---|
| `circuit.wat:1938` | `t-stop0` — where the "stop" phase begins today, immediately before the bookkeeping |
| `circuit.wat:1939-1948` | `sum-calls`, `sum-ticks`, `topic-ticks`, `sum-disrupts`, `seen-stats`, `collect-stop` — all bookkeeping, all inside the timed phase |
| `circuit.wat:1982-1983` | the `phases` format string and `:drain`/`:stop` bindings |
| `circuit.wat:1044` | `publish-until-accepted!*` — `retries` is bumped only in the `Accepted 0` branch; the partial branch at the end passes it through |

## SKETCH

```wat
t-collect0  ;; after drain, before any stats call
  … sum-calls / sum-ticks / topic-ticks / sum-disrupts / seen-stats / collect-stop / empty-flags …
t-stop0     ;; teardown begins HERE
  … topic-worker/stop, process reaps …
t-end
```

phases line gains `collect={c}` beside `stop={s}`; `:stop` becomes `(ms t-stop0 t-end)` with the
new `t-stop0` placed after the bookkeeping.

For the counter: thread a `calls` alongside `retries`, incremented on **every** recursion of
`publish-until-accepted!*` — the `Accepted 0` branch **and** the partial branch. Report it as
`publish-attempts`. `full-retries` keeps its current meaning (bounces only).

## BLAST RADIUS

`wat-scripts/fanout/circuit.wat` only. **No `wat/`, no `src/`, no `sqs.wat`, no `sns-fanout.wat`.**
No system behaviour changes.

## STOP TRIGGERS

- **STOP-1** — `publish`, `drain`, `distinct` or `dup` move at all. This stone moves a timer and
  adds a counter; if a measured quantity changes, something else changed and that is the finding.
- **STOP-2** — the phases do not sum to the wall within a few ms. Report the gap; an unaccounted
  remainder is a fourth place time is hiding.
- **STOP-3** — anything outside `circuit.wat`.

## THE MEASUREMENT

`circuit.wat` ×3, shipped configuration (do **not** pin the delay — the DESIGN records why pinning
corrupts `asleep`).

```
today   setup 12297  publish 20812  drain 186  stop 6797
        (stop = collect ~4823 + teardown ~1790, measured by instrumenting it)

after   setup ~12300  publish ~20800  drain ~190  collect ~4800  stop ~1800
```

★ `stop` should **drop to ~1.8 s** and `collect` should appear at ~4.8 s. Publish and drain must
not move.

## GRADE AGAINST

`SCORE-the-benchmark-has-more-than-one-publisher.md` — the arc's other instrument stone, whose
gate was that the old numbers reproduce.

Write `SCORE-the-instrument-reports-what-happened.md`, then `pulsare_yield kind=scored`.
