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

> ⛔ **VERIFIED FALSE 2026-09-02.** `wat-scripts/scratch-pad/probe-parked-waiters-stop.wat`: one
> queue and J parker services, all at **process** locus, each parked inside a `Queue/receive` with
> `wait-ns > 0` on a permanently empty queue, then `Admin::Stop`. J = 1, 2, 3, 4, 5, 8.
> **Every J stops cleanly.** There is no ≥4 threshold.
>
> One variable — the park duration:
>
> ```
> park = 5 s     j=1: 5990 ms   j=4: 6991 ms   j=8: 8336 ms
> park = 50 ms   j=1: 1275 ms   j=4: 2399 ms   j=8: 3580 ms
> ```
>
> Step time tracks **`wait-ns`**, not waiter count. At j=1 the difference is 4715 ms — the 5 s park
> minus the 250 ms settle, exactly. Growth with J is ~350 ms per parker, which is process spawn.
>
> **What actually happens:** `Stop` waits for the in-flight parked receive to return, because a
> `defservice` is a serializing actor and the arm must finish before the serve loop can take
> `Admin::Stop`. That is correct, it is **bounded by `wait-ns`**, and it does not depend on how many
> waiters exist. A long `wait-ns` makes shutdown slow; it never makes it hang. Read as a hang, it
> produced a workaround — workers polling at `wait-ns 0`, re-arming every 1 ms — that generates
> **144,485 receive calls to deliver 8,000 messages, 94% of them empty.**

★ ~~Both findings share a shape worth naming: **a capability that works at thread tier and silently
does not at process tier.**~~ **AMENDED 2026-09-02: only finding (1) has that shape.** Finding (2)
was verified false (above) and is not a locus-transparency failure at all — `Admin::Stop` behaves
identically at both loci and is simply bounded by `wait-ns`.

The generalisation was drawn from two data points while one of them was explicitly unverified, and
it read as a pattern for a day. **One confirmed instance and one unconfirmed report is not a shape**
— finding (1) stands on its own evidence and needs no second case to be worth fixing.

## What this unblocks

The capstone now has a fixture that can measure: 35 s a run, deterministic, and a program whose shape
a reader would recognise — consumers that consume, a producer alongside them, and a shutdown that is
a signal rather than a count.
