# EXPECTATIONS — the trace separates durable work from waiting

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE. ▪ = REPORT.

★ This stone produces a number; it does not chase one. Its gates are that the trace becomes honest
and that measuring does not perturb.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ★★ **the phantom is gone** | `grep ":t2 t1" wat-scripts/topic/sns-fanout.wat` | empty. `hop12` no longer exists in `Traces` |
| 2 | ★★ **the split is real, not a relabel** | the two new histograms | `publish-work` + `inbox-wait` ≈ today's `outbox` per bucket; `fanout-work` + `subq-wait` ≈ today's `t3->t4`. **If either new stage is all-zero, it is a second phantom** |
| 3 | ★★ **`t0b` is stamped by the TOPIC** | read the impl | inside `publish`, after `send-all` returns. A publisher-side stamp folds the reply hop into "durable work" and would answer the question wrongly |
| 4 | ⛔⛔ **measuring does not perturb** | `circuit.wat` ×3 | `publish` ~20700 ±300, `collect` ~5900, `stop` ~400. Telemetry that moves the wall has invalidated itself |
| 5 | ⛔ **delivery exact** | ×3 | `total=8000; distinct=8000; dup=0` |
| 6 | ⛔ **e2e still spans the whole path** | the histogram | `t0 -> t4`, unchanged shape (~50-250 dominant, a 250-1000 tail) |
| 7 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ count reported |
| 8 | ⛔ **blast radius** | `git status --porcelain` | `sns-fanout.wat`, `circuit.wat`, probes, SCORE. **No `wat/`, no `src/`, no `sqs.wat`** |

## REPORTS

| ▪ | what |
|---|---|
| a | ★★★ **durable work as a fraction of each hop** — the number this stone exists for |
| b | all five histograms plus `e2e` |
| c | the phase line, to confirm row 4 |
| d | body length with seven stamps, against the 524288-byte declared cap |

## RUNTIME

30–50 min. Two stamps, a part-count change, and two histogram fields.

## TRAP DOORS

- ⚠⚠ **Row 2 is the whole point.** A split where one side is always zero has produced a second
  phantom — which is exactly the defect being removed. Check both new histograms have mass before
  believing either.
- ⚠⚠ **Row 4 is the integrity gate.** Longer bodies mean more bytes per message through every
  store write and every wire hop. If the wall moves, the instrument is changing the system and its
  numbers cannot be trusted — report it rather than absorbing it.
- ⚠ **`t0b` on the publisher side would be wrong and would look right.** It would produce a
  plausible "durable work" number that silently includes the reply hop. Row 3 gates the location,
  not the value.
- ⚠ **Do not adopt `wat/telemetry.wat` here.** A real Span/Journal system exists and the circuit
  hand-rolls stamps; swapping mechanism mid-investigation would make this measurement
  incomparable with every prior one.
