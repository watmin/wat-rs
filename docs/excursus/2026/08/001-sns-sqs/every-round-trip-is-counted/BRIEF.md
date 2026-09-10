# BRIEF — every round-trip is counted

Make process-boundary crossings a reported budget, attributed by peer class, **adding zero round-trips
to do it.** One file if possible.

**Read in order:**

1. `docs/excursus/2026/08/001-sns-sqs/every-round-trip-is-counted/DESIGN.md` — why this is an instrument
   and not an optimisation, and the three conclusions its absence already broke.
2. `wat-scripts/fanout/circuit.wat:2578` — the phase-line format string. **The only three round-trip
   counters that exist are `store-calls`, `queue-receive-calls`, `poll-calls`.** Your budget line goes
   here.
3. `wat-scripts/fanout/circuit.wat:400` — `:fanout::Seen/check`, called **per delivery**. With
   `seen-recorded=8000` at `2000 4 3`, this is ≥8000 crossings that appear in no number today.
4. `wat-scripts/fanout/circuit.wat`, `:fanout::sample-of` (added at `202763705`) — the exemplar for
   taking **one** reply and reading many fields from it. Your counters must obey the same discipline.
5. `wat/service.wat:2928-2968` — the generated `stop`: one `send Admin::Stop` + one `recv`, returning an
   author-chosen `:stop` projection. **This is your free channel** for a worker's own internal counts:
   a round-trip the harness already makes.
6. `wat-scripts/fanout/circuit.wat:2533` — `:fanout::sum-disrupts wpeers`, the *second* question `collect`
   asks each worker. If your worker counts ride the stop projection, this call may become removable —
   note it, do not necessarily remove it.

**Sketch:**

```
;; per peer class, counted where it is FREE to count
store-rt   already have (store-calls, per tier)
queue-rt   already have (queue-receive-calls)  + publish-attempts
poll-rt    already have (poll-calls)
seen-rt    the CONSUMER counts its own Seen/check + Seen/mark in local state,
           and reports them on the stop projection it already returns
worker-rt  the harness counts its own calls to workers locally  (2 per worker today)
topic-rt   the harness counts its own calls to the topic locally

phase line gains:  rt-store=… rt-queue=… rt-seen=… rt-worker=… rt-topic=… rt-total=…
```

**Blast radius, as a property:** `wat-scripts/fanout/circuit.wat` should be enough — the consumer, the
worker and the harness all live there. **If you need to change `wat-scripts/queue/sqs.wat` or `wat/`,
STOP** (STOP-3) and report what forced it.

**STOP-1 — the instrument must not inflate what it measures.** If any count cannot be obtained without a
new round-trip, **report it as `unknown` and say which** rather than paying for it. An honest gap beats a
self-inflating instrument. **This is the whole discipline of the stone** — `sample-of` exists because four
folds each paid a round-trip to read one field.

**STOP-2 — if the phase timings move materially, STOP and report.** This adds counters, not work. A
measurable slowdown means the instrument is costing something, and that is a finding, not a rounding
error.

**STOP-3** — if this cannot stay inside `circuit.wat`, stop before the second file and report what forced
it.

**STOP-4** — on any red in the floor or corpus gate: do **not** re-run. Capture whole, name the exact arm,
surface it. ⛔ Earlier today a chaos run piped through `grep` lost its failure text and the re-run went
green, destroying the evidence.

**Measure**: `2000 4 3 8192 true 1000` ×3 before and after, box quiet, all phases plus every counter.
⚠ `collect` at small n has been measured at **±20 %** spread, so **three runs per side is the minimum**
for any claim about it.

**Run everything heavy through `./scripts/capped.sh`** (`--limit 8g`). `.wat` is edited with an editor;
scratch `.wat` goes in `wat-scripts/scratch-pad/`. **Read the floor's Summary line, never a piped exit
code.** Leave everything uncommitted.

**Write your SCORE** to `docs/excursus/2026/08/001-sns-sqs/every-round-trip-is-counted/SCORE.md`, graded
row by row, with the full budget for one standard run and the before/after phase table. Copy the shape of
`docs/excursus/2026/08/001-sns-sqs/one-sample-per-boundary/SCORE.md`.
