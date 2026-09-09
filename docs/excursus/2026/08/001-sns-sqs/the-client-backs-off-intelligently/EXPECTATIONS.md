# EXPECTATIONS — the client backs off intelligently

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE: an invariant this stone controls. ▪ = REPORT.

★ **`publish` is a REPORT.** It is the point of the stone and it is still a consequence — the
stone controls the *policy*, not the wall. Gating it would be the fifth consequence-gate in this
arc. The policy is gated; `publish` is the SCORE's headline number and STOP-3 governs what to say
about it.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ★★ **the delay is bounded and never zero** | probe `backoff-delay` over attempts 0..12, many seeds | every draw in `[1, min(100, 1<<attempt)]`; **never 0**, never above 100 |
| 2 | ★★ **it is a DRAW, not a fixed schedule** | same probe, many seeds at one attempt | distinct values across seeds — full jitter, not plain exponential wearing jitter's name |
| 3 | ★★ **attempt resets on ANY acceptance** | read the loop | a partial `Accepted c` with `c > 0` resets to 0, not just a full one. A client that keeps backing off while making progress is the bug this row exists for |
| 4 | ⛔ **the seed is threaded** | read the loop; run the circuit twice with a fixed seed | `int-from`'s `state'` is carried; the run is reproducible |
| 5 | ⛔ **delivery unaffected** | `circuit.wat` ×5 | `total=8000; distinct=8000; dup=0` every run |
| 6 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ count reported |
| 7 | ⛔ **blast radius** | `git status --porcelain` | `circuit.wat`, scratch probes, the SCORE. **No `sqs.wat`, no `sns-fanout.wat`, no `wat/`, no `src/`** |
| 8 | ⛔ **the constants are readable bounds** | `grep` for them | `BACKOFF-BASE-MS` / `BACKOFF-CAP-MS` exist as named defs, not literals buried in the loop |

## REPORTS — recorded, not gated

| ▪ | what | baselines, all measured this session |
|---|---|---|
| a | ★★★ `publish` median ×5 | fixed 1 ms **22395** · fixed 25 ms **18472** · spin **22794** |
| b | ★★ `full-retries` | 3807 · 540 · 4088 |
| c | `queue-receive-calls` | 5290 · 4687 · 5313 |
| d | the realised delay distribution | how much of the window the publisher actually slept |
| e | `setup` / `stop` | 10252 / 6001 — next, after publish |

## RUNTIME

30–50 min. The loop already threads `attempts`; adding a seed and computing the delay is small.
The work is the probe for rows 1–2 and the five-run measurement.

## TRAP DOORS

- ⚠⚠ **STOP-3 is the row I care about.** If this ties 18472 it is *more principled and no faster*,
  and that is what the SCORE must say. Tuning `BASE`/`CAP` until it wins would re-create the magic
  value with extra steps and I would grade it as a failed strike.
- ⚠⚠ **Reset on PARTIAL acceptance, not just full.** `Accepted c` with `0 < c < n` is progress. A
  loop that only resets on `c == n` will back off to the cap while it is still getting work done —
  slower than today, and it would look like the design failing rather than the implementation.
- ⚠ **`lo = 1`, not 0.** `NonZeroDuration` forbids zero, and the spin measurement says zero is
  worse anyway. A draw of 0 would not even construct.
- ⚠ **Do not let the seed come from the clock.** `int-from` is threaded precisely so the benchmark
  stays reproducible; seeding from `now()` would make every run a different experiment.
- ⚠ **Five models for this system's cost have already been wrong today.** If `publish` moves in a
  direction the DESIGN did not predict, that is a finding to report, not a number to chase.
