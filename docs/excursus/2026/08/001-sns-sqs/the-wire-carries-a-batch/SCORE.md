# SCORE — the wire carries a batch

**STRUCK, and STOP-5 fired.** Executor: grok, 2026-09-03. Two records carry many. The wire
batches. Throughput did not improve.

```
Summary [ 352.730s] 5185 tests run: 5185 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-03T00-48-29Z/`

## Row 1 — the wire actually batches

```
calls=2;msgs=20;shape=batch
```

N=20, K=10, delay so the outbox fills first. Two deliver calls, not twenty.
`tests/services/probe_async_publish.rs::wire_carries_a_batch` is in the floor. This is the
only row an unbatched one-element vector fails.

## Row 3 — throughput went the wrong way

`8000 / publish-seconds` at cap 16, five runs, against **661/s**:

| run | publish ms | drain ms | e2e max | deliveries/s |
|---|---|---|---|---|
| 1 | 28105 | 28120 | 37449 ms | **285** |
| 2 | 28392 | 26438 | 36255 ms | **282** |
| 3 | 27244 | 27268 | 35822 ms | **294** |
| 4 | 28327 | 26097 | 36511 ms | **282** |
| 5 | 28935 | 31083 | 41551 ms | **276** |

Median **282/s**. 0.43×, not 3–5×. STOP-5: the chain was not the bound. Did not tune toward
the estimate.

Every run: `total=8000; distinct=8000; dup=0`. topic-ticks=203–207 (still ~N/10). No linger
timer (`git diff` has zero new `:after`).

## Row 4 — e2e max is 37 s, not 200 ms

At cap 16 the FINDING had e2e max 184–202 ms and `t3→t4 >1 s` count **0**. After wire-batching:

```
t3->t4   >1000 = 5619–7370    max 35–41 s
e2e      >1000 = 5687–7381    max 36–42 s
outbox   >1000 = 0            max 338–413 ms
```

The topic cap is still holding (outbox max ~400 ms). The reservoir moved into the **queues**.
A batcher that improves throughput by re-accumulating a reservoir has rebuilt the thing this
arc just removed — and this one did not even improve throughput. Row 4 catches it.

`t1`/`t2`/`t3` are per-batch (one now for the k-vector). t1→t2 still ~all <1 ms. t2→t3 is the
ten-row put, max 129–174 ms.

## What landed

- `Sub::DeliverRequest` `{msgs}` and `Queue::SendRequest` `{bodies}`. `DeliverResponse::Ok`
  carries a count, not an echoed body.
- `-deliver`: one round of four-send/four-recv carrying k = min(10, length). No linger.
- `send`: N `StoredRow`s, **one** `Store::PutRequest`. `pending += N`. Waiter foldl once.
  isk staggered 1 ns per row (equal isk makes scan-index `:limit` unspecified — the file
  header already said so).
- Counting subscriber + floor test.
- Circuit default cap **16** (the FINDING's backpressured system).
- Scratch-pad call sites that load the new surfaces.

`wat/`, `src/` empty.

Tried and reverted as not the cause: fair-share `take` across waiters (one waiter eating the
batch of 10). Same 28 s / 37 s after. isk stagger kept as correctness, not as a throughput fix.

## cap / K sweep (row 7), K=10

| cap | publish | drain | e2e max | outbox max | /s |
|---|---|---|---|---|---|
| **16** | 28.1 s | 28.1 s | 37 s | 0.4 s | **285** |
| **64** | 26.6 s | 28.1 s | 37 s | 1.6 s | **301** |
| **256** | 21.6 s | 37.9 s | 45 s | 6.5 s | **370** |

No knee recovers 661/s. Deeper cap moves delay into the outbox; `t3→t4` stays a 35–40 s
pileup at every cap. The queues are unbounded and the batch dumps into them faster than
the workers drain.

## EXPECTATIONS

| # | what | this strike |
|---|---|---|
| 1 | ★ deliver calls ≈ N/K | ✅ `calls=2;msgs=20;shape=batch` in the floor |
| 2 | ★ nothing lost | ✅ `dup=0` all five + sweep |
| 3 | ★ throughput vs 661/s | **282/s median, 0.43×.** STOP-5. Chain was not the bound |
| 4 | ★ e2e max ~200 ms | **37 s.** Reservoir is now the queues |
| 5 | one store put per batch | ✅ N rows, one `PutRequest`; waiter foldl once |
| 6 | tail batch min(K, length) | ✅ same bound as tick-batching; 8000/8000 |
| 7 | cap/K sweep | ✅ 16 / 64 / 256 above. No recovering knee |
| 8 | no timer | ✅ zero new `:after` |
| 9 | trace still parses | ✅ five stages; t1/t2/t3 per-batch, stated |
| 10 | no substrate | ✅ `wat/`, `src/` empty |
| 11 | floor | ✅ 5185/5185, `FLOOR=0`, `.floor/2026-09-03T00-48-29Z/` |

## The estimate was the first honest one, and it was still wrong

3–5× assumed the chain was the remaining bound. Putting ten messages in a send made the
queue a 35 s FIFO. Hop count fell; so did throughput. The next lever is not more batching.
