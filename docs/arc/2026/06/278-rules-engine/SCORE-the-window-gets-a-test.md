# SCORE — the window gets a test

**STRUCK.** Executor: grok, 2026-09-05. Tree safe, uncommitted.
`wat-scripts/fanout/circuit.wat` + `tests/services/probe_arc278_sane_circuit.rs`. No `wat/`, no
`sqs.wat`, no `src/`, no codemod.

```
Summary [ 363.365s] 5215 tests run: 5215 passed (4 slow), 22 skipped
```

`.floor/2026-09-05T11-23-32Z/`

## THE GATE

`redelivery_mid_processing_never_loses` is on the floor, not `#[ignore]`d. It asserts
**`distinct = 1`**. `total` and `dup` are `eprintln!`. A future re-check-and-skip that made
`total = 1` would still pass.

The window is forced: `vis = 200 ms`, `work-delay-ms = 350`, `ack-delay-ms = 0`. The work nap
sits between the `check` result (`absent?`) and the `conj` of the `Outcome`. The ack nap stays
after `mark`, before `ack`.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ deterministic ×6 | ✅ `distinct=1` every run |
| 2 | ★★ window actually forced | ✅ same 6: **`total=2; dup=1`** (reported, not gated) |
| 3 | ⛔ gate is `distinct` alone | ✅ no `assert` on `total`/`dup` |
| 4 | existing redelivery tests | ✅ both still pass |
| 5 | the floor | ✅ **5215/5215, 22 skipped** — one more test, not ignored |
| 6 | rename complete | ✅ no bare `delay-ms`; 6 `mk-worker` sites carry `ack-delay-ms` and `work-delay-ms` |
| 7 | chaos cells | ✅ check-drop ×6 and mark-drop ×6: `distinct=100` |
| 8 | rate-0 | ✅ `total=8000; distinct=8000` ×5 |
| 9 | timings | report only: publish **48891 49023 49045 49811 48851** (before `48163–50664`) |

×6 verbatim: `total=2;distinct=1;dup=1;seen-recorded=1;seen-skipped=1` every time. STOP-3 did
not fire.

## THE RENAME

`delay-ms` → `ack-delay-ms`. New `work-delay-ms`. Absorbed fixture keeps `ack-delay-ms=350;
work-delay-ms=0`. Circuit workers stay `0 0`.

## NOT TOUCHED

The window's behaviour (still a duplicate). The two existing redelivery tests' assertions.
Rung 3. Perf.

---

# ORCHESTRATOR GRADING — claude, 2026-09-05

**STRUCK, and committed.** Every row re-run by me.

| # | my result | |
|---|---|---|
| 1 | ×6: `distinct=1` every run | ✅ |
| 2 | ×6 **byte-identical**: `total=2;distinct=1;dup=1;seen-recorded=1;seen-skipped=1` | ✅ |
| 3 | the test asserts `distinct` **only**; `total`/`dup` are `eprintln!`; **no `#[ignore]`** | ✅ |
| 4 | both existing redelivery tests still pass (on the floor) | ✅ |
| 5 | `Summary [ 359.980s]` **5215 run, 5215 passed, 22 skipped** — `.floor/2026-09-05T11-32-03Z/` | ✅ |
| 6 | 13 uses of `ack-delay-ms`/`work-delay-ms`; **no bare `delay-ms`** | ✅ |
| 7 | check-drop ×3 and mark-drop ×3: `total=100; distinct=100; dup=0` | ✅ |
| 8 | rate-0 ×5: `total=8000; distinct=8000; dup=0` | ✅ |
| 9 | publish `47784 47790 48170 49124 49856` vs before `48163–50664` — reported | ✅ |

★ **Row 5 is the one that proves it runs.** Tests went 5214 → **5215** while skipped stayed at
**22**. One more test, and it is not ignored — which was the failure mode row 5 existed to catch.

## ★★ WHAT ACTUALLY CHANGED — a rumour became a property

The s3 window fired **once in eighteen runs** as a chaos artefact. Here it is **6 of 6, identical
to the byte**.

★ The difference is not luck: the fixture **constructs the precondition** — `vis=200 ms` against
a 350 ms nap placed between `check` and the emit — instead of sampling and hoping. **A rare event
becomes a property when you build its precondition rather than wait for it.** That is worth
carrying: this arc has repeatedly been unable to test the thing that mattered because it was
waiting for a coincidence.

And the gate is aimed at the right neighbour. `distinct=1` reds on **loss**. Duplication —
the design's stated, permitted residue — is reported and does not red.

## ★ THE RULE FROM THE LAST SCORE, APPLIED AND HOLDING

`SCORE-the-queue-can-drop-too` recorded: *a row must state what must HOLD, not what was last
observed.* This stone was the first written under it, and the split survived contact:

- **gated:** `distinct = 1` — an invariant
- **reported:** `total = 2`, `dup = 1` — observations

A later change that made the woken worker re-check and skip would give `total = 1`, strictly
better, and **this test would still pass.** Under the habit I had before, it would have red on
the improvement.

## WHAT REMAINS

The correctness queue is down to one item: **rung 3 — an undeadlined generated client method
should have no form.** It needs a tree-wide census before it can be scoped.

Then perf, with two located, measured items: the send-path double scan, and the `Record` rebuild
on every `receive`/`ack` when the seed has not moved.
