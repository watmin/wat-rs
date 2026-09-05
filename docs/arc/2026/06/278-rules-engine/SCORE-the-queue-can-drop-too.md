# SCORE — the queue can drop too

**STRUCK.** Executor: grok, 2026-09-05. Tree safe, uncommitted.
Recorded migration `wat-scripts/fixes/declare-queue-drop-knobs.wat` + `sqs.wat` impls +
`circuit.wat` cells + two `#[ignore]`d tests. No `wat/`, no `src/`.

```
Summary [ 363.587s] 5214 tests run: 5214 passed (3 slow), 22 skipped
```

`.floor/2026-09-05T10-42-13Z/`

## THE KNOBS

`:queue::queue` durable: `drop-recv-bp`, `drop-ack-bp`, shared `drop-seed`. Defaults 0.
`hit?` copies `:fanout::seen`'s mark: seeded `int-from`, hide reply as `:wat::core::None`.
Seed advances only when that verb's rate is > 0.

- **receive**: `take` (the lease) runs first, then the reply is hidden. STOP-3 did not fire.
- **ack**: `Store/delete` runs first, then the reply is hidden.

## ROW 4 — a second ack is a no-op

Asked from a run, not assumed. `ack-drop` ×6 all terminated; none hit
`queue.ack: store delete failed`. mem-store: *"Missing key is a no-op"* and always
`DeleteResponse::Success`. sqlite maps `Ok` (including DELETE of 0 rows) to `Success`.
The client's retry after a hidden ack is Success, not an error. STOP-2 did not fire.

## THE MIGRATION

Census (`wat --grep`): **41** keyword hits across the 10 files (40 constructors + 1 type
annotation at `sqs.wat:130`). Applier walks list heads; the type annotation is not a
constructor and is correctly left. 40 constructors gained
`:drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0`.

Re-run after apply: **0 bytes changed** (`diff` of the two `git diff`s is empty). STOP-1
did not fire.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ dropped receive does not hang | ✅ ×6 terminate, `total=100; distinct=100; dup=0` |
| 2 | ★★ dropped ack does not hang | ✅ ×6 terminate, `total=100; distinct=100; dup=0` |
| 3 | ★★ migration complete | ✅ floor green; corpus gate type-checked the 7 scratch probes |
| 4 | ⛔ second ack no-op or error? | ✅ **no-op** (`Success`); see above |
| 5 | codemod idempotent | ✅ re-run changes 0 bytes |
| 6 | existing chaos | ⚠ mark-drop **6/6** `dup=0; gave-back=0`. check-drop: `gave-back` fired **2/6**; **5/6** `dup=0`; **1/6** `total=101; dup=1` (run 1, captured, not re-run) |
| 7 | rate-0 invariant | ✅ `total=8000; distinct=8000; dup=0; seen-recorded=8000; gave-back=0` ×5 |
| 8 | the floor | ✅ **5214/5214, 22 skipped** |
| 9 | timings | report only: publish **47915 49899 48617 48043 49631** (before `45984–46672`) |

Cells: `:user::drop-recv-tiny` / `drop_recv_tiny`, `:user::drop-ack-tiny` / `drop_ack_tiny`.
22 skipped = 20 + the two new `#[ignore]`d cells.

## ⚠ FINDING — check-drop run 1 duplicated

Before-state on `4a45b6362` was 12/12 check-drop `dup=0`. This strike's check-drop ×6 had
one `total=101; distinct=100; dup=1` with `gave-back=0`. Not a hang and not a loss. Not
patched. Queue knobs were 0 on that cell; receive still rebuilds the durable Record on
every call (seed unchanged when rate is 0). Timing, not a second ack.

## ⚠ FINDING — publish sits above the old band

Rate-0 publish 47.9–49.9 s vs 45.9–46.7 s before. Report only; STOP-4 forbade perf work.
Receive now reconstructs `Record` on every call even at rate 0.

## NOT TOUCHED

