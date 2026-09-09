# EXPECTATIONS — the wire carries a batch

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ the wire actually batches | a counting subscriber, wired into the floor like `fanout_is_max_not_sum` | **`deliver` calls ≈ N/K, not N.** Everything else passes on an unbatched implementation; this is the only row that cannot |
| 2 | ★ nothing is lost | the circuit at `2000×4×3`, **five runs** | `total=8000; distinct=8000; dup=0` **every time** (STOP-1) |
| 3 | ★ throughput improves | `8000 / publish-seconds` | **reported**, against **661/s**. Decomposed estimate 3–5× (~2000–2600/s) — a measurement, not a target (STOP-5) |
| 4 | ★ latency does NOT regress | the `e2e` histogram at `cap 16` | max stays **~200 ms**. A batcher that improves throughput by re-accumulating a reservoir has rebuilt the thing this arc just removed (STOP-2) |
| 5 | ★ one store put per batch | read `sqs.wat`'s `send` | a single `Store::PutRequest` carrying N rows; the waiter-serving foldl runs **once** per call, not per message |
| 6 | the tail batch is handled | the last messages of a run | `min(K, length)`; nothing lost when the outbox holds fewer than K (STOP-4) |
| 7 | the `cap`/`K` knee | `cap ∈ {16, 64, 256}` at `K=10` | throughput **and** e2e max reported for each. The knee is the deliverable, not a single pair |
| 8 | no timer was added | `git diff` | **zero** new `Alarm`/`after` in the batching path (STOP-2) |
| 9 | the trace still parses | the histogram lines | all five stages still reported. `t2`/`t3` are now per-batch — say so, since messages in one batch share a stamp |
| 10 | no substrate change | `git diff --stat wat/ src/` | **empty** (STOP-3) |
| 11 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, `FLOOR=0`, ≥5185 tests (row 1 adds one) |

**Runtime prediction:** 2–4 hours. The shape is mechanical; the tail batch and the three-way sweep
are the work, and two surfaces moving means every call site must be found.

## Trap doors, named in advance

- **Row 1 is the only row an unbatched implementation fails.** Rows 2–11 all pass if `msgs` carries
  a one-element vector every time. A counting subscriber is the only proof, and it belongs in the
  floor, not in a one-off run.
- **Row 4 is the one that catches a "successful" regression.** Throughput is trivially improvable by
  letting the outbox refill — which is the reservoir, rebuilt. Latency is what says whether the win
  is real or borrowed.
- **The tail batch is where batched pipelines lose messages**, and it will pass at `N=2000` while
  failing at some other N. Row 6 plus five runs is the guard; a single green run is not.
- **`t2`/`t3` become per-batch stamps.** Messages in one batch share them, so those two intervals
  stop being per-message truths. Row 9 asks for that to be *stated*, not silently reinterpreted —
  the trace is now our primary instrument and a misread stage is worse than a missing one.
- **Firing on nothing:** the estimate is 3–5×. If it comes back 1.2×, the per-message CPU dominates
  the chain and the conclusion is that the chain was never the bound. **Report it; do not tune
  toward the estimate.**
