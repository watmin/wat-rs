# SCORE — the sane circuit

**STRUCK.** Executor: grok, 2026-09-01. Every row re-run by me.

```
Summary [ 364.890s] 5180 tests run: 5180 passed (4 slow), 15 skipped
FLOOR=0
```

| # | what | my re-run |
|---|---|---|
| 1 | ★ the invariant holds | ✅ `total=8000; distinct=8000; dup=0`, both runs |
| 2 | ★ nothing lost at shutdown | ✅ **and the term is proven load-bearing** — see below |
| 3 | `stats` reports depth | ✅ `pending` + `in-flight` |
| 4 | no fixed iteration counts | ✅ `range 0 cap` gone, zero hits |
| 5 | the worker is interruptible | ✅ `Admin::Stop` taken between ticks |
| 6 | producer and consumers overlap | ✅ workers start first |
| 7 | empty polls gone | ✅ one tick = one receive, `limit 10` |
| 8 | participation healthy | ✅ workers **8** and **10** of 12 — not the 4-of-12 collapse |
| 9 | tallies via `Status::Stopped` | ✅ no invented channel |
| 10 | substrate untouched | ✅ `git diff wat/ src/` empty |
| 11 | the wall time | ✅ **35.9 s and 35.7 s** against 88.6 s |
| 12 | floor | ✅ 5180/5180, my own run |

**2.5× faster, and deterministic** — 35.9 / 35.7 s, against long polling's 104.8 / 181.4 s on the old
fixture. The sane program is both quicker and stable, which is the point: the old one's variance was
its consumers guessing when to stop.

## Row 2 earned its design

The row required the in-flight term be proven by **removing** it and demanding a failure. A same-tick
ack cannot demonstrate it — the window is too small to lose anything. The strike built a
**delayed-ack worker**, drained on `pending = 0` alone, and lost the held message: `lost=yes`.

That is the difference between a condition that is *present* and one that is *load-bearing*, and it
is the kind of check that only works by deliberately breaking the thing.

## ★ Two substrate findings, and both break locus transparency

These are not circuit bugs. They are the substrate behaving differently at the two loci, which is
what the entire IPC-stands-in-for-the-network model rests on **not** happening.

**1. A duration-0 timer never fires at process tier — silently.** I verified this independently:

```
thread  ns=0        -> FIRED
process ns=0        -> TIMED-OUT (500 ms guard)
process ns=1000000  -> FIRED
```

`timerfd_settime` with `it_value = 0` **disarms** rather than firing immediately, and that leaks
straight through `after`. The same program does different things at the two loci, with no error and
no diagnostic. The queue's wake and flush were `Nanosecond 0`; at process locus that was silence.
Clamped to 1 ms as a workaround — **the substrate defect is untouched and wants its own stone.**

**2. Four parked waiters hang `Admin::Stop`.** Reported repro: `1 1 2` stops, `1 1 4` hangs, with
`-tick` blocked inside `Queue/receive`. The circuit routes around it (workers receive at `wait-ns 0`
and re-arm at 1 ms, so Stop is taken in the serve loop) while the row-2 worker still long-polls with
a single waiter.

⚠ **I have NOT independently verified this one** — I could not reconstruct the repro's invocation
from the report. It is recorded as the strike's finding with the strike's numbers, and it should be
confirmed before anything is built on the workaround.

★ Both findings share a shape worth naming: **a capability that works at thread tier and silently
does not at process tier.** Every stone today has treated `:locus` as a parameter. These are the
first two places where that promise measurably fails, and neither announces itself.

## What this unblocks

The capstone now has a fixture that can measure: 35 s a run, deterministic, and a program whose shape
a reader would recognise — consumers that consume, a producer alongside them, and a shutdown that is
a signal rather than a count.
