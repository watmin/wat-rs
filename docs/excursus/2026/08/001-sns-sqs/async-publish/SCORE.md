# SCORE — accept, then fan out

**STRUCK.** Executor: grok, 2026-09-02. Every row re-run by me.

```
Summary [ 361.517s] 5184 tests run: 5184 passed (4 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-02T00-39-45Z/`

The circuit, my own run:

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=10;empty=1
setup=8884;publish=4677;drain=74725;stop=2396
WALL_SEC=91.452
```

| # | what | my re-run |
|---|---|---|
| 1 | ★ publish returns before delivery | ✅ `prompt=yes` against a 200ms subscriber |
| 2 | ★ nothing is lost | ✅ `total=8000; distinct=8000; dup=0` |
| 3 | ★ the outbox term is load-bearing | ✅ `:user::outbox-term-loses` → `lost=yes`. Drain on queues only, with a 500ms topic delay, loses the accepted-but-undelivered messages. |
| 4 | refusal, not drop | ✅ `a=ok;b=ok;c=full` at cap 2 |
| 5 | idle topic never ticks | ✅ `ticks=0` |
| 6 | outbox depth is observable | ✅ `stats` reports `outbox` + `ticks`; circuit drain reads it |
| 7 | no substrate change | ✅ `git diff wat/ src/` empty |
| 8 | the phase split | ✅ **reported**: setup 8.9 s, **publish 4.7 s** (was 24.3 s), **drain 74.7 s** (was 0.02 s), stop 2.4 s |
| 9 | wall time | ✅ **91.5 s** against 35.7 s — reported, not promised |
| 10 | floor | ✅ 5184/5184, my own run |

## The shape that landed

```
publish  → append to the outbox, reply Ok (accepted), arm -deliver on empty→non-empty
-deliver → take the head, fan out to subscribers, re-arm while non-empty
Full     → distinct refusal when count >= cap; the caller retries (backpressure)
```

`Ok` no longer carries a delivered count — it has not delivered yet. `stats` exposes
outbox depth. The circuit drain is `pending=0 AND in-flight=0 AND outbox=0`.

Timer delay is 1 µs (Record `delay-ns 1000`). Duration 0 is still silence at process
tier; that substrate finding is untouched.

## Row 3 earned the third term

Same discipline as the sane circuit's in-flight row. A dedicated fixture publishes
into a topic whose `-deliver` is delayed 500 ms, drains on **queues only**, Stops.
`distinct < n`. Without that, a lucky run (outbox already empty when the queues
look drained) would pass and an unlucky one months later would not.

## The work moved, it did not vanish

Publish is the hop it was supposed to become: **4.7 s vs 24.3 s**. Drain absorbed
the fan-out that used to live inside `publish` — 74.7 s of one-at-a-time `-deliver`
ticks, each still doing four adapter/queue/store round-trips on the topic actor,
plus a non-zero timer between them. Wall 91.5 s against 35.7 s is that relocation,
not a lost invariant. The consumers were never the problem; they still are not.
The publisher no longer waits. The topic now does, on its own tick.

A dropped publish would have made this number look better and destroyed the proof.
Refusal + retry is what a real broker does; the bound shipped with the change.

## What did not change

The summary string. `probe_ex001_fanout.rs` unedited. `wat/` and `src/` empty.
The sane circuit's pending-only row still fails when in-flight is removed.
