# EXPECTATIONS — every tier reports its backlog

Written **before** the strike. Measurement only: the numbers are the deliverable.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **five counters exist and are distinct** | read the diff | `sends-accepted`, `sends-refused`, `acks`, `redeliveries`, `expired-waiters` — never summed, never reusing an existing field |
| 2 | ★ **they live on `:ephemeral`** | read the diff | `:queue::queue::Record` is **not** in the diff; no Record constructor changes anywhere |
| 3 | ★ **the instrument costs no store calls** | the same run | `store-calls` at n=2000 within a few of the pre-stone baseline (9731 mine / 12336 grok's earlier run). A counter increment must not touch the store |
| 4 | ★ **every tier reports** | run n=2000 `vis-ms=1000` | the inbox **and** each of the m subscriber queues, named per tier — not one aggregate |
| 5 | **refusals reconcile** | the same run | `sends-refused` on the inbox accounts for the publisher's `full-retries` (1240 on my last run), **or** the SCORE names the gap |
| 6 | **the absence is recorded, not faked** | the SCORE | it states plainly that **no message-discard path exists**, so no such counter was added |
| 7 | **existing entries unaffected** | no-arg run + each wrapper | reported completeness fields identical to today |
| 8 | **blast radius** | `git diff --stat` | 8 files: `sqs.wat`, `sns-fanout.wat`, `circuit.wat`, and the five scratch-pad probes matching `StatsResponse::Ok` |
| 9 | **the box was quiet** | the SCORE | load stated at the start of the measured run |
| 10 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5235 passed, 22 skipped, **0 FAIL, 0 TIMEOUT** |

## The rows that carry it

★ **Row 3 is the stone's integrity.** An instrument that costs 2 store calls per read would be 39 % of
store traffic if sampled — measured, not guessed. Counters must be free, and row 3 proves they are by
comparing `store-calls` against the baseline. **If `store-calls` jumps, the instrument is now part of
the phenomenon.**

★★ **Row 5 gates my own analysis, again.** I claim the publisher's `full-retries` are inbox refusals.
If `sends-refused` does not account for them, there is another rejection path — the same shape as the
gap STOP-5 already caught once today. **A gap is a finding, not a failure.**

★ **Row 6 exists because the builder asked for a discard metric and there is nothing to count.** Adding
a counter that reads zero forever would look like diligence and be noise. Say the absence.

⚠ **Row 2 is the blast-radius lesson made a gate.** Last stone I changed a Record's fields and five
scratch-pad probes came along unannounced. Keeping the counters `:ephemeral` means the Record is
untouched by construction, not by care.

## Runtime prediction

**60–90 minutes.** Five counters, a `Stats` record, 17 call sites the compiler will name, one report
line. One n=2000 run at ~41 s. The floor is the long pole.

## Trap-doors

- **`sends-refused` is per attempt, not per body.** A refused 40-body batch is one refusal.
- **`acks` must count acknowledged messages, not ack *calls*** — a batch ack is many.
- **`redeliveries` fires when visibility expiry makes a message visible again**, which is not the same
  as a client's `ack-retries`. Two different observers of one event; do not conflate.
- **`expired-waiters` is the `:1001` path** — waiters, never messages.
- **The `Stats` record must be EDN-expressible** — plain `i64` fields only, no peers, no handles.
- **Rates are `count ÷ phase duration`.** Do not add a timer per counter.

## What this stone does NOT claim

⚠ It does **not** remove the cap, restructure the topic, or choose a pool size. It makes all three
judgeable.
⚠ It does **not** add retention, discard, or a dead-letter path.
⚠ It does **not** explain the slope — it makes the next explanation cost one run instead of eleven.
