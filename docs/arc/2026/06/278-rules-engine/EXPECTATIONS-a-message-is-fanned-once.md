# EXPECTATIONS — a message is fanned once

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE: an invariant this stone controls. ▪ = REPORT.

★ **`dup` is the point of this stone and it is still a REPORT, not a gate.** What the stone
controls is that a message the topic *counts* as accepted is *fully fanned*; how many duplicates
that removes depends on how often the boundary splits, which is traffic, not invariant. Gating a
consequence has fired wrongly four times in this arc.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ★★ **a counted message is fully fanned** | probe: `nsubs 4`, inbox room for 6 pairs, publish 2 | if it reports `Accepted 1`, **exactly 4 pairs** of message 0 are in the store — never 5, 6 or 7 |
| 2 | ★★ **the top-up carries identical bodies** | read the stored bodies | the pairs from the top-up are byte-identical to what the first send would have written — same `"{i}\|{msg}"` |
| 3 | ⛔ **at most ONE extra send per publish** | read the impl | no loop. STOP-2 exists because an unbounded internal retry in a service arm is a hang |
| 4 | ⛔ **a refused top-up degrades to today** | probe: room for 6, top-up refused | `Accepted 1` (floor), service alive, no assertion |
| 5 | ⛔ **delivery still exact** | `circuit.wat` ×5 | `distinct=8000` every run |
| 6 | ⛔ **the nsubs cliff stays gone** | probe `nsubs 7`, publish 10 | a count comes back, no assertion |
| 7 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ count reported |
| 8 | ⛔ **blast radius** | `git status --porcelain` | `sns-fanout.wat`, scratch probes, the SCORE. **No `circuit.wat`, no `sqs.wat`, no `wat/`, no `src/`** |

## REPORTS — recorded, not gated

| ▪ | what | prior |
|---|---|---|
| a | ★★ **`dup`** | median **4226**. The number this stone aims at |
| b | ★★ **the `outbox` histogram** | `50-250=4982 250-1000=7148 max 562ms`, 12161 items. The DESIGN says this and `dup` move together |
| c | ★ `publish` median ×5 | **61944**. Before the regression: 22583 |
| d | `seen-skipped` | median 13594 |
| e | `setup` / `stop` | 10245 / 11690 — the next target, after publish |

## RUNTIME

30–50 min. One arm of one file; the content is reconstructing which subscriber indices are missing
and sending exactly those.

## TRAP DOORS

- ⚠⚠ **`k mod nsubs` is only meaningful because the fanout is msg-major and the queue admits a
  PREFIX.** Both were verified this session (`stored=f0..f5,m0,m1,m2,m3` read from the store, not
  inferred from a count). If either changes, this design is wrong — STOP-1.
- ⚠ **A top-up that itself splits.** `need` is at most `nsubs-1`, so it is small, but "small" is
  not "atomic". STOP-2 says report, do not loop.
- ⚠ **Do not touch the driver.** Making re-publishes rarer is this stone; changing what a
  re-publish *does* is a different one, and mixing them makes the measurement unreadable.
- ⚠⚠ **I have been wrong four times today about why publish is slow** — the busy-wait, the
  per-call model, the closure allocation, the unused-arm mechanism. If `dup` falls and `publish`
  does not move, **report it and stop**; that refutes the drain-bound model and is worth more than
  this stone's own result.
