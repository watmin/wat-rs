# SCORE — a call outcome cannot lie

**STRUCK.** Executor: grok, 2026-09-05. Tree safe, uncommitted.
`wat/service.wat` + `wat-scripts/fanout/circuit.wat` (`73/52`). No `.rs`.

```
Summary [ 367.253s] 5214 tests run: 5214 passed (4 slow), 19 skipped
```

## ROW 1 — the pair has no form

`call-by-deadline` now returns `(:wat::service::CallOutcome :- [:O])`:

```
Answered [reply <- :O] | PeerGone [] | DeadlineFired []
```

`(None, 0)` and `(Some x, 2)` cannot be written. Receive at `circuit.wat:401` matches the three arms; `PeerGone` and `DeadlineFired` each redial (identical behaviour, stated twice). No `first`/`second` on a helper result anywhere.

This is the tree's first wat-declared parametric `defenum`. Probe syntax copied. `:wat::enum::Pure` copied.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ receive cannot ignore the discriminator | ✅ three-arm `match recv-got`; no pair-read of a call result |
| 2 | ★★ rate-0 identical | ✅ `total=8000; distinct=8000; dup=0; seen-recorded=8000` ×5 |
| 3 | ★★ tiny identical | ✅ **6/6**, `total=100; distinct=100; dup=0; seen-recorded=100` |
| 4 | the floor | ✅ **5214/5214, 19 skipped** |
| 5 | enum is the only new stdlib form | ✅ `defenum` immediately before the helper; hunks start at `:3117` |
| 6 | goldens undisturbed | ✅ no `peers_bijection` failure |
| 7 | blast radius | ✅ `service.wat` + `circuit.wat` only |
| 8 | timings | publish **45944 46186 46619 46213 45708** (band 45547–46716) |

`seen-skipped` remains a noisy counter (tiny 15–17; circuit 4–16). The required fields did not move.

## NOT TOUCHED

Lost vs Closed stays merged as `PeerGone`. Generated methods still undeadlined. `claim deadline exhausted` and the redelivery fixture untouched.

---

# ORCHESTRATOR GRADING — claude, 2026-09-05

**STRUCK, and committed.** Rows 1–7 verified by my own runs. **Row 8 did not hold as written,
and that is my error, not the executor's** — see below.

| # | my result | |
|---|---|---|
| 1 | `grep "first recv-got\|second got\|first got"` → **empty**. Receive matches `CallOutcome::Answered`. 12 `CallOutcome::` uses = 4 sites × 3 arms | ✅ |
| 2 | rate-0 ×5: `total=8000; distinct=8000; dup=0; seen-recorded=8000` | ✅ |
| 3 | tiny **6/6**: `total=100; distinct=100; dup=0; seen-recorded=100` | ✅ |
| 4 | `Summary [ 360.766s] 5214 passed, 19 skipped` — `.floor/2026-09-05T08-39-33Z/` | ✅ |
| 5 | `wat/service.wat` hunks all at **`:3120`+** — nothing above `:896` | ✅ |
| 6 | no `peers_bijection` failure | ✅ |
| 7 | `service.wat` + `circuit.wat` only, **no `.rs`** | ✅ |
| 8 | publish `45965 46578 46581 47397 47486` vs my band `45547–46716` — **2 of 5 above it** | ⚠ |

★ **Row 1 is the stone and it holds structurally, not behaviourally.** `(None, 0)` and
`(Some x, 2)` have no form. The site that ignored the discriminator (`circuit.wat:401`) now
matches three arms, and both non-answer arms redial — identical behaviour, *stated* instead of
derived from an integer.

## ⚠ ROW 8 — MY EXPECTATIONS ROW WAS MALFORMED

| | publish ms | median |
|---|---|---|
| before | 45547 45923 46074 46100 46716 | 46074 |
| after | 45965 46578 46581 47397 47486 | 46581 |

Medians differ by **1.1 %**; the bands overlap across most of their range; and a three-arm
`match` replacing an integer compare has **no plausible mechanism** for a real cost. My reading
is box noise — but I am reporting the numbers rather than declaring a pass, because the row as
written says `45547–46716` and two runs are outside it.

★ **The defect is in the row, not the code.** I turned **five observations into a tolerance.**
A band from a single 5-run sample describes what that sample did; it is not a margin, and
band-containment is not a test. The row should have read *"median within 5 % of 46074"* — or,
better for a refactor, *"report and compare, no gate."*

★★ **This extends the tracker's existing method rule.** *"A perf row needs a distribution, not a
sample"* buys you one configuration honestly. **Comparing two configurations needs a stated
margin**, because the spread of a five-run sample is itself noisy — here 2.6 % before and 3.3 %
after, on a change that cannot cost anything.

## NOT TOUCHED, STILL OPEN

`Lost` vs `Closed` stays merged as `PeerGone` (cut in the DESIGN, with its reason). Generated
client methods are still callable with no deadline — the standing rung-3 stone. The
`claim deadline exhausted` crash. The redelivery fixture that kept its name and lost its meaning.
The send-path double scan.
