# BRIEF — the trace separates durable work from waiting

Add `t0b` and `t3b`, delete the phantom `t2`, and report five real stages.
`wat-scripts/topic/sns-fanout.wat` + `wat-scripts/fanout/circuit.wat`. Telemetry only. Read
`DESIGN-the-trace-separates-durable-work-from-waiting.md` first.

## READ IN ORDER

| room | why |
|---|---|
| `wat-scripts/topic/sns-fanout.wat:481` | `:t1 t1 :t2 t1` — the phantom. `t2` is `t1` |
| `sns-fanout.wat` `publish` impl | where `Queue/send-all` returns — `t0b` is stamped the instant it does |
| `sns-fanout.wat` topic-worker, around `:481` | where the bucket is built (`t3`) and sent — `t3b` is stamped after that send returns |
| `wat-scripts/fanout/circuit.wat:1099`, `:1276` | `stamped-range` / the child's inline `stamp` — where `t0` is written by the publisher |
| `circuit.wat:1703-1720` | `traces-add` — splits the body on `\|`, expects **6** parts, derives the five hops |
| `circuit.wat:1656-1662` | the `Traces` record — `outbox`/`hop12`/`hop23`/`pending`/`e2e` |
| `circuit.wat:1737-1745` | `traces-report` — the histogram lines |

## SKETCH

Body carries seven stamps instead of five: `msg|t0|t0b|t1|t3|t3b|t4`.

`traces-add`'s part count changes from 6 to 8; the record's fields become:

```
publish-work   t0  -> t0b     RPC + the topic's Store/put
inbox-wait     t0b -> t1      pure queueing
worker-proc    t1  -> t3      topic-worker processing (was hop23)
fanout-work    t3  -> t3b     RPC + the subscriber queue's Store/put
subq-wait      t3b -> t4      pure queueing
e2e            t0  -> t4      unchanged
```

`hop12` is **removed**, not renamed — there is nothing at that point to measure.

⚠ `t0b` must be stamped by the **topic**, not the publisher: the publisher only learns the outcome
after the RPC returns, so a publisher-side stamp would fold the reply hop into "durable work".

## BLAST RADIUS

`wat-scripts/topic/sns-fanout.wat`, `wat-scripts/fanout/circuit.wat`, scratch probes.
**No `wat/`, no `src/`, no `sqs.wat`.** No behaviour changes — only the body string grows.

## STOP TRIGGERS

- **STOP-1** — ⛔ `publish`, `drain`, `collect` or `distinct` move. Two extra stamps lengthen every
  body slightly; if a phase moves measurably, **report it** — the trace would be perturbing the
  thing it measures, which is the one thing telemetry may not do.
- **STOP-2** — `t0b` cannot be stamped inside the topic where the send returns (e.g. the outcome is
  assembled somewhere the timestamp cannot reach). Report the shape; do **not** stamp it on the
  publisher side.
- **STOP-3** — the body exceeds a declared limit with the extra stamps. `Queue::send` declares
  `:max-request-bytes 524288`; report the margin rather than raising anything.
- **STOP-4** — anything outside the blast radius.

## THE MEASUREMENT

`circuit.wat` ×3, shipped config, **unpinned**. Report all five stages plus `e2e`, and the phase
line, against today:

```
outbox   50-250=7989  max 236ms      ← splits into publish-work + inbox-wait
t2->t3   <1ms=7997                   ← becomes worker-proc, expected unchanged
t3->t4   10-50=5274 50-250=2232      ← splits into fanout-work + subq-wait
publish  ~20700   collect ~5900   stop ~400
```

★ The number the stone exists for: **what fraction of `outbox` and `t3->t4` is the durable write?**

## GRADE AGAINST

`SCORE-the-instrument-reports-what-happened.md` — the arc's other telemetry stone, whose gate was
also "the measured quantities must not move".

Write `SCORE-the-trace-separates-durable-work-from-waiting.md`, then `pulsare_yield kind=scored`.
