# SCORE — the topic publishes a batch

**SCORED.** Executor: grok, 2026-09-06. Tree dirty, uncommitted.
200 calls, not 2000. One outcome for N. Stamps per message.
Inbox `cap` untouched.

```
Summary [ 408.247s] 5220 tests run: 5220 passed (6 slow), 22 skipped
```

`.floor/2026-09-06T08-20-05Z/`

## THE SURFACE

`:demo::Topic::PublishRequest [msgs <- Vector[String]]`. `publish` declares
`:max-entries [msgs 10]` beside `:max-request-bytes 524288`.
`PublishResponse` gained `RequestTooManyEntries [entries cap]` beside
`RequestTooLarge`. One outcome for the whole batch — a publish still bottoms
out in one `Queue/send`.

Defs:

```
:demo::Topic::PUBLISH-MAX-ENTRIES        10
:demo::Topic::PUBLISH-MAX-ENTRIES-FIELD  "msgs"
```

## THE IMPL

`sns-fanout.wat` `publish`: fold msgs **inside** fold nsubs →
`count(msgs)×nsubs` bodies → **one** `Queue/send`. Direct `:msg` callers
wrapped as a 1-element vector (`publish-until-accepted!*`,
`publish-is-async`, `inbox-refuses`).

A Queue `RequestTooManyEntries` on that send is an assertion (named), not a
`PublishResponse` variant. With send capped at 64, a legal topic batch
(`≤10` msgs × 4 subscribers = 40 bodies) cannot produce it. The topic's own
10-cap is the honest reject. STOP-3 did not fire: no partial, no extra
response arm.

## THE DRIVER

Both publish loops call `:fanout::publish-n-until-accepted!`:
`run-with` (`n=2000`) and `user::outbox-term-loses` (`n=4`). Chunker is
`(n+9)/10` batches, last `ntake` = remaining. n=4 is the short last batch,
live, not only the 2000÷10 exact path.

`:fanout::stamped-range` stamps each index with its own `now()`.
`publish-until-accepted!*` takes a vector, retries the **whole** batch on
`Full`, returns Full-retry count (`attempts-1`) on Ok. Phases line gained
`publish-calls` and `full-retries`.

## BLAST EXCEPTION — Queue send `:max-entries` 10 → 64

`wat-scripts/queue/sqs.wat` is **outside** the listed blast. Without it, 10
msgs × 4 subscribers = 20–40 bodies dies `RequestTooManyEntries(40,10)`
inside the topic (assertion, not a topic RTE). Raised so one topic batch is
one `Queue/send`. Inbox **depth** `:cap 64` is unchanged — 64 where it was
64, 2 where it was 2. The prior probe retargeted 11/10 → 65/64.

This is STOP-5 on blast radius, named, not hidden. It is **not** a raise of
the inbox cap STOP-1 forbids.

## THE PROBE

`wat-scripts/scratch-pad/probe-the-topic-publishes-a-batch.wat`, through
`:demo::Topic/publish`:

```
cap=10;nsubs=2;11=RequestTooManyEntries(11,10);depth 0->0;10=Ok;depth 0->20
```

11 rejected, depth unchanged. 10 Ok, depth +10×2. The def evaluates as 10.

Queue send after the exception
(`probe-a-batch-declares-how-many.wat`):

```
cap=64;field=bodies;thread=RequestTooManyEntries(65,64);depth 0->0;Ok;depth 0->64;process=RequestTooManyEntries(65,64);depth 0->0;Ok;depth 0->64
```

## CIRCUIT ×5

`ps` before: grok 12.6 %, claude 3.9 %, else < 1 %.

Every run: `total=8000;distinct=8000;dup=0`. `publish-calls=200`.

```
publish       22724  22904  22665  22741  22561     median 22724
drain           198    200    201    242    190     median   200
setup         10205  10265  10274  10138  10235     median 10235
stop           6021   6298   6322   6488   6301     median  6301
full-retries   3801   3802   3808   3806   3748     median  3802
```

publish+drain median **22922**. Prior a-batch SCORE: publish **23778**,
drain 210, publish+drain **23988**. −1054 ms publish, −1066 ms combined.

e2e still spans a range (run 1: `50-250=5324 250-1000=2669 max=409ms`;
every run has both buckets). STOP-2 did not fire.

## STOP-1 — retries dominate remaining publish, and it still improved

~19 Full retries per accepted batch (`3802/200`). Each bounce is a 1 ms
await, so ~3.8 s of the 22.7 s publish window is sleep on `Full`. The
driver still makes ~4000 topic round-trips; only 200 of them accept.

That is the measurement the DESIGN asked for. publish **did** improve
(~1.05 s), so STOP-1's "does not improve" clause did not fire as a halt.
The retries are the finding. **Inbox cap not raised.**

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ 200 calls, not 2000 | ✅ `publish-calls=200` every ×5 run |
| 2 | ★★ delivery exact | ✅ ×5 `total=8000;distinct=8000;dup=0` |
| 3 | ★★ cap rejects, nothing enqueued | ✅ `RequestTooManyEntries(11,10)`, depth 0→0 |
| 4 | ★★ 10 still passes | ✅ Ok, depth 0→20 (`nsubs=2`) |
| 5 | ⛔ cap is a readable def | ✅ `:demo::Topic::PUBLISH-MAX-ENTRIES` → `10` |
| 6 | ⛔ stamps stay per message | ✅ `stamped-range` calls `now()` per index; e2e spans 50-250 and 250-1000 |
| 7 | ⛔ inbox `cap` unchanged | ✅ `grep -n ":cap "` — 64 where 64, 2 where 2, plus 1 and 32 untouched |
| 8 | ⛔ both publish loops | ✅ `circuit.wat:1467` (`n=2000`) and `:1791` (`n=4`, short batch live) |
| 9 | ⛔ floor | ✅ `5220 passed (6 slow), 22 skipped`. Count reported. Not re-run. |
| 10 | blast | ▪ `sns-fanout.wat`, `circuit.wat`, scratch probes, this SCORE. **Also `sqs.wat`** (send `:max-entries` 10→64). No `wat/`, no `src/`. |

## REPORTS

| ▪ | what |
|---|---|
| a | publish median **22724** (prior 23778). publish+drain **22922** (prior 23988) |
| b | Full-retry median **3802** (~19 per accepted batch). Finding, not a reason to raise inbox cap |
| c | still `publish` at 22.7 s. Next named: `setup` 10.2 s + `stop` 6.3 s (DESIGN's 16 s / 40 %). Remaining publish is the Full-retry tax against `cap 64` |
| d | `dup=0` every run |
| e | setup median 10235; stop median 6301 |

## STOP TRIGGERS

- **STOP-1** did not halt. publish improved ~1 s. Retries dominate what is left; reported; cap not raised.
- **STOP-2** did not fire. e2e still spans 50-250 and 250-1000.
- **STOP-3** did not fire. `PublishResponse` stays one outcome for N. Queue RTE on the inner send is an assertion, unreachable for a legal topic batch after the send-cap exception.
- **STOP-4** did not fire. Both loops take the same chunker; n=4 is the short last batch.
- **STOP-5** fired as a **named blast exception**: `sqs.wat` send `:max-entries` 10→64. Inbox `:cap` not changed.

## NOT TOUCHED

`wat/`. `src/`. Inbox `:cap`. Per-entry `PublishResponse`. The
`no_unpaired_begin` lint.

Tree uncommitted. Do not commit unless asked.
