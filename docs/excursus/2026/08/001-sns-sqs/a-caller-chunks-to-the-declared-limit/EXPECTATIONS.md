# EXPECTATIONS — a caller chunks to the declared limit

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE. ▪ = REPORT.

★ The 3.3× is the **reason** for the stone and is still a REPORT. What the stone controls is the
emission condition, the chunk contract, and the wall staying a wall. Gating the speedup would be
the fifth consequence-gate in this arc.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ★★ **the sum is a true PREFIX** | probe: limit 4, send 10 into a queue with room for 6 | `Accepted 6`, and the **stored bodies are the first 6 in order** — read them, do not trust the count |
| 2 | ★★ **it stops at the first short chunk** | same probe, read the store | **no body from chunk 3 is present.** Sending past a short chunk breaks contiguity and is the defect this row exists for |
| 3 | ★★ **one chunk when it fits** | probe: limit 64, send 40 | exactly **one** `Queue/send` — verifiable via the queue's `receive-calls`/tick counters or a wrapped peer |
| 4 | ⛔ **the wall stays a wall** | call plain `Queue/send` with 65 bodies | `RequestTooManyEntries(65,64)`, nothing enqueued. **`send` did not become lenient** |
| 5 | ⛔ **emission condition holds both ways** | `grep` the expansion for `-all` methods | `Queue/send-all` and `Topic/publish-all` exist; **no `Store/put-all`, no `Store/delete-all`** — those responses carry `:Success []`, not `Accepted [count]` |
| 6 | ⛔ **the caller does not know the limit** | `grep -n "MAX-ENTRIES" wat-scripts/topic/sns-fanout.wat` | **empty.** The topic must not read the queue's def |
| 7 | ⛔ **types are interchangeable** | read the two signatures | `send-all` returns the same `RecvOutcome`-of-`SendResponse` as `send` |
| 8 | ⛔ **delivery still exact** | `circuit.wat` ×3 at m=4 **and** m=8 | `distinct=8000` in both configurations |
| 9 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ count reported |
| 10 | ⛔ **blast radius** | `git status --porcelain` | `service.wat`, `sqs.wat`, `sns-fanout.wat`, probes, the SCORE. **No `src/`, no `circuit.wat`** |
| 11 | ⛔ **no `:cap` changed** | `grep -o ":cap [0-9]*"` both files | as today |

## REPORTS

| ▪ | what | today |
|---|---|---|
| a | ★★★ **m=8 publish** | **61431** (retries 1652, receives 10387) — the 3.3× |
| b | ★★ **m=4 publish** | **18472** (retries 540, receives 4687) — must not move |
| c | chunks per publish at each m | 40/64 = 1 · 80/64 = 2 |
| d | `setup` / `stop` | 10205 / 6001 |

## RUNTIME

60–90 min. The emission condition and the ordered chunk-fold are the content; the surface and
topic edits are small.

## TRAP DOORS

- ⚠⚠ **Row 2 is the one that will be got wrong.** A chunk loop that keeps going after a short
  chunk produces a *larger* number and looks better. It is wrong: `Accepted n` promises the first
  `n`, contiguously. Read the stored bodies; a count cannot tell a prefix from a scatter.
- ⚠⚠ **Row 4 is the DoS wall.** If `send` starts accepting 65 because the chunker exists, the
  declared limit protects nothing and we have re-created the thing this arc keeps closing.
- ⚠ **Row 3 guards the m=4 case.** 40 bodies against a 64 limit must be *one* call — if the
  chunker splits unconditionally we pay 4× the operations, and operations are the measured cost.
- ⚠ **Pin BOTH delay sites or neither.** Patching only `circuit.wat`'s parent silently measures
  adaptive; that produced a false refutation for me earlier today.
- ⚠⚠ **`Store::put` must not acquire a chunker.** Its response is `:Success []` — all-or-nothing,
  proven this session (a partial store write is unreachable). A `put-all` would be a chunker over
  a contract that cannot express a partial result.
