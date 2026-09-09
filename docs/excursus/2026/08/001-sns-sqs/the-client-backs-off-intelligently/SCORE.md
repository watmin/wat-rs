# SCORE — the client backs off intelligently

**SCORED.** Executor: grok, 2026-09-06. Tree dirty, uncommitted.
Full jitter, cap 100, reset on any acceptance.
Beats fixed 1 ms. **Does not beat fixed 25 ms.**

```
Summary [ 406.622s] 5220 tests run: 5220 passed (6 slow), 22 skipped
```

`.floor/2026-09-06T21-44-11Z/`

## THE POLICY

```
ceiling = min(CAP, BASE << attempt)     ;; BASE=1, CAP=100
delay   = uniform [1, ceiling]
attempt = 0 on any Accepted c with c > 0
```

`:wat::rand::int-from` is `[lo, hi)`. The call is `(int-from seed 1 (ceiling+1))`
so a ceiling of 1 is `[1, 2)` = `{1}`, never an empty range. STOP-2 did not
fire. A draw of 0 cannot occur.

`attempt` is the backoff exponent; `retries` is the Accepted-0 count returned
to the phases line. Partial acceptance resets `attempt` to 0 and keeps
`retries` and the seed.

Named defs: `:fanout::BACKOFF-BASE-MS`, `:fanout::BACKOFF-CAP-MS`,
`:fanout::BACKOFF-SEED`. Seed is 1, not the clock.

## THE PROBE

`probe-the-client-backs-off-intelligently.wat` (circuit.wat is not loadable —
it carries `set-redef!` — so the five names are copied; a drift is a probe
bug):

```
base=1;cap=100;a0:cap=1;min=1;max=1;bad=0;a1:cap=2;min=1;max=2;bad=0;a4:cap=16;min=1;max=16;bad=0;a7:cap=100;min=1;max=99;bad=0;a12:cap=100;min=1;max=99;bad=0;distinct-a4=16;same-seed=yes;seq=24,29,3,12,27,18,23,31
```

Attempt 0 is always 1. Attempt 4 hits all 16 values of `[1,16]`. Same seed,
same sequence. Never 0, never above 100.

## CIRCUIT ×5

`ps` before: claude 3.7 %, grok 1.9 %, else < 1 %.

Every run: `total=8000;distinct=8000;dup=0`. `publish-calls=200`.

```
publish              20302 20629 20452 20389 20362     median 20389
full-retries          1383  1393  1388  1388  1385     median  1388
queue-receive-calls   4989  5030  5000  5031  5028     median  5028
asleep               11460 11733 11621 11642 11610     median 11621
setup                10205 10284 10151 10162 10215     median 10205
stop                  6604  8065  7067  6524  6763     median  6763
```

asleep / publish = 57 %. Mean wait = 11621 / 1388 ≈ **8.4 ms**.

| | this stone | fixed 1 ms | fixed 25 ms | spin |
|---|---|---|---|---|
| publish | **20389** | 22395 | **18472** | 22794 |
| retries | 1388 | 3807 | 540 | 4088 |
| receives | 5028 | 5290 | 4687 | 5313 |

## STOP-3 — more principled, not faster than the swept optimum

20389 does not beat 18472. It beats 1 ms (−2006 ms) and spin. It loses to
the 25 ms point by **1917 ms**.

That is what the DESIGN said to write rather than dress a loss as a win.
BASE and CAP were not tuned. They stayed 1 and 100.

The publisher sleeps 11.6 s of a 20.4 s publish window and still finishes
slower than sleeping 13.6 s at a fixed 25 ms — fewer, longer waits at the
sweep's sweet spot steal fewer turns than jitter around 8 ms.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ delay bounded, never zero | ✅ all scans `bad=0`; min 1; max ≤ cap |
| 2 | ★★ it is a DRAW | ✅ attempt 4, 40 seeds: 16 distinct (the whole window) |
| 3 | ★★ attempt resets on ANY acceptance | ✅ partial `c > 0` recurses with `attempt=0` |
| 4 | ⛔ seed is threaded | ✅ `int-from` `state'` carried; same seed → same seq |
| 5 | ⛔ delivery unaffected | ✅ ×5 `total=8000;distinct=8000;dup=0` |
| 6 | ⛔ floor | ✅ `5220 passed (6 slow), 22 skipped` |
| 7 | ⛔ blast | ✅ `circuit.wat`, the probe, this SCORE. No `sqs.wat`, no `sns-fanout.wat`, no `wat/`, no `src/` |
| 8 | ⛔ constants are named bounds | ✅ `BACKOFF-BASE-MS` / `BACKOFF-CAP-MS` / `BACKOFF-SEED` |

## REPORTS

| ▪ | median |
|---|---|
| a | publish **20389** (1 ms 22395 · 25 ms 18472 · spin 22794) |
| b | full-retries **1388** (3807 · 540 · 4088) |
| c | queue-receive-calls **5028** (5290 · 4687 · 5313) |
| d | asleep **11621** ms (57 % of publish); mean wait 8.4 ms |
| e | setup 10205 / stop 6763 |

## STOP TRIGGERS

- **STOP-1** did not fire. Seed threads next to `attempt` / `retries`.
- **STOP-2** did not fire. `[1, ceiling+1)` yields `{1}` when ceiling is 1.
- **STOP-3** **fired as the finding**: 20389 does not beat 18472. Reported. Not tuned.
- **STOP-4** did not fire. `distinct=8000` every run.
- **STOP-5** did not fire. `probe-what-a-1ms-await-costs.wat` was already untracked and is not this stone.

## NOT TOUCHED

`sqs.wat`. `sns-fanout.wat`. `wat/`. `src/`. `Wait :UpTo` on send. Inbox `:cap`.

Tree uncommitted. Do not commit unless asked.