`Queue/send`. Topic's internal queue (fields present, knobs stay 0). Redelivery fixture.
Retry budget. Perf.

---

# ORCHESTRATOR GRADING — claude, 2026-09-05

**STRUCK, and committed.** Every row re-run by me.

| # | my result | |
|---|---|---|
| 1 | recv-drop ×6: **6/6 terminate**, `total=100; distinct=100; dup=0` | ✅ |
| 2 | ack-drop ×6: **6/6 terminate**, `total=100; distinct=100; dup=0` | ✅ |
| 3 | floor green — the corpus gate type-checked all 7 scratch probes | ✅ |
| 4 | second ack is a no-op: grok's run, corroborated — my 6/6 ack-drop runs, **none hit `store delete failed`** | ✅ |
| 5 | **idempotent — I re-ran the codemod myself: `diff` of the two `git diff`s is empty, 0 bytes** | ✅ |
| 6 | check-drop ×12: **12/12 `dup=0`**, `gave-back` 3/12. **I did not reproduce the duplicate** | ⚠ |
| 7 | rate-0 ×5: `total=8000; distinct=8000; dup=0; gave-back=0` | ✅ |
| 8 | `Summary [ 360.893s] 5214 passed, **22 skipped**` — `.floor/2026-09-05T11-00-56Z/` | ✅ |
| 9 | publish `48163 48599 49145 50664 48457` vs before `45984–46672` | ⚠ |

★ **All four client calls can now be made to fail.** `receive` and `ack` join `check` and `mark`,
and the deadlines built for them are exercised for the first time. That is the coverage gap this
arc named three stones ago, closed.

## ⚠ FINDING 1 — the duplicate is the design's own residue, and MY ROW WAS WRONG

Grok saw one check-drop run at `total=101; distinct=100; dup=1`. **I ran twelve and saw none.**
Combined: **1 duplicate in 18 runs.** `distinct` held at 100 in grok's run too.

★ That is not a regression. It is the **s3 window** — *redelivery after a successful emit* — which
`DESIGN-a-ledger-is-a-receipt` names as irreducible and which the arc's standing invariant
(`distinct=N; dup >= 0`) explicitly permits. Its first appearance in the wild.

★★ **The defect is in my row, not the code.** Row 6 said `dup = 0`. That is the **third time** I
have written an observation as a gate:

1. `BRIEF-a-claim-remembers-its-owner` STOP-3 — `dup=0`, contradicting the tracker's own ruling
2. `EXPECTATIONS-a-call-outcome-cannot-lie` row 8 — a publish *band* from one 5-run sample
3. here, row 6 — `dup=0` because it had been `dup=0` twelve times

**A row must state what must HOLD, not what was last OBSERVED.** `distinct = N` is an invariant
and belongs in a gate. `dup` and `publish` are measurements and belong in a report. I will write
them that way.

## ⚠ FINDING 2 — a real +5 % regression, mechanism located

publish `48163–50664` (mine) vs `45984–46672` before — **worse than the executor measured**
(47.9–49.9), and reproducible.

Mechanism, read off the diff: `receive` and `ack` now build a fresh `queue::queue::Record` on
**every call** to store the advanced `drop-seed`, where they previously carried
`(:queue::queue::State/durable s)` forward by reference. **At rate 0 the seed never moves**, so
the rebuild is pure waste on the hottest path in the circuit.

★ The guard is one line — rebuild only when that verb's rate is `> 0`. **STOP-4 forbade perf
work and the executor correctly did not take it.** It is now a located, mechanism-known item for
the perf phase, not a mystery to rediscover.

## THE MIGRATION

41 keyword hits, 40 constructors edited, 1 correctly skipped (a type annotation at
`sqs.wat:130`, not a constructor). 11 files. **Idempotency verified by me, not read.**

## NEXT

1. The redelivery fixture that kept its name and lost its meaning.
2. Rung 3: an undeadlined generated client method should have no form.
3. **Then perf** — and it now has two located items: the send-path double scan, and this
   stone's Record rebuild.
