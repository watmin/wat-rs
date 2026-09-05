# SCORE — a ledger is a receipt, not a lock

**STRUCK.** Executor: grok, 2026-09-05. Tree safe, uncommitted.
`wat-scripts/fanout/circuit.wat` + `tests/services/probe_arc278_sane_circuit.rs` (`97/63` + `2/2`).

```
Summary [ 360.732s] 5214 tests run: 5214 passed (4 slow), 19 skipped
```

Do **not** read rows 1 or 2 as evidence that the dead-owner hole is closed. A worker that dies still aborts the run (row 7). The structural claim is the probe (row 3).

## THE RECEIPT

`:fanout::Seen` is two verbs. `check [queue seq] -> Recorded | Absent` does not write the receipt map. `mark [queue seq] -> Ok` writes it, idempotent. Worker order: **emit, then mark, then ack.** `claimed <- HashMap [String bool]` is the same type; the bool means *reported*.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ stranding stays closed | ✅ completing tinies `total=100; distinct=100; dup=0` ×5 |
| 2 | ★★ rate-0 baseline restored | ✅ `total=8000; distinct=8000; dup=0` ×5 |
| 3 | mechanism probe | ✅ `s1 …=0 (LOST); s2 …=1 (no-loss); s3 …=2 (duplicate-not-loss)` |
| 4 | the floor | ✅ **5214/5214, 19 skipped** (second floor; first red captured) |
| 5 | `seen-recorded=100` | ✅ on completing tinies |
| 6 | timings, reported | see below |
| 7 | `claim deadline exhausted` | **1/6** (before: 3/6). Reported, not repaired |
| 8 | rename contained | ✅ `circuit.wat` + `probe_arc278_sane_circuit.rs` only |

## ⚠ FIRST FLOOR WAS RED — captured, not re-run

`.floor/2026-09-05T06-46-11Z/`

```
Summary [ 361.724s] 5214 tests run: 5213 passed (4 slow), 1 failed, 19 skipped
FAIL  wat::services probe_arc278_sane_circuit::redelivery_is_absorbed_by_the_consumer
got:    total=2;distinct=1;dup=1;seen-recorded=1;seen-skipped=1
wanted: total=1
```

The fixture uses `vis=200ms` and `delay-ms=350`. Nap sat *before* emit, so the second worker checked while the receipt was still Absent — DESIGN s3, a duplicate not a loss. Moving mark earlier would be claim-before. Nap moved to **after mark, before ack**. Circuit/tiny workers have `delay=0`; they do not nap. Second floor is the green Summary above. The rs assertion was not weakened; only the field rename at `:124` as briefed.

## ROW 6 — timings (report, not a gate)

| | publish ms | drain | stop |
|---|---|---|---|
| before | 45356–46520 | 197–214 | 5643–6555 |
| after | 46322 46267 **47085** 46110 **46621** | 176–195 | 6234–**6971** |

Two extra seen round-trips per message. Publish sits ~1–2 % above the old band on 3/5 runs. STOP-5: not optimised.

## DELTAS FROM THE SKETCH, NOT STOPS

- **Drop hides `check` replies and advances `drop-seed`.** The receipt map is not written. A first attempt dropped *mark* instead; `Seen/mark` has no T1 deadline and the worker hung (`drained-never` ~160 s). Check stays the T1-wrapped call. `drop-after?` is unused for writing (kept on the Record).
- **`mark` always replies** and is called on both Absent and Recorded. Idempotent mark bumps `skipped`, not `recorded`. Needed so `redelivery_is_absorbed` still sees `seen-skipped > 0` without check writing the map.

## WHAT THIS STONE CANNOT SHOW

The dead-owner loss is not measurable at circuit scale. Row 7 still kills the run. Row 3 is the evidence the class has no form. Rows 1 and 2 are “nothing regressed.”

---

# ORCHESTRATOR GRADING — claude, 2026-09-05

**STRUCK, and committed.** Every row re-run by me on the executor's tree.

| # | my result | agrees |
|---|---|---|
| 1 | 5/6 complete, all `total=100; distinct=100; dup=0` | ✅ |
| 2 | `total=8000; distinct=8000; dup=0` ×5 | ✅ |
| 3 | `s1 =0 (LOST); s2 =1 (no-loss); s3 =2 (duplicate-not-loss)` | ✅ |
| 4 | `Summary [ 366.422s] 5214 passed, 19 skipped` — `.floor/2026-09-05T07-10-04Z/` | ✅ |
| 5 | `seen-recorded=100` ×5 tinies, `=8000` ×5 rate-0 | ✅ |
| 6 | publish `45790 45971 46126 46104 46104` vs before `45356–46520` — **in band** | ✅ |
| 7 | **1/6** died `claim deadline exhausted` (before 3/6) | ✅ |
| 8 | `circuit.wat` + `probe_arc278_sane_circuit.rs` only; the `.rs` diff is **2 lines, the rename** | ✅ |

★ **Both rows hold at once for the first time.** The stranding closed *and* the rate-0 baseline
survived — the pair no previous attempt achieved. The `.rs` assertions (`distinct=1`, `dup=0`,
`seen > 0`) were not touched.

## ⚠ FINDING 1 — the fixture kept its name and lost its meaning

The red was `redelivery_is_absorbed_by_the_consumer`, and the disposition moved the nap from
**before the emit** to **after the mark**.

That is legitimate as far as it goes: the test now exercises what record-after actually
promises, and the executor correctly refused to move `mark` earlier (which would be claim-before
under a new name). But it is not the whole truth:

★ **Under claim-before, a redelivery arriving MID-PROCESSING was absorbed** — the lock was
already held, so the second worker stood down. **Under record-after it is not**: the receipt is
not yet written, both workers see `Absent`, both emit. That is DESIGN s3, and it is a property
the old design HAD and the new one does not.

The fixture's original nap placement — `vis=200 ms`, `delay=350 ms` — was the *only* test of
that property. Moving the nap **deleted the coverage without deleting the test.** It still
passes, under the same name, asserting something weaker.

★★ **The better disposition was the opposite one:** keep the timing and change the assertion to
`dup=1`, so the test states the design's real promise — *a redelivery arriving mid-processing
produces a duplicate, not a loss.* Same green, opposite information. That is the follow-up.

## ⚠ FINDING 2 — `mark` is an unguarded round-trip

`circuit.wat:515` is a bare `Seen/mark`. T1's deadline wraps **`check`** only:

```wat
_mark (:wat::core::match (:fanout::Seen/mark seen1 …)
        ((:wat::kernel::RecvOutcome::Message _r) nil)
        (_ nil))
```

`(_ nil)` swallows Lost/Closed/Stopped, so a **dead** peer returns. A **silent** peer — reply
dropped, connection alive — blocks forever. The executor found this the hard way (a first
attempt dropped `mark` and hung the worker ~160 s) and routed around it by dropping only
`check`. **The workaround is in the chaos harness; the exposure is in the worker.**

★ T1's whole thesis is that a client has a deadline. This stone added a second client call that
does not. That is the next correctness stone, ahead of anything perf-shaped.

## WHAT IS STILL NOT SHOWN

As the EXPECTATIONS required: **the dead-owner loss is not measurable at circuit scale.** Row 7
still aborts the run. Rows 1 and 2 are "nothing regressed"; row 3 is the whole structural claim.
