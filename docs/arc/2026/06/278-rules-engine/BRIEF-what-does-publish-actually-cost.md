# BRIEF — what does publish actually cost

Measure five primitives on one quiet box, then check whether unit × count explains the 37.3 s
publish. **No production change.**

Read `DESIGN-what-does-publish-actually-cost.md` first.

## READ IN ORDER

| room | why |
|---|---|
| `wat-scripts/scratch-pad/probe-what-a-scan-costs.wat` | **the harness to extend.** It already starts a sqlite store + queue, fills to depth, and times a loop with warmup |
| `wat-scripts/queue/sqs.wat` — the `send` arm | what a publish actually does per message: `total` (count-index) then `Store/put` |
| `wat-scripts/fanout/circuit.wat:996` | where `t0` is stamped — the publish the 37.3 s measures |
| `wat-scripts/scratch-pad/probe-what-a-process-impl-can-call.wat` | **the two-loci shape.** A process handle is `(Handle :- [Wire])`, a thread handle `(Handle :- [Shared])`; `if` cannot unify them, so write two functions, not one with a flag |

## THE FIVE UNITS

1. **bare round trip, thread locus** — a trivial service, one nullary arm returning a constant.
2. **bare round trip, process locus** — same service, `process/post-spawn`.
3. **`Store/put`** — one row, sqlite `:memory:`, with the queue already at depth ~60.
4. **`Store/count-index`** — the cap gate's call.
5. **`Store/scan-index` limit 1** — for contrast with 4.

Each: **warm up, then time ≥1000 iterations**, report µs/op. Same run, same box.

## THE ARITHMETIC

Report, explicitly:

```
predicted = 8000 × (put + count-index + 2 × round-trip-process)
actual    = 37300 ms
gap       = actual − predicted
```

⚠ **Report the gap whatever it is.** A large gap is a finding, not a failure — it means the cost
is somewhere the model does not look, and that is the most valuable thing this stone can produce.

## BLAST RADIUS

`wat-scripts/scratch-pad/` only. **No `wat/`, no `src/`, no `sqs.wat`, no `circuit.wat`.**

## STOP TRIGGERS

- **STOP-1** — if a bare round trip cannot be measured without a service that does real work,
  STOP and report the shape. The *floor* is the number; a trip that does work is not the floor.
- **STOP-2** — check `ps` before every timing loop and say so in the SCORE. A contended box has
  invalidated a perf claim in this arc already.
- **STOP-3** — if `2 × round-trip` is the wrong call count for a publish, **say what the real
  count is and use it.** Do not silently fit the model to the answer; the whole point is whether
  an honest model closes.
- **STOP-4** — change nothing outside `scratch-pad/`. If a measurement seems to require a
  production edit, STOP and report.

## PRIOR RESULT TO COPY

`SCORE-stop-fetching-rows-to-get-a-number.md` — its row 6 is the shape: a unit cost, measured
against a named baseline, that made a 21 s claim checkable.
