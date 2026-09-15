# EXPECTATIONS — the probe scan says which of six (nine, measured)

Written **before** the strike. Graded on the orchestrator's **own** re-runs.

Baseline (`33bc6bdab`): floor **5251**/5251, 0 FAIL, clippy 0, happy path `distinct=8000;dup=0`.
Today: **9** failure paths, **1** message, `grep -c 'peer is dead, not a broken pipe' sqs.wat` = **6**
(the `_` hides three more behind one of those six).

⚠ Floor delta as a **band**. ⛔ `timeout -k` on anything that might block.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⭑⭑ **the `_` is gone** (STOP-1) | the diff | four **named** `ScanResponse` arms. A better string over a live wildcard is a fail |
| 2 | ⭑⭑ **nine distinct messages** | `grep -c 'peer is dead, not a broken pipe' sqs.wat` | **0**, and nine different messages present |
| 3 | ⛔ **`Transient` is not called a death** | its message | names it **retryable**, and cites that `transient-means-try-again` applies |
| 4 | ⛔ **caller-caused refusals are not blamed on the store** | `RequestTooLarge` / `RequestMalformed` messages | name **this caller's** request as the cause, carrying `bytes`/`cap` and `path`/`expected`/`got` — the actionable numbers the service handed us |
| 4b | ⛔⛔ **their DISPOSITION is reported** | the SCORE | states what a caller *should* do with a refusal it caused, and that dying while holding the fix is wrong. ⭑ Arc 278 Stone 2 made the RECEIVING half crash-proof; this is the SENDING half of the same doctrine |
| 5 | ⛔ **no disposition changed** (STOP-2) | the diff | all nine still raise, `Stopped` included |
| 6 | ⚠ **`Stopped` reported for a ruling** | the SCORE | says raising on a shutdown is wrong and that 324 siblings stand |
| 7 | ⚠ **`Transient`-should-retry reported** (STOP-3) | the SCORE | a view, not a change |
| 8 | ⭑ **driven where drivable** | the store-fault injector | ≥1 `RecvOutcome` arm driven and its message quoted; reachability of the `ScanResponse` arms stated honestly |
| 9 | ⭑ **no invented causes** (STOP-4) | the diff | `Closed`/`TimedOut`/`Stopped` carry none and claim none |
| 10 | ⭑ happy path | the record invocation | `distinct=8000;dup=0`, completeness counters identical |
| 11 | floor | **Summary** line | `5251 passed` or higher, 0 FAIL |
| 12 | clippy | `--all-targets -D warnings` | 0 |
| 13 | blast radius | `git status --porcelain` | `sqs.wat` only |

## Runtime prediction

**45–75 minutes.** One fold, nine arms, one file, no codemod and no helper.

## Trap-door risks, ranked

1. ⭑⭑ **Row 1 half-done.** Prettying the six visible strings and leaving the `_` alive would "fix" the
   thing the census could see and leave the worse half — including `Transient` — hidden.
2. **Row 3.** Calling a retryable error a death is the specific lie that matters most here, because a
   sibling stone in the same file was drawn to retry exactly that.
3. **Row 5 drifting.** Making `Stopped` a no-op at one site while 324 stand is a one-sided fix of a class
   the builder reserved.
4. **Row 4.** Blaming the store for our own oversized or malformed request sends the reader to the wrong
   process entirely.
