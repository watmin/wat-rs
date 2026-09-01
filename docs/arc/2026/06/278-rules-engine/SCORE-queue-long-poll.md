# SCORE — long polling in `wat-queue`

**STRUCK, and the capability is sound. The circuit's ADOPTION is withdrawn: it is a measured loss.**
Executor: grok, 2026-09-01. Every row re-run by me.

```
Summary [ 362.140s] 5176 tests run: 5176 passed (4 slow), 15 skipped
FLOOR=0
```

| # | what | my re-run |
|---|---|---|
| 1 | ★ send wakes a parked receive | ✅ with the visibility re-put applied |
| 2 | ★ parked receive times out | ✅ |
| 3 | ★ one receive path | ✅ `State/take`, built at `:init`, used by `receive` and `-tick` |
| 4 | `wait-ns = 0` behaviour unchanged | ✅ gates unedited — **but see the source-compat finding** |
| 5 | empty round-trips fall | ✅ 3 messages → 2 receive calls (was a 10-spin) |
| 6 | idle queue is silent | ✅ the wake tick is conditional on waiters |
| 7 | waiters are `:ephemeral` | ✅ `:durable []` |
| 8 | FIFO | ✅ |
| 9 | the circuit | ⛔ **REGRESSION — see below** |
| 10 | substrate untouched | ✅ `git diff wat/ src/` empty |
| 11 | floor | ✅ 5176/5176, my own run |

## ★ Row 9: a measured loss, decomposed

The report gave one sample (104.7 s vs 88.6 s). Two runs showed **104.8 s and 181.4 s** — so the
first finding is not the mean, it is that **the circuit became nondeterministic**, and a benchmark
that swings 73% cannot measure anything. My row asked for a wall time to be *reported*, which was
right, and for **one**, which was not: a single sample of a variable quantity is the same error class
as an unverified census.

Isolating it by configuration, two runs each:

| configuration | time | workers |
|---|---|---|
| pre-stone | **88.6 s** | 9 |
| new queue, **old** worker logic, `wait-ns 0` | 99.4 / 98.5 s | 10, 8 |
| new queue, **new** worker (done-flag), `wait-ns 0` | 106.0 / 105.8 s | **4** |
| new queue, new worker, `wait 50 ms` | 104.8 / **181.4 s** | 4 |

Three distinct costs:

- **+10 s — the queue's own machinery**, present with the feature entirely unused. Unattributed;
  see the ruled-out list below.
- **+7 s — the worker's `done` flag**, which collapses participation from 9 workers to 4. At
  `wait-ns 0` an empty reply means *"not filled yet"*, **not** *"no work"* — the circuit publishes
  before workers start, so a worker that stops on its first empty exits before the queue fills.
- **the variance** — only with `wait > 0`.

**Ruled out by measurement, so nobody re-derives them:** `after` construction (25 µs × 8000 = 0.2 s);
the 0-ns wake tick (conditional on waiters — never armed at `wait 0`); closure-vs-named dispatch
(1.95 µs vs 1.68 µs); and the 50 ms wait as the *baseline* cause (the shift is present at `wait 0`).

## What I did about it

**The capability is committed; the adoption is withdrawn.** `circuit.wat` is back to its original
worker logic with `:wait-ns 0` — long polling exists, the circuit does not use it. That is honest:
the queue can now wait, and the demo does not yet benefit.

★ **Long polling cannot help this circuit by construction.** It publishes all 2000 messages *before*
workers start, so the queue is never empty during useful work. Long polling pays off when consumers
outpace producers; this benchmark is the opposite shape. **The stone is right and the witness is
wrong** — row 5 shows the real win (3 messages → 2 receive calls, was a 10-spin), in the case the
feature is actually for.

## A finding row 4 nearly missed

`wait-ns` is a **required** field, so the change is behaviour-additive but **source-breaking**: every
existing `ReceiveRequest` construction must be edited to pass `:wait-ns 0`. I discovered this by
trying to revert the circuit and finding it would not compile. Row 4 read "gates unedited" as
green — true, because those constructions live inside `sqs.wat` itself, which the strike updated.
An external consumer would not have been so lucky.

## Two substrate gaps that are MINE

`sqs.wat:306` says it plainly: *"0-ns tick when we both ReplyTo and need to re-arm (no
ReplyToAndArm)."*

1. **No `ReplyToAndArm`.** `Reply` and `NoReply` each have an `AndArm` twin; `ReplyTo` does not. I
   added the variant without its twin.
2. **A surface `ReplyTo` wraps in the *current arm's* reply variant** — which I specified explicitly
   — so a `send` arm waking a `receive` waiter would produce `Reply::Send`. **Cross-op waking is the
   entire use case**, and my rule made it inexpressible.

Together these forced the queue through an internal tick to wake anyone. The workaround is correct
and costs little (measured), but the gaps are real and were introduced one stone ago.

## Next

Attribute the +10 s before the capstone builds on this, and close the two `Outcome` gaps. The
capstone should not adopt long polling until a benchmark exists whose shape it can actually help.
