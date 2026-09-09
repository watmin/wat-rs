# SCORE — the server manages its own capacity

**SCORED.** Executor: grok, 2026-09-06. Tree dirty, uncommitted.
`Accepted [count]` is a prefix. `:Full` is gone. Inbox `:cap` untouched.
The retries did not go down.

```
Summary [ 407.370s] 5220 tests run: 5220 passed (6 slow), 22 skipped
```

`.floor/2026-09-06T09-56-23Z/`

## THE SURFACE

`Queue::SendResponse` and `Topic::PublishResponse`: `:Ok` and `:Full [depth cap]`
collapse into `:Accepted [count]`. Queue send dropped `:max-entries`. Topic
publish keeps `[msgs 10]`.

No response arm is named `:Full`. `RequestTooLarge` still has a `cap` field —
that is the **byte** budget, pre-existing. Topic `StatsResponse::Ok` still
has `depth` — that is the stats question, not admission. Admission no longer
leaks queue depth or queue cap.

## THE ADMISSION

`sqs.wat` send:

```
room  = cap - depth          ;; may be <= 0
take  = min(n0, room)
take  = max(take, 0)
```

`take == 0` → `Accepted 0`, write nothing. `take > 0` → first `take` bodies,
one `Store/put`, `Accepted take`. Lost/Closed/TimedOut on the put →
`Accepted 0` (unknowable; retry unchanged).

Topic: `msgs-accepted = floor(pairs-accepted / nsubs)`. A partial tail
message is not counted. The client re-publishes it. That is at-least-once;
`Seen` is the dedupe.

Topic-worker acks the **prefix** of a subscriber send (`nacc` ids), not
all-or-nothing.

## THE DRIVER

`publish-until-accepted!*` on `Accepted c`:
- `c >= n` → done
- `c == 0` → wait 1 ms, resend all (counts as full-retry)
- else drop first `c`, resend the tail immediately

Both loops (`n=2000`, `n=4`) use it.

## THE PROBE

`wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat`:

```
fill=6;depth6=6;send8=4;depth10=10;stored=f0,f1,f2,f3,f4,f5,m0,m1,m2,m3;send3=0;depth-full=10;drain5-send8=5;depth-after=10;nsubs4-room6=Accepted(1);nsubs7-pub10=Accepted(9)
```

11 of the 8 is a prefix, in order. `Accepted 0` writes nothing. Drain 5 then
send 8 takes 5. `nsubs 4` / room 6 pairs → `Accepted 1`. `nsubs 7` publish 10
against cap 64: **no assertion**, `Accepted 9` (`floor(64/7)`). The nsubs
cliff is gone.

Retargeted `probe-a-batch-declares-how-many.wat` to `Topic/publish`:

```
cap=10;field=msgs;11=RequestTooManyEntries(11,10);depth 0->0;10=Accepted(10);depth 0->20
```

## CIRCUIT ×5

`ps` before: grok 12.1 %, claude 4.2 %, else < 1 %.

Every run: `distinct=8000`. `publish-calls=200`.

```
publish        60778  61084  61126  62508  61920     median 61126
drain            252    275    277    289    247     median   275
setup          10227  10277  10190  10245  10245     median 10245
stop           13285  11690  11477  10175  13033     median 11690
full-retries    8358   8314   8351   8507   8424     median  8358
dup             4066   3993   4021   4016   4159     median  4021
seen-skipped   13609  13563  13457  13819  13594     median 13594
```

publish+drain median **61401**. Prior topic-batch SCORE: publish **22724**,
full-retries **3802**, dup **0**.

e2e still spans 50-250 and 250-1000 (run 1: `50-250=3068 250-1000=8986 max=646ms`).

## THE FINDING — partial admission filled the inbox; the tail 429-spins more

`full-retries` counts `Accepted 0` only. Median **8358**, up from 3802.
publish **61126**, up from 22724.

Immediate tail resend (as the BRIEF wrote) plus taking a prefix fills cap
64. The remaining messages then `Accepted 0` until workers drain. More
1 ms waits, more topic round-trips, than whole-batch `Full` against a
queue that usually had *some* space.

