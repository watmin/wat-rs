# EXPECTATIONS — the queue can drop too

Written **before** the strike. Every "before" is a run of mine on `4a45b6362`.

| # | what | expected |
|---|---|---|
| 1 | **★★ a dropped `receive` reply does not hang** | new recv-drop cell ×6: every run **terminates**, `total=100; distinct=100; dup=0` |
| 2 | **★★ a dropped `ack` reply does not hang** | new ack-drop cell ×6: every run **terminates**, `total=100; distinct=100; dup=0` |
| 3 | **★★ the migration is complete** | the floor is green — the corpus gate type-checks all 7 scratch-pad probes; a missed constructor reds it |
| 4 | **⛔ is a second ack a no-op?** | **answer it explicitly in the SCORE.** If it errors, that is STOP-2 |
| 5 | the codemod is idempotent | re-running it changes **0 bytes** — show the `git diff` is empty |
| 6 | existing chaos unaffected | check-drop ×6 and mark-drop ×6: `total=100; distinct=100; dup=0`; `gave-back` still fires on check-drop |
| 7 | rate-0 invariant | circuit ×5: `total=8000; distinct=8000; dup=0; seen-recorded=8000; gave-back=0` |
| 8 | **the floor** | `5214 passed`, **22 skipped** (20 + the two new cells) |
| 9 | timings | report only, no gate. Before: publish `45984–46672` |

### Before-state, recorded verbatim

```
row 6  check-drop ×12: 3/12 gave-back (1,1,2), all total=100; distinct=100; dup=0
       mark-drop  ×6:  6/6, total=100; distinct=100; dup=0; gave-back=0
row 7  total=8000; distinct=8000; dup=0 ×5; seen-recorded=8000; gave-back=0
row 8  Summary [ 360.684s] 5214 passed, 20 skipped   .floor/2026-09-05T10-09-46Z/
row 9  publish 45984 46063 46171 46298 46672
```

## ⛔ ROW 4 IS A QUESTION, NOT A CHECKBOX

`ack` deletes the row, then the reply is dropped, then the client retries. **Does the second ack
succeed, no-op, or error?** I do not know, and the DESIGN says so rather than assuming. Answer it
from a run, in the SCORE. An error there is a real defect in `ack` idempotency and is STOP-2 —
report it, do not work around it.

## ⛔ ROW 3 IS WHY THE CODEMOD IS SAFE

Seven scratch-pad probes construct `queue::queue::Record`, and
`every_wat_scripts_file_loads_on_the_current_runtime` type-checks every one. **A constructor the
migration misses cannot hide** — it reds the floor. That is the census, and it is why this is a
tooled migration rather than a careful hand-edit.

## RUNTIME PREDICTION

70–100 min — the longest stone in this run. The codemod is most of it: write it, census it, diff
it, apply it. The two impls and two cells are small once the field exists.

## TRAP DOORS, NAMED

1. **A non-idempotent codemod.** Row 5 exists for this. Re-run it and show an empty diff.
2. **Dropping the work instead of the reply.** If `receive` skips the lease or `ack` skips the
   delete, the fault modelled is "the server did nothing", not "the reply was lost" — and every
   row would still pass. STOP-3.
3. **A shared seed advancing differently.** `:fanout::seen` advances its seed only when that
   verb's rate is > 0. Copy that, or turning on one knob changes another's draw sequence.
4. **22 skipped, not 20.** Two new `#[ignore]`d cells. State it, so it is not read as silencing.
