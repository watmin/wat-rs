# SCORE — the wait names its verb

**STRUCK.** Executor: grok, 2026-09-03. Every row re-run by me on a quiet box.

```
Summary [ 360.203s] 5213 tests run: 5213 passed (3 slow), 15 skipped
FLOOR=0        .floor/2026-09-03T22-45-39Z/        my own run, 0 FAIL/TIMEOUT lines
```

## ★ THE CONTRACT DECISION HOLDS

```
$ grep -n 'wait' wat-scripts/queue/sqs.wat | grep -E '<=|>=|< 1|> 0|i64::<|i64::>'
(empty)
```

**No comparison against a wait magnitude survives anywhere in the queue.** `sqs.wat:496` is now
`(:wat::core::match wait ((:queue::Queue::Wait::Immediate) …))`. The mode is read from the
constructor and never from the number — which is the whole stone, and it is grep-checkable rather
than argued.

Zero is now walled at three depths:

| depth | mechanism | evidence |
|---|---|---|
| **language** | `(Millisecond 0)` has no form | check-time `MalformedForm`, span, program never runs |
| **protocol** | `wait-ns 0` has no spelling | the field is `wait <- Queue::Wait` |
| **wire** | a zero payload is refused, service lives | `zero=[MALFORMED…]; then=[ok:250000000]`, 3/3 |

## Rows — my re-run

| # | row | result |
|---|---|---|
| 1 | ★★ no magnitude comparison | ✅ **empty grep** |
| 2 | the fork is a `match` | ✅ `sqs.wat:496` |
| 3 | zero cannot be spelled | ✅ **rung 3 confirmed** — see below |
| 4 | zero over the wire refused, service lives | ✅ 3/3 |
| 5 | ★ the invariant | ✅ `total=8000; distinct=8000; dup=0`, **all five runs** |
| 6 | throughput | ⚠ **one of five outside my band** — see below |
| 7 | codemod recorded + idempotent | ✅ **verified by me**: re-applied to all seven paths, diff sha `83557d8e61f5290f` before and after |
| 8 | the finder's census, not mine | ⚠ **13 sites / 7 files. My DESIGN said 6.** Mine again |
| 9 | clamp behaviour unchanged | ✅ condition and constants identical; WHY added and accurate |
| 10 | six arm delays untouched | ✅ zero diff hunks |
| 11 | `sqs.wat:11-12` (S21) | ✅ **better than asked** — see below |
| 12 | prose the codemod cannot reach | ✅ the 9 residual `:wait-ns` hits are all inside the codemod itself |
| 13 | helpers not merged | ✅ three, at `:796 :809 :878` |
| 14 | the floor | ✅ **5213/5213, my run.** Stone D race `PASS [1.637s]` — not chased, not re-run |

### Row 3 — rung 3, established properly

`:UpTo (:wat::time::Millisecond 0)` inside a **never-called** function fails at check time:

```
#wat.check/MalformedForm  head ":wat::time::Millisecond"
  "a wait must be positive; … zero is a legal MEASUREMENT … and an illegal COMMITMENT (got 0)"
  span: line 5, col 55
```

The program never runs. That is the definition of rung 3, and it needed the never-called form to
prove — a literal in a body that *does* run cannot distinguish check-time refusal from a runtime
raise.

★ **But the same literal produced two different error shapes.** In one position, the clean
`MalformedForm` above. In another (nested inside `(show …)` in `main`'s body) it surfaced as a raw
`LociDiedError/Panic` carrying the same text. One wall, two diagnostics, and only one of them is a
type error a reader can act on. **S23**, not chased.

### Row 6 — one run outside the band, recorded rather than waved off

```
baseline (mine, pre-stone)   25582 / 26170 / 26604 / 26735 / 26403 ms
post     (mine)              26449 / 26712 / 26932 / 27388 / 26503 ms
post     (grok's)            25591 … 26437 ms
```

My band was 25.5–27.0 s. **One run, 27388 ms, is 1.4% above the ceiling.** Mean moved +1.9%.

**I do not claim a regression, and I do not dismiss it.** The ranges overlap my own baseline
(26449 < 26735), grok's five runs sat entirely inside the band, and I have no mechanism — Stone B
removed a comparison and added an enum match on a path taken once per receive. Two five-run samples
straddling a boundary is exactly the shape my own method rule was written for: *a perf row needs a
distribution, not a sample*, and I have two distributions that disagree by less than their spread.
Recorded as an open number, with the raw runs above so the next measurement has something to beat.

### Row 11 — S21 closed better than specified

I asked for a comment saying both things. It says three, and the third is the one I would have
missed:

> *"Time types cross a service boundary now (Stone B-pre). `now-ns` / `visibility-ns` stay i64 so a
> fixture can drive the clock as a value. **`wait` is the exception: it is a mode (`:Immediate` /
> `:UpTo [NonZeroDuration]`), not a magnitude.**"*

The old comment argued testability and concealed an impossibility. The new one states the
capability, keeps the testability reason where it is still true, and names *why* one field is
different in kind. **S21 closed.**

## ★ MY CENSUS WAS WRONG A FOURTH TIME

I wrote **13 sites, 6 files**. The finder found **13 sites, 7 files**. The site count was right; I
miscounted my own enumeration — the seven files are listed in my DESIGN's own table and I wrote six
below it.

Four census errors this campaign, each a different failure: omitted constructors; omitted an entire
directory; an empty grep reported as a fact; and now a miscount of a list I had already written out
correctly. **EXPECTATIONS row 8 anticipated this** — *"if the finder disagrees with me, the finder is
right"* — and it is the only reason this cost nothing.

## What landed

`:queue::Queue::Wait` with `:Immediate []` / `:UpTo [d <- :wat::time::NonZeroDuration]` in the
surface. `wait-ns <- i64` → `wait <- :queue::Queue::Wait`. The `<= 0` fork is a `match`.
`:deadline-ns (+ start-ns (nanoseconds d))`. The clamp kept, with the WHY it never had. Three helpers
still three. A recorded, idempotent migration at `wat-scripts/fixes/wait-ns-to-wait.wat`.

## Still open

- **Stone C** — `Alarm :delay`, `Milliseconds`, `visible`/`unacked`.
- **Stone D** — the helper vocabulary. **Owns the live race**; reproducer deterministic at
  `probe-refused-retry-self-consumes.wat`. Also owns the `do-receive`/`do-receive-wait` merge.
- **Item 3c/3d** — reactor drop, reply-drop. The chaos work, untouched.
- **S15**–**S23**, incl. **S23** (two error shapes for one wall) and the row-6 throughput number.
