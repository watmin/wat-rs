# BRIEF — the benchmark has more than one publisher

Give `run-with` a publisher count, run P publishers concurrently each with its own seed, and
re-measure the three backoff policies at p=1 and p=3. `wat-scripts/fanout/circuit.wat`. Read
`DESIGN.md` first — it corrects a grading of mine and
carries the integrity gate that matters more than the new numbers.

## READ IN ORDER

| room | why |
|---|---|
| `circuit.wat:1379-1385` | `run-with`'s parameter list — where `p` joins `n`, `m`, `j` |
| `circuit.wat:241-258` | the `Worker` surface: `start` fires, `disrupts` reports. **This is the shape to mirror** |
| `circuit.wat:852` | `start-worker!` — how the parent fires one |
| `circuit.wat:1525-1530` | how the parent dials and starts the whole worker set |
| `circuit.wat:1044` | `publish-until-accepted!*` — the loop that moves inside a publisher, seed and all |
| `circuit.wat:1534` | the current single sequential publish fold — what P publishers replace |

## SKETCH

A `Publisher` surface mirroring `Worker`:

```wat
(start [self <- :fanout::Publisher  req <- :fanout::Publisher::StartRequest]
   -> :fanout::Publisher::StartResponse   :max-request-bytes 524288)
(stats [self <- :fanout::Publisher  req <- :fanout::Publisher::StatsRequest]
   -> :fanout::Publisher::StatsResponse   :max-request-bytes 524288)
;; StatsResponse::Ok [done? <- bool  calls <- i64  retries <- i64]
```

Durable record: `id`, `topic-addr`, `lo`, `hi` (its half-open share of the id range), `seed`.

Parent: spawn P, `start` each, poll `stats` until every `done?`, sum `calls`/`retries`.
`publish` = first spawn → last done.

★ **Seeds:** `seed_i = BACKOFF-SEED + i * 7919` (any spreading derivation, but they must differ).
A shared seed makes P publishers emit identical delay sequences — a synthetic herd that is an
artifact of the harness, and the experiment would answer nothing.

★ **Ids:** publisher `i` owns `[lo, hi)` of `0..n`, so `distinct=8000` still catches a partition
bug. Do not interleave by modulo unless the ranges provably tile.

## BLAST RADIUS

`wat-scripts/fanout/circuit.wat` and scratch probes. **No `sqs.wat`, no `sns-fanout.wat`, no
`wat/`, no `src/`.** This is harness work; the system under test does not change.

## STOP TRIGGERS

- **STOP-1** — ⛔ **p=1 does not reproduce today's numbers** (publish ~20379, retries ~1386,
  receives ~5032, `distinct=8000`). **STOP and report.** It would mean the harness changed the
  measurement, and every baseline the last six stones were graded against is void. This is the
  most important trigger in the brief.
- **STOP-2** — publishers cannot run concurrently (the parent serialises them). Report the shape;
  a sequential "P publishers" is the same single-client experiment with extra steps.
- **STOP-3** — ids collide or a share is dropped, so `distinct != 8000`. Report; do not paper over
  with a wider id space.
- **STOP-4** — the fixed-25 ms policy does **not** degrade at p=3. **That is a finding, not a
  failure** — it would mean jitter is unjustified here at any cardinality we ship, which is worth
  more than this stone.
- **STOP-5** — anything outside the blast radius.

## THE MEASUREMENT

Three policies × two cardinalities, ×3 runs each, same box. The policy is selected by editing the
one delay expression; **do not build a policy-selection knob** — a harness flag is a new surface
and this stone is already the harness.

```
                       p=1 (known)     p=3
adaptive (shipped)       20379          ?
fixed 25 ms              18472          ?
fixed  1 ms              22395          ?
```

Report `publish`, `full-retries`, `queue-receive-calls`, `distinct` for all six cells.

## GRADE AGAINST

`docs/excursus/2026/08/001-sns-sqs/the-client-backs-off-intelligently/SCORE.md` — the stone whose framing this corrects.

Write `SCORE.md`, then `pulsare_yield kind=scored`.
