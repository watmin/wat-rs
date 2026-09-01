# SCORE — item (c) stone D: the bounded buffer

**STRUCK.** Executor: grok, 2026-09-01. Every row re-run by me. One weigh.

```
Summary [ 336.908s] 5161 tests run: 5161 passed (3 slow), 15 skipped
FLOOR=0
```

5161 = 5155 + six `probe_arc278_span_bounded` tests. **All twelve rows pass, no deltas.**

| # | what | result |
|---|---|---|
| 1 | ★ the bound holds | ✅ `bound_holds` — **RED this morning** |
| 2 | ★ the drop count is exact | ✅ `drop_count_is_exact` (10 logs, max 3 → `:logs-dropped` = 7) |
| 3 | the caller is told | ✅ `overflowing_log_returns_dropped_not_ok` |
| 4 | the OLDEST go | ✅ `oldest_logs_are_dropped` — the survivors are the last three |
| 5 | samples too | ✅ two gates |
| 6 | counters untouched | ✅ no bound on `counters` |
| 7 | the producer never blocks | ✅ |
| 8 | the counter survives the condition | ✅ in `counters`, `O(1)` per key |
| 9 | under the bound, unchanged | ✅ |
| 10 | prior gates hold, assertions unedited | ✅ **verified line by line** — see below |
| 11 | no new surface op, no runtime change | ✅ Span 5; `src/runtime.rs` empty |
| 12 | floor | ✅ 5161/5161, my own run |

## Row 10, checked rather than taken

Five prior gate files changed, which is exactly the shape stone B taught me not to accept on a
count. Every altered line, with the two new Record fields filtered out:

- `:metrics-flush-after-ms N)` → `:metrics-flush-after-ms N` — a closing paren moving because two
  more fields now follow it. Mechanical.
- two new match arms: `((LogResponse::Dropped _buffered _cap) 6)` and the `TimedResponse` twin.

The second is the **cascade** — a new variant makes existing matches non-exhaustive, so those
fixtures must gain arms. And they map to a **distinct sentinel** (`6`), not folded into an existing
value and not wildcarded: if a drop ever occurred in those tests they would report it rather than
pass quietly. **No assertion weakened.** This is the row-8 lesson from stone B applied correctly:
"assertions unedited", not "empty diff".

## The chain is closed

```
log/incr/timed   → buffer, arm on empty→non-empty        (stones A, B)
buffer full      → drop OLDEST, count it, say Dropped     (stone D)
size cap         → flush, and SPEAK if it fails           (stone C)
over-cap buffer  → fragment into ≤cap submissions         (item b)
failed chunk     → keep exactly the un-written suffix     (item b)
```

Every path either lands the data or **names what it lost, on a channel that survives the loss**.
Nothing in the chain is silent, and nothing grows without bound.

## The ruling that opened at stone A is now closed

`LogResponse::Ok` still means **accepted** — and it is now the truth rather than a hedge, because the
three ways it could have been a lie all have their own answer: a failed flush **speaks** (stone C), a
dropped record says **`Dropped`** (this stone), and an over-cap buffer **drains** (item b).

Backpressure needed no work: every hop is blocking request/reply, and the span's serializing loop
means a slow sink transitively slows its producers while it flushes. The only unbackpressured window
is accumulation between flushes, which is what a buffer is for.

## What remains, by ruling rather than omission

- **Item (a)** — `write-*-stream` over a lazy `Stream`. Unchanged: it waits for a consumer that
  actually streams. Nothing in the tree does.
- **`NOTE-impls-completeness-is-unenforced.md`** — a `defservice` may `:satisfies` a surface and not
  implement all of it. Found grading stone A, still unbuilt, and the sibling of excursus 001's
  `:messages` guard.
