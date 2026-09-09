# EXPECTATIONS — find the 934 milliseconds

Written before the strike. Graded by my own re-run of every row.

⛔ = a GATE: an invariant this stone controls. ▪ = a REPORT: an observation, recorded not gated.

★ **This is a measurement stone, so the gates are about measuring, not about a number.** Whether
the 934 ms is recoverable is an *outcome* — it may be irreducible, and STOP-2/STOP-3 are the
honest exits. Gating "publish returns to 23723" would gate a consequence, which has fired wrongly
three times in this arc.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ⛔ **the A/B is reproduced before any edit** | the SCORE carries both arms, 3 runs each | two arms ~900 ms apart, or STOP-1 fired with its numbers |
| 2 | ★★ **a mechanism is NAMED, with the measurement that shows it** | read the SCORE | a stated mechanism plus the probe or variant that demonstrates it — **a plausible story with no measurement fails this row** |
| 3 | ⛔ **behaviour is unchanged** | `probe-transient-means-try-again.wat`, my run | `RETRY=send=Ok;total=10;distinct=10`; EXHAUST/CONSTRAINT/FATAL each `Lost:disconnected` with its own named assertion |
| 4 | ⛔ **every variant was behaviour-checked, not just the winner** | the SCORE's variant table | each row reports its probe result, not only its timing |
| 5 | ⛔ **the retry was not traded away** | read the send arm | `:Transient` still retried on the a1/a2/a3 budget; no outcome silently folded into another |
| 6 | ⛔ **no `_` returned to a store response** | `grep -n "(_ " wat-scripts/queue/sqs.wat` | matches only on `ConnectOutcome`/`RecvOutcome`, none on a store response |
| 7 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ the count is reported, not gated |
| 8 | ⛔ **delivery still exact** | `circuit.wat` ×5 | `total=8000; distinct=8000` every run |
| 9 | **blast radius** | `git status --porcelain` | `sqs.wat` + `scratch-pad/` + the SCORE. **No `wat/`, no `src/`** |

## REPORTS — recorded, not gated

| ▪ | what | why it is not a gate |
|---|---|---|
| a | `publish` median after the change | the stone controls whether the cost is *found*, not whether it is *removable* |
| b | the variant table: one variable each, 3 runs, median | the search itself is the deliverable |
| c | `dup` per run | `distinct` is the invariant; `dup` is an observation |
| d | any new micro-probe and its number | reusable beyond this stone |

## RUNTIME

60–90 min. Most of it is wall-clock: each circuit run is ~40 s and a variant needs three, so a
six-variant bisect is ~12 min of pure measurement before any thinking.

## TRAP DOORS

- ⚠ **Run-to-run variance is ~±150 ms.** My own three-run arms spread 23629–23777 and
  24506–24751. A variant that "wins" by 100 ms has measured nothing. **Three runs, report the
  median, and treat anything under ~300 ms as noise.**
- ⚠ **The box must be quiet.** Both my A/B arms were taken with nothing else running. A variant
  measured while a floor is compiling is not comparable to one that was not.
- ⚠ **The fastest variant may be the wrong one.** Row 4 exists because a bisect naturally
  behaviour-checks only the winner, and a variant that is fast *because* it dropped an arm would
  pass a timing check and fail the queue.
- ⚠⚠ **The strongest pull here is to ship a local workaround without naming the mechanism.** If
  the cost is a general fact about evaluating a large body in a `let` versus a `match` arm, that
  fact is worth more than these 934 ms — it applies to every service arm in the tree. Row 2 is the
  row I care about most.
- ⚠ **The ack arm got the same restructure and costs nothing.** If a proposed mechanism does not
  also explain *that*, it is incomplete — say so rather than rounding it off.
