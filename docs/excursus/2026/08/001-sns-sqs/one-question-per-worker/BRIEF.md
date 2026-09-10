# BRIEF — one question per worker

Move `:fanout::worker`'s disrupt counters onto its `:stop` projection so `collect` asks each worker
**once** instead of twice, and delete `:fanout::sum-disrupts`.

**Read in order:**

1. `docs/excursus/2026/08/001-sns-sqs/one-question-per-worker/DESIGN.md` — the measurement, and the
   contract decision it takes away.
2. `wat-scripts/fanout/circuit.wat:2533` — `dpair (:fanout::sum-disrupts wpeers)`, the second question.
   Its five outputs (`dhits`, `ce`, `me`, `ars`, `aes`) all feed the summary.
3. `wat-scripts/fanout/circuit.wat`, `:fanout::sum-disrupts` and `:fanout::collect-stop` — the two folds
   over the workers. **After this stone there should be one.**
4. `wat/service.wat:2928-2968` — the generated `stop`: one `send Admin::Stop` + one `recv`, returning the
   author's `:stop` projection. **Your channel, on a round-trip already being made.**
5. `wat-tests/service-stop-resp.wat` — the worked exemplar. A service whose `:stop` renders final state to
   an `i64`, proven on **both** thread and process tiers. Copy its shape.
6. `wat-scripts/fanout/circuit.wat`, `:fanout::worker`'s `:durable` block — `disrupt-hits`,
   `disrupt-draws`, `disrupt-points`, and the `check-exhausted`/`mark-exhausted`/`ack-retries`/
   `ack-exhausted` counters. These are what must arrive at stop.
7. `wat-scripts/fanout/circuit.wat`, `:fanout::Outcome` — what `collect-stop` already receives per worker,
   so you can see what the projection must be widened to carry.

**Sketch:**

```
:fanout::worker gains  :stop (fn [s] -> <a record carrying the outcomes AND the counters>)
collect-stop            reads both from the one reply
sum-disrupts            DELETED — and with it 12 crossings and 12 poll-waits
```

**Blast radius, as a property:** `wat-scripts/fanout/circuit.wat`. **The compiler is the census** — the
summary reads `dhits`/`ce`/`me`/`ars`/`aes` and the phase line prints them, so every consumer will name
itself. Anything forced elsewhere, make and name.

**STOP-1 — if the `:stop` projection cannot carry both the outcomes and the counters, STOP and report
what blocks it.** ⚠ Note the recorded trap: **`Tuple` has no fourth accessor** (`wat/core.wat:1737`, hit
twice in this campaign). If you need more than three values out, **use a record, not a wider tuple.**

**STOP-2 — if `collect` is not the only caller of `sum-disrupts`, STOP and report the other callers.**
The DESIGN's contract decision assumes it is. **Verify from the code, not from my sentence.**

**STOP-3 — if `collect` does not fall materially, STOP and report the numbers.** The premise is that
two-thirds of `collect` is per-worker round-trip latency at ~338 ms per worker at the shipped 250 ms poll.
Removing one of two questions should remove about half of that. If it does not, my attribution is wrong
and that is the finding.

**STOP-4** — on any red in the floor or corpus gate: do **not** re-run. Capture whole, name the exact arm,
surface it.

⛔ **Do not print from inside a service handler.** A `println` in a forked service's handler **corrupts the
frame stream** — measured today: `defservice stop: expected Status::Stopped`. It is documented nowhere,
and it is why an earlier drift raise was invisible. If you need a value out of a worker, it rides a reply.

**Measure**: `2000 4 3 8192 true 1000` and `20 4 3 8192 true 1000`, **interleaved, ≥3 pairs**, box quiet.
⚠ `collect` has been measured at **±20 %** spread, so three per side is the minimum. Report every phase
plus `rt-worker` and `rt-total`.

**Run everything heavy through `./scripts/capped.sh`** (`--limit 8g`). `.wat` is edited with an editor —
**not python or sed**. **Read the floor's Summary line, never a piped exit code.** Leave everything
uncommitted.

**Write your SCORE** to `docs/excursus/2026/08/001-sns-sqs/one-question-per-worker/SCORE.md`, graded row by
row. Copy the shape of `docs/excursus/2026/08/001-sns-sqs/one-sample-per-boundary/SCORE.md`.
