# BRIEF — a message is fanned once

When the inbox admits a prefix that splits a message's fanout, the topic tops up that message's
remaining pairs before replying, and counts it. Read `DESIGN.md` first —
it carries the three dead hypotheses so you do not re-run them.

## READ IN ORDER

| room | why |
|---|---|
| `wat-scripts/topic/sns-fanout.wat:84-99` | `publish` — the msg-major fold that builds `bodies` |
| `sns-fanout.wat` `publish`'s send + reply | where `Accepted k` comes back and `floor(k/nsubs)` is computed today |
| `wat-scripts/queue/sqs.wat:344-362` | the admission gate — `room`, `take`, prefix. This is what returns `k` |
| `wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat` | the working probe: `nsubs4-room6=Accepted(1)` is exactly the partial-fan case you are fixing |

## SKETCH

Inside `publish`, after the first `Queue/send` returns `Accepted k` over `n = count(msgs) × nsubs`:

```wat
rem  (:wat::i64::mod k nsubs)          ;; 0 => clean boundary, nothing to do
;; rem > 0: the message at index (k / nsubs) has `rem` of its nsubs pairs enqueued
need (:wat::i64::- nsubs rem)          ;; the pairs still missing, 1 .. nsubs-1
```

- `rem == 0` → reply `Accepted (k / nsubs)` — unchanged
- `rem > 0`  → send exactly those `need` pairs (the tail message's remaining subscriber indices)
  - top-up `Accepted need`  → reply `Accepted ((k / nsubs) + 1)`
  - top-up anything less    → reply `Accepted (k / nsubs)` — today's behaviour

⚠ The top-up must carry **the same bodies** the first send would have — same `"{i}|{msg}"` format,
same subscriber indices, so a delivered pair is byte-identical whichever send carried it.

## BLAST RADIUS

`wat-scripts/topic/sns-fanout.wat` and scratch probes. **No `wat/`, no `src/`, no `circuit.wat`,
no `sqs.wat`.** The driver's retry policy does not change — this stone makes its re-publishes
rarer, not different.

## STOP TRIGGERS

- **STOP-1** — the topic cannot reconstruct which subscriber indices are missing from `k` alone.
  Report it; the msg-major ordering is what makes `k mod nsubs` meaningful and if that is not
  reliable the whole design rests on sand.
- **STOP-2** — the top-up needs more than one extra send (e.g. it too gets split). Report the
  measurement; do **not** loop until it lands — an unbounded internal retry inside a service arm
  is a hang, and this arc has paid for that lesson.
- **STOP-3** — `dup` does not fall. **That is a finding, not a failure** — report it with the
  `outbox` histogram, because it would mean the duplicates come from somewhere other than the
  partial fan and the DESIGN's mechanism is wrong.
- **STOP-4** — anything outside the blast radius, or any `:cap` change.

## THE MEASUREMENT THAT DECIDES THIS STONE

`circuit.wat` ×5, reporting **`dup`, the `outbox` histogram, and `publish`** together. The DESIGN
predicts they move as one: fewer duplicates → fewer items in `outbox` → shorter waits → lower
`publish`. If `dup` falls and `publish` does not, say so plainly — that breaks the drain-bound
model and is worth more than the stone.

## GRADE AGAINST

`docs/excursus/2026/08/001-sns-sqs/the-server-manages-its-own-capacity/SCORE.md` — the stone that created this, same file.

Write `SCORE.md`, then `pulsare_yield kind=scored`.