`dup` is live (~4021 outcomes, ~13594 Seen skips). Two workers can both
see `Absent` on a bursty re-publish before `mark`. `distinct=8000` still
holds. Inbox `:cap` was not raised.

## WORKER STOP — 512 KiB was too small for a bursty 2000

First circuit attempt (captured, not re-run as a flake):

```
frame exceeded cap … :fanout::worker/stop
seen-recorded=8000;seen-skipped=13433
```

Stop returns every first-seen `Outcome`. A bursty queue can land all 2000
on one worker; 512 KiB does not hold that vector. Worker spawn in
`run-with` now uses `ProcessOpts` `:max-message-bytes 2097152`. Not an
inbox `:cap`. Named.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ partial admission | ✅ send 8 at depth 6 → `Accepted 4`, depth 10 |
| 2 | ★★ it is a PREFIX | ✅ stored `f0..f5,m0,m1,m2,m3` — first 4 of the 8, in order |
| 3 | ★★ no room is `Accepted 0` | ✅ send 3 at depth 10 → `Accepted 0`, depth unchanged |
| 4 | ★★ room reappears | ✅ drain 5, send 8 → `Accepted 5` |
| 5 | ⛔ no response leaks internals | ✅ `:Full` gone. See note below |
| 6 | ⛔ `Queue::send` has no entry cap | ✅ `grep max-entries sqs.wat` empty. Topic keeps `[msgs 10]` |
| 7 | ⛔ topic reports whole messages | ✅ `nsubs 4`, room 6 pairs → `Accepted 1` |
| 8 | ⛔ nsubs cliff is gone | ✅ `nsubs 7`, publish 10 → `Accepted 9`, service alive |
| 9 | ⛔ delivery still exact | ✅ ×5 `distinct=8000`. `total` is ~12000 because `dup` is live — the prose gate, not the copied `total=8000` cell |
| 10 | ⛔ no `:cap` changed | ✅ 10× 64, 2× 2, 2× 1, 1× 32, 7× 1024 |
| 11 | ⛔ floor | ✅ `5220 passed (6 slow), 22 skipped`. Count reported |
| 12 | blast | ▪ `sqs.wat`, `sns-fanout.wat`, `circuit.wat`, scratch probes, this SCORE. **Also `tests/services/probe_async_publish.rs`** (pinned `a=ok;b=ok;c=full` and Full's depth/cap). No `wat/`, no `src/` |

Row 5 note: `RequestTooLarge [bytes cap]` remains on every response enum
(byte budget). Topic `StatsResponse::Ok [depth ticks]` remains (stats).
Neither is admission leaking queue sizing.

## REPORTS

| ▪ | what |
|---|---|
| a | full-retries median **8358** (prior 3802). The number this stone exists to move went **up** |
| b | publish median **61126** (prior 22724). publish+drain **61401** |
| c | `dup` median **4021**; seen-skipped median **13594**. At-least-once is visible |
| d | still `publish`, now as Accepted-0 spin against cap 64. Next named: `setup` 10.2 s + `stop` 11.7 s — but publish is 61 s |
| e | setup median 10245; stop median 11690 (stop grew: 2 MiB outcome vectors) |

## STOP TRIGGERS

- **STOP-1** did not fire. Rows are built in body order (`range 0 take`, 1 ns isk stagger). The probe read the stored prefix.
- **STOP-2** did not fire. `Accepted 0` is only `take == 0` (clamped room). Lost/Closed/TimedOut also return 0 because the write is unknowable — same 429, same retry.
- **STOP-3** did not fire. Inbox Accepted count is a prefix of the pairs sent (msg-major). `floor` is honest.
- **STOP-4** did not fire. Removing `:max-entries` from send needed no `wat/` / `src/`. Absent means uncapped.
- **STOP-5** fired as **named exceptions**, not inbox `:cap`:
  1. `tests/services/probe_async_publish.rs` — the test pinned `Full` and `depth`/`cap` on the liveness bound.
  2. Worker spawn `:max-message-bytes` 512 KiB → 2 MiB so `worker/stop` can return a 2000-outcome vector.

## NOT TOUCHED

`wat/`. `src/`. Inbox `:cap`. `Store::put` per-entry outcomes. Fanout topology.

Tree uncommitted. Do not commit unless asked.
