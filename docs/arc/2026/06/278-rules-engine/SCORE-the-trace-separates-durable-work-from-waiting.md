# SCORE — the trace separates durable work from waiting

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
`sns-fanout.wat` + `circuit.wat`. Telemetry only. Delay not pinned.

```
Summary [ 476.144s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-07T05-35-47Z/`

## THE SHAPE

Body is `msg|t0|t0b|t1|t3|t3b|t4` (7 parts, not 8 — BRIEF added two stamps
and forgot to subtract the phantom). `t2` is deleted. `hop12` is gone.

```
pub-work     t0  → t0b
inbox-wait   t0b → t1
worker-proc  t1  → t3     (was hop23; still free)
fanout-work  t3  → t3b
subq-wait    t3b → t4
e2e          t0  → t4
```

## STOP-2 — the return instant cannot ride in the payload

A timestamp taken *after* `send-all` / `Queue/send` returns is after persist.
The body is already in the store. Putting that instant in the payload is
physically the next write.

`t0b` is inside `Topic::publish`, last instant **before** `send-all`.
`t3b` is inside the topic-worker, last instant **before** subscriber
`Queue/send`. Not on the publisher — that would fold the reply hop into
"durable work". Gate 3's *location* holds; the *after* is the shape
STOP-2 asked to report.

Store/put of that send therefore sits in the following wait stage.
At ms resolution (hist buckets, `ns->ms` truncates) a 675 µs write is 0 ms
either side of the stamp.

## CIRCUIT ×3, shipped, unpinned

`ps` before: grok 11.3 %, claude 4.8 %.

Every run: `total=8000;distinct=8000;dup=0`.

```
                 r1     r2     r3    med    today
publish       20542  20709  20811  20709   ~20700
collect        5974   6064   6018   6018    ~5900
stop            351    241    605    351     ~400
drain           180    195    136    180
```

**STOP-1 did not fire.** publish +9 of 20700.

## THE FIVE STAGES (r2, buckets sum to 8000)

```
pub-work <1ms=40   10-50=280  50-250=7680              max=129ms
inbox    <1ms=0    10-50=3586 50-250=4414              max=141ms
worker   <1ms=7997 1-10=3                              max=2ms
fanout   <1ms=1997 1-10=333  10-50=5106 50-250=564     max=88ms
subq     <1ms=0    1-10=1825 10-50=6166 50-250=9       max=61ms
e2e      <1ms=0    10-50=7   50-250=7464 250-1000=529  max=329ms
```

Neither new stage is all-zero. **Not a second phantom.** worker-proc
unchanged (free). e2e still 50-250 dominant with a 250-1000 tail.

Old `outbox` 50-250=7989 split across pub-work + inbox-wait (a message's
two halves land in different buckets; they do not add per-bucket).
Old `t3->t4` 10-50/50-250 split across fanout-work + subq-wait.

## ★ THE NUMBER — durable work as a fraction of each hop

Sample seq=0 (first message, no retry; three runs agree):

```
         pub-work  inbox-wait   fanout  subq     e2e
r1         0.44ms      10.5ms   0.08ms  5.8ms   16.8ms
r2         0.43ms      10.9ms   0.08ms  5.2ms   16.6ms
r3         0.42ms      10.2ms   0.12ms  5.7ms   16.5ms
```

For a message that did not bounce: **inbox-wait is ~96 % of t0→t1**,
**subq-wait is ~99 % of t3→t4**. The pre-send "work" stamp is sub-millisecond.
Store/put at 675 µs is invisible at this resolution. DESIGN's prediction
holds for the per-message path: the hop is wait, not write.

Histogram mass in pub-work 50-250 is **retries keeping t0** (asleep), not
Store/put. Histogram mass in fanout-work 10-50 is **sibling sends in the
same -tick** (t3 is per-envelope; t3b is per-bucket, after earlier
subscribers' RPCs). Neither is the durable write. The write of *this*
message sits in inbox-wait / subq-wait and is 0 ms of them.

The system is queue-bound (and retry-bound on the publish path). Effort
does not belong on how fast a write is.

## BODY LENGTH

Sample 121 bytes (`seq` 1 digit + six 19-digit stamps). Worst `seq=1999`
is 124. `Queue::send` cap 524288. Margin **524164 bytes**. Queue
`:max-frame-bytes` 8192. STOP-3 did not fire.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ phantom gone | ✅ `grep ":t2 t1"` empty; `hop12` removed |
| 2 | ★★ split is real | ✅ both new stages have mass; worker free; see histograms |
| 3 | ★★ t0b stamped by the topic | ✅ inside `publish`, not the publisher. After-return is STOP-2 |
| 4 | ⛔⛔ measuring does not perturb | ✅ publish 20709 vs 20700; collect 6018; stop 351 |
| 5 | ⛔ delivery exact | ✅ ×3 `distinct=8000;dup=0` |
| 6 | ⛔ e2e spans t0→t4 | ✅ 50-250 dominant, 250-1000 tail |
| 7 | ⛔ the floor | ✅ `5221 passed (7 slow), 22 skipped` |
| 8 | ⛔ blast | ✅ `sns-fanout.wat` + `circuit.wat` + SCORE. No `wat/`, no `src/`, no `sqs.wat` |

**STOP-4 did not fire.**
