# SCORE — the consumer is idempotent

**STRUCK.** Executor: grok, 2026-09-03. Identity is the published `seq` (first
field of the body). Workers claim it on a shared seen-service; a second
delivery is acked and dropped. `distinct` counts `(queue, seq)`, not
`(queue, envelope-id)`.

```
Summary [ 358.329s] 5190 tests run: 5190 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-03T06-07-14Z/`

## Why body-key was never wired

`:fanout::body-key` keyed on the whole body, timestamps included. A
redelivery would still look distinct. The instrument existed and pointed
at the unstable suffix. `key-of` now uses `seq-of` — the prefix the trace
placed first for this.

A receive-redelivery of an *unacked* row is the same envelope id (the
visibility probe). The duplicate the floor red actually produced — and the
one envelope-id cannot see — is **two SENDs of the same seq**, each minting
a new uuid. Row 1 forces that.

## Rows 1–3

```
total=2;distinct=1;dup=1;same-seq=yes;envelopes-differ=yes   ;; dedupe off
total=1;distinct=1;dup=0                                     ;; dedupe on
n=4;distinct=<4;lost=yes                                     ;; pending-only, unchanged
```

## Five runs, mem, `2000×4×3`

All `total=8000; distinct=8000; dup=0`. Circuit vis windows **unchanged**
(topic-worker 5 s, circuit worker 10^12 ns).

| run | publish+drain | e2e max | deliveries/s |
|---|---|---|---|
| 1 | 52.270 s | 743 ms | **153** |
| 2 | 53.942 s | 743 ms | **148** |
| 3 | 53.217 s | 726 ms | **150** |
| 4 | 53.142 s | 737 ms | **151** |
| 5 | 54.064 s | 712 ms | **148** |

Median **150/s** against 149–161/s. The seen-service is one extra RPC per
delivery; reported, not chased.

## What landed

- `:fanout::seen` — one process, `claim` → First/Dup. Workers dial it.
- Worker claims before ack; Dup acks and does not record an outcome.
- `key-of` = `queue/seq`. `sqs.wat`, `sns-fanout.wat`, `wat/`, `src/` empty.

## EXPECTATIONS

| # | what | this strike |
|---|---|---|
| 1 | ★ detector sees a message duplicate | ✅ two sends of seq 7: `dup=1`, envelopes differ, seq same |
| 2 | ★ consumer absorbs it | ✅ `total=1;distinct=1;dup=0` |
| 3 | ★ loss still detected | ✅ pending-only `lost=yes`, distinct < n |
| 4 | ★ identity from the publisher | ✅ seq, the body's first field, not envelope `sk` |
| 5 | dedupe in the consumer only | ✅ `git diff sqs.wat` empty |
| 6 | nothing lost at weight | ✅ `total=8000; distinct=8000` five times |
| 7 | window not widened | ✅ run* vis 5 s / 10^12 unchanged; 200 ms only on the probe |
| 8 | throughput | ✅ **148–153/s** against 149–161/s |
| 9 | no substrate | ✅ `wat/`, `src/` empty |
| 10 | floor | ✅ 5190/5190, `FLOOR=0`, `.floor/2026-09-03T06-07-14Z/` |
