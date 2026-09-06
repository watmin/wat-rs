# EXPECTATIONS — partial only when it must

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE: an invariant this stone controls. ▪ = REPORT.

★ **`queue-receive-calls`, `publish` and `dup` are REPORTS, not gates** — even though they are the
whole point. This stone controls the *admission rule*; those numbers are what the rule causes.
Gating a consequence has fired wrongly four times in this arc, and I said in the last message I
would gate on `receive-calls` — that would have been the fifth. The rule is gated; the numbers are
reported, and the SCORE's headline is `receive-calls`.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ★★ **at or below cap is all-or-nothing** | probe `cap 10`, depth 6, send 8 | **`Accepted 0`**, depth unchanged — the updated case |
| 2 | ★★ **it lands whole once there is room** | same probe: drain 5 (depth 1), send 8 | `Accepted 8`, depth 9 |
| 3 | ★★ **above cap still takes a prefix** | probe `nsubs 7`, publish 10 (70 bodies, cap 64) | a count comes back, **no assertion**, service alive |
| 4 | ⛔ **the prefix is still a prefix** | above-cap case, read the stored bodies | first `room` bodies, in order |
| 5 | ⛔ **no response leaks internals** | `grep` the response enums | no `depth`/`cap` on an admission arm; `:Full` still gone |
| 6 | ⛔ **delivery still exact** | `circuit.wat` ×5 | `distinct=8000` every run |
| 7 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ count reported |
| 8 | ⛔ **blast radius** | `git status --porcelain` | `sqs.wat`, scratch probes, the SCORE. **No `sns-fanout.wat`, no `circuit.wat`, no `wat/`, no `src/`** |
| 9 | ⛔ **no `:cap` changed** | `grep -o ":cap [0-9]*"` both files | as today |

## REPORTS — recorded, not gated

| ▪ | what | now → pre-regression |
|---|---|---|
| a | ★★★ **`queue-receive-calls`** | **11859** → 5396. The headline |
| b | ★★ `dup` | 2759 → 0 |
| c | ★★ `publish` median ×5 | 70244 → 22583 |
| d | `full-retries` | 7900 → 3770 |
| e | `outbox` histogram | 10759 items, `50-250=2992 250-1000=7804 max 662` → 8000 items, `50-250=7966 250-1000=0 max 209` |
| f | `setup` / `stop` | 10232 / 9742 — next, after publish |

## RUNTIME

20–35 min. Three lines plus a probe update; the work is the measurement.

## TRAP DOORS

- ⚠⚠ **The probe case that changes is not a regression.** `cap 10`, depth 6, send 8 → `Accepted 0`
  is the new rule working. The temptation is to "preserve" `Accepted 4` and thereby preserve the
  fragmentation. Row 1 gates the NEW value deliberately.
- ⚠⚠ **`n0 > cap` vs `n0 > room`.** The rule keys on `cap` (can this EVER fit) not `room` (does it
  fit right now). Keying on `room` reproduces today's behaviour exactly and would pass a careless
  reading of row 2.
- ⚠ **`room` must be clamped before use.** A redelivery can push `depth` past `cap`, and a negative
  `room` would build rows from a negative-length prefix.
- ⚠⚠ **Five models for this slowdown have already been wrong.** If `receive-calls` falls and
  `publish` does not follow, STOP-3 says report and stop. That refutes the operation-count model
  and matters more than this stone's own result.
