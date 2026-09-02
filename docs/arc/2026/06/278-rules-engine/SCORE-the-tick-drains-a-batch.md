# SCORE — the tick drains a batch

**STRUCK.** Executor: grok, 2026-09-02. One arm, one file, K=10, rebuild outside the loop.

```
Summary [ 340.974s] 5184 tests run: 5184 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-02T09-53-55Z/`

## Row 1 — the topic actually batches

```
topic-ticks=200
```

Against 2000. Exactly N/10 at K=10. Drain-until-empty would be ticks≈1. STOP-4 does not fire.

## What landed

`wat-scripts/topic/sns-fanout.wat` `-deliver` only:

- `k = min(length, 10)` — the worker's existing `:limit 10`, not a second tunable.
- k rounds of the existing concurrent fan-out (four sends, then four recvs). Unchanged shape.
- **one** rebuild after that loop, dropping k at once (`get box (+ i k)` over `range 0 (nbox-k)`).
- every `vector::get` still `Option/expect` with a located message. The bound is `min(K, length)`; the `Option` is the crash backstop, not the guard.
- `arm-deliver` untouched. It already re-arms from state when the outbox is still non-empty.

`circuit.wat`, `sqs.wat`, `wat/`, `src/` empty. Temporary `:user::main` N swaps for 500/1000 were reverted; landed circuit diff is empty.

## Per-delivery (row 3) — flattened, did not vanish

`per-delivery = drain_ms / (N × M)`. Row 3 is the acceptance criterion, not row 1.

| N×M×J | drain before | drain after | per-delivery | was |
|---|---|---|---|---|
| 500×4×3 | 3501 | 2697 | **1.35 ms** | 1.75 |
| 1000×4×3 | 9268 | 5784 | **1.45 ms** | 2.32 |
| 2000×4×3 | 19205 | 12472 | **1.56 ms** | 2.40 |

Slope 1.35 → 1.45 → 1.56 ms (**+0.21** over 4× N). Was +0.65. Predicted ≲ +0.1 if the rebuild was the only superlinear term.

The expensive half **did** move: row 1 would pass with the rebuild still inside the loop, and that would have left the +0.65. We have a third of that. Drain saved 6.7 s at weight, matching ~90% of the ~4 ms/message that was per-tick cost wearing per-message clothes.

The leftover **+0.21 against ≲ +0.1** is real and monotonic (not noise). Rebuild-once-per-tick of the tail is still O(remaining) per arm, which predicts ~+0.065 after ÷10 — so something else is still superlinear, smaller than the term this stone amortised. Named, not papered. Cursor still not taken; re-measure, then rule.

## Drain (row 7) — it dropped

**12.5 s** against 19.2 s. Reported, not promised.

## The circuit at weight

```
queue-receive-calls=2234
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1
setup=8275;publish=786;drain=12472;stop=4094;qticks=696;topic-ticks=200
WALL_SEC=26.226
```

Invariant holds. `stop=4094` against 4325 — same range, not growing with N as a drain-until-empty tick would on drain (and ticks=200, not 1, is the bound holding). 500/1000/2000 `stop=` was 2404 / 2727 / 4094.

Receive calls 2234 against ~7,100. Park and wakeup untouched. Ten deliveries in one arm burst the four queues hard enough that `:limit 10` actually fills (8000/2234 ≈ 3.6 msgs/receive, was ≈ 1.1). Side effect of tick-batching, not a second change.

## EXPECTATIONS

| # | what | this strike |
|---|---|---|
| 1 | ★ topic-ticks ~N/10 | ✅ **200** against 2000 |
| 2 | ★ nothing lost | ✅ `total=8000; distinct=8000; dup=0` |
| 3 | ★ per-delivery slope nearly vanishes | 1.35 / 1.45 / 1.56 ms, **+0.21** against +0.65 (bar ≲+0.1). Flattened 3×, did not vanish. Remaining superlinear term exists |
| 4 | ★ interruptible | ✅ circuit completes; `stop=4.1 s` against 4.3 s; ticks=200 not 1 |
| 5 | one rebuild per tick, dropping K | ✅ rebuild is after the k-loop, `get box (+ i k)` |
| 6 | fan-out still concurrent | ✅ `dt-ms=204;shape=max`; `fanout_is_max_not_sum` passes |
| 7 | drain drops | ✅ **12.5 s** against 19.2 s |
| 8 | no surface change | ✅ `:demo::Sub` / `:demo::Topic` messages and features unchanged |
| 9 | blast radius | ✅ `sns-fanout.wat` only. circuit/sqs/wat/src empty |
| 10 | receive calls | 2234 against ~7,100 — burst-batching at the worker limit, reported |
| 11 | wall time | **26.2 s** against 33.3 s — reported |
| 12 | floor | ✅ 5184/5184, `FLOOR=0`, `.floor/2026-09-02T09-53-55Z/` |

## Wire-batching still cut

Tick-batching never needed a surface. Wire-batching still does: `Sub::DeliverRequest` and `Queue::SendRequest` both carry many. Size it now that a tick already handles ten.

## Cursor still deferred

This stone amortised the term a cursor would remove. Leftover slope +0.21 is the number to re-measure against, not the pre-batch 20%.
