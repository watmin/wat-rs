# BRIEF — a give-back is counted

Add one durable counter to `:fanout::worker`, surface it on the existing `disrupts` channel, and
print it in the summary. `wat-scripts/fanout/circuit.wat` only.

Read `DESIGN-a-give-back-is-counted.md` first. **The exemplar to copy is `disrupt-hits`** — the
same file already runs this exact path for a sibling counter, so copy it rather than inventing a
shape.

## READ IN ORDER

| room | why you are there |
|---|---|
| `circuit.wat:221-226` | `DisruptsResponse::Ok [hits draws points]` — gains a fourth field, `gave-back <- :wat::core::i64` |
| `circuit.wat:236-250` | the worker's `:durable` — gains `gave-back <- :wat::core::i64`, init `0` |
| `circuit.wat:319-330` | the `disrupts` impl — reads the counter, exactly as it reads `disrupt-hits` |
| `circuit.wat:474-477` | **the give-back arm.** The counter increments here, and only here |
| `circuit.wat:1010-1020` | `sum-disrupts` — currently returns one number; it must carry both |
| `circuit.wat:1346, 1381` | `dhits` and the summary format string — add `gave-back={gb}` |
| `circuit.wat:604-609` | `held-worker`'s `disrupts` stub returns `(Ok 0 0 "")` — needs the fourth field to stay type-correct. **It stays a stub** |
| `circuit.wat:1740-1760` | every fixture constructing a worker `Record` needs the new field |

## SKETCH

In the give-back arm, the accumulator already returns unchanged outcomes; the counter is the one
thing that moves:

```wat
;; The budget is spent … leave it unacked, let visibility redeliver it.
;; Counted: this is the only place a give-back happens, and an uncounted
;; give-back is indistinguishable from an exhaustion that never occurred.
(:wat::core::Tuple q0 seen1 outs0)     ;; + the worker's gave-back += 1
```

⚠ The fold's accumulator is a 3-tuple `(q, seen, outs)` and the counter lives in worker
**state**, not in the fold. If threading it requires widening the accumulator, that is fine —
but say so in the SCORE, because it touches every arm of the fold.

## BLAST RADIUS

`circuit.wat` only. No `wat/`, no `sqs.wat`, no `src/`, no `.rs`, no nextest config.

## STOP TRIGGERS

- **STOP-1** — if `gave-back` cannot reach the summary without a new feature on `:fanout::Worker`,
  STOP and report. The `disrupts` channel exists precisely for this and a second channel is a
  design change, not an implementation detail.
- **STOP-2** — if the counter cannot be incremented in the give-back arm alone (e.g. it would
  also fire on a `PeerGone` or a successful check), STOP. **A counter that counts more than its
  name is worse than no counter** — it is the ambiguity this stone exists to remove.
- **STOP-3** — do not change the drop rate, the retry budget, or the number of runs in any
  fixture to make the number larger.
- **STOP-4** — no perf work, no queue-side knobs, no fixture repairs.

## PRIOR RESULT TO COPY FOR SHAPE

`SCORE-a-worker-gives-the-message-back.md` — the stone this makes visible, and my grading on it
naming exactly this gap.
