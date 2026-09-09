# EXPECTATIONS — the benchmark has more than one publisher

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE. ▪ = REPORT.

★ **Every timing here is a REPORT.** This stone builds an instrument; it changes no system code.
Gating a number produced by a new instrument would be gating the thing the instrument is supposed
to measure.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ⛔⛔ **p=1 reproduces today** | `circuit.wat` ×3 at p=1 | publish ~20379, retries ~1386, receives ~5032, `distinct=8000`. **Within run-to-run spread (~±300 ms).** If not, the instrument changed the measurement and six stones' baselines are void |
| 2 | ★★ **P publishers are concurrent** | read the parent; and the wall at p=3 | all P started before any is joined. p=3 wall is **not** ~3× the p=1 wall — that would be sequential publishers wearing a parameter |
| 3 | ★★ **each publisher has its own seed** | read the record construction; probe the derived seeds | P distinct seeds. A shared seed is a synthetic herd and voids the experiment |
| 4 | ⛔ **the id space tiles** | `distinct` at p=1 and p=3 | `distinct=8000` at both — no collision, no dropped share |
| 5 | ⛔ **counts aggregate** | `publish-calls`, `full-retries` at p=3 | sums across publishers, not one publisher's view |
| 6 | ⛔ **no system code changed** | `git status --porcelain` | `circuit.wat`, scratch probes, the SCORE. **No `sqs.wat`, no `sns-fanout.wat`, no `wat/`, no `src/`** |
| 7 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ count reported |

## REPORTS — the six cells

| ▪ | policy | p=1 (known) | p=3 |
|---|---|---|---|
| a | adaptive (shipped) | 20379 | — |
| b | fixed 25 ms | 18472 | — |
| c | fixed 1 ms | 22395 | — |

Plus `full-retries`, `queue-receive-calls`, `distinct` for every cell, and `setup`/`stop`.

## RUNTIME

60–90 min. The `Publisher` service mirrors `Worker` closely; the cost is the join and eighteen
circuit runs.

## TRAP DOORS

- ⚠⚠⚠ **Row 1 is the whole stone.** A harness that shifts the p=1 numbers invalidates every
  comparison this arc has made. Run p=1 **first**, before anything else is measured.
- ⚠⚠ **"P publishers" that the parent starts and joins one at a time is not P publishers.** Row 2
  exists because that mistake produces plausible numbers and answers nothing.
- ⚠ **A shared seed looks like it works.** Three publishers with identical delay sequences will
  produce a clean, reproducible, completely meaningless result — it manufactures the herd instead
  of testing for it.
- ⚠ **Do not build a policy knob.** Editing one expression per measurement is tedious and correct;
  a flag is a new surface in a stone that is already the harness.
- ⚠⚠ **STOP-4 is the outcome I most want to hear if it happens.** If fixed 25 ms does *not*
  degrade at p=3, jitter is unjustified in this system and my correction of the last SCORE was
  itself premature. Report it plainly; I have been wrong five times today and would rather be
  wrong a sixth in writing than have it hidden.
