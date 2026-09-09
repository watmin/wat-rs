# EXPECTATIONS — the server manages its own capacity

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE: an invariant this stone controls. ▪ = REPORT: an observation.

★ **`dup` moves from "expected 0" to a pure REPORT in this stone**, and that is a deliberate
consequence, not a slip. A partially-fanned tail message is re-published, so subscribers that
already received it see it again — which is what at-least-once means, and `Seen` is the dedupe.
`distinct` remains the gated invariant.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ★★ **partial admission** | probe: `cap 10`, depth 6, send 8 | `Accepted 4`, depth 10 |
| 2 | ★★ **it is a PREFIX** | same probe, read the stored bodies | exactly the **first 4** are present, in order |
| 3 | ★★ **no room is `Accepted 0`, nothing written** | send 3 at depth 10 | `Accepted 0`, depth unchanged |
| 4 | ★★ **room reappears after a drain** | drain 5, send 8 | `Accepted 5` |
| 5 | ⛔ **no response leaks internals** | `grep -nE "depth|cap" ` the response enums in `sqs.wat` + `sns-fanout.wat` | **no `depth` and no `cap` field on any response arm**; `:Full` is gone |
| 6 | ⛔ **`Queue::send` has no entry cap** | `grep -n "max-entries" wat-scripts/queue/sqs.wat` | none. `Topic::publish` **keeps** `[msgs 10]` |
| 7 | ⛔ **the topic reports whole messages** | probe with `nsubs 4`, inbox room for 6 pairs | `Accepted 1` — floor(6/4), the partial tail message is NOT counted |
| 8 | ⛔ **the nsubs cliff is gone** | probe at `nsubs 7`, publish 10 | **no assertion**; a count comes back and the service is alive |
| 9 | ⛔ **delivery still exact** | `circuit.wat` ×5 | `total=8000; distinct=8000` every run |
| 10 | ⛔ **no `:cap` changed** | `grep -o ":cap [0-9]*"` both files | 10× `:cap 64`, 2× `:cap 2`, 2× `:cap 1`, 1× `:cap 32`, `:cap 1024` — as today |
| 11 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ count reported |
| 12 | **blast radius** | `git status --porcelain` | `sqs.wat`, `sns-fanout.wat`, `circuit.wat`, probes, the SCORE. **No `wat/`, no `src/`** |

## REPORTS — recorded, not gated

| ▪ | what | why |
|---|---|---|
| a | ★★ **`full-retries`** | the number this stone exists to move. Prior median **3770**. Whole batches bouncing off a queue with room is the waste being removed |
| b | ★ `publish + drain` median ×5 | prior: publish 22583, publish+drain ~22783 |
| c | **`dup`** | may become non-zero — see the note above. Report it; do not gate it |
| d | the new dominant term with numbers | rule 4 |
| e | `setup` / `stop` | untouched; the next target |

## RUNTIME

60–90 min. The admission arithmetic is small; the driver's resend-from-`count` and the topic's
floor division are the real content, and there are two publish loops.

## TRAP DOORS

- ⚠⚠ **`Accepted 0` must not be conflated with an error.** It is the 429 — retry unchanged. If the
  driver treats it as failure the circuit will assert instead of waiting.
- ⚠⚠ **The prefix must be the FIRST `take`, not an arbitrary subset.** Row 2 reads the stored
  bodies rather than trusting the count, because a count alone cannot tell a prefix from a
  scramble, and the caller's resend-from-`count` is wrong if it is a scramble.
- ⚠ **`room` can go negative** if depth already exceeds cap (a redelivery can push it). Clamp at 0;
  a negative `take` would build rows from a negative-length prefix.
- ⚠ **Removing `:max-entries` from `Queue::send` makes an existing probe stale.** Retarget it to
  `Topic/publish`, do not delete the coverage.
- ⚠ **`distinct=8000` is the real safety net here.** Partial admission plus resend is exactly the
  shape that duplicates or drops messages; row 9 is what catches a resend-offset bug that the
  timings would happily hide.
