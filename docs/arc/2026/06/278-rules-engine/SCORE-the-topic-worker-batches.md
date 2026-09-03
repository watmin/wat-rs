# SCORE — the topic-worker batches

**STRUCK.** Executor: grok, 2026-09-03. One tick of ≤10 inbox rows is grouped by
subscriber: one `Queue/send` of that subscriber's bodies, then ack the batch.
Order within a subscriber is receive order. Full still skips the whole
subscriber-batch (do not ack).

```
Summary [ 354.930s] 5190 tests run: 5190 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-03T06-21-07Z/`

## Five runs, mem, `2000×4×3`

All `total=8000; distinct=8000; dup=0`. STOP-distinct did not fire.

| run | publish+drain | e2e max | deliveries/s |
|---|---|---|---|
| 1 | 42.417 s | 5710 ms | **189** |
| 2 | 43.236 s | 5762 ms | **185** |
| 3 | 43.466 s | 5785 ms | **184** |
| 4 | 42.103 s | 5687 ms | **190** |
| 5 | 41.511 s | 5615 ms | **193** |

Median **189/s** against S13's 148–153/s (**1.25×**). Round trips collapsed
(queue-receive-calls 8012 → ~6300). Per-message CPU did not: uuid/edn/row
build still per row, as predicted.

e2e max **5.6–5.8 s** against S13's 712–743 ms. `t2→t3` grows a >1 s tail
(15–26 messages). Same shape the wire-batching stone named on mem: a batched
put is one O(table) write of N rows. Throughput up, latency out. Reported,
not chased.

Refusal and stalled gates still pass — grouping does not fuse subscribers.

## What landed

`wat-scripts/topic/sns-fanout.wat` `-tick` only: bucket by `idx`, one send
per non-empty bucket, ack every id in the bucket on Ok, none on Full.
`circuit.wat`, `sqs.wat`, `wat/`, `src/` empty of this stone (S13 already in
circuit).
