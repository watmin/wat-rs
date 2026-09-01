# EXPECTATIONS — item (b): the batched writer

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ **an over-cap buffer drains** | buffer > cap, working sink, flush | every item written across multiple submissions. **RED today** — the finding this stone closes |
| 2 | ★ **partial progress is exact** | sink accepts chunk 1, refuses chunk 2 | the buffer afterwards holds **exactly** the un-written suffix. One more = duplicate on the next flush; one fewer = silent loss |
| 3 | one item over the cap | a single item whose encoding alone exceeds the cap | `RequestTooLarge{bytes, cap}`, and the flush **returns** — a hang is a FAIL worse than the failure it reports |
| 4 | cut at `>` not `>=` | a chunk sized to exactly the cap | is SENT, not split further (STOP-3) |
| 5 | under-cap path unchanged | a small buffer | exactly ONE write, as today |
| 6 | stone C still speaks | failing sink past the cap | the failure still reaches the caller, never `Ok` |
| 7 | stones A/B hold | `cargo nextest run --release -E 'test(probe_arc278_span)'` | all pass; **assertions unedited** (mechanical construction edits are fine) |
| 8 | no `Stream`, no `WriteResult` | `grep -rn 'Stream\|WriteResult' wat/telemetry.wat` | absent (STOP-2) |
| 9 | cap from the contract | `git diff` | `WRITE-{LOGS,METRICS}-MAX-REQUEST-BYTES`; a literal is a FAIL |
| 10 | no new surface op | Span + Journal op counts | 5 and 6, unchanged |
| 11 | no runtime change | `git diff --stat src/runtime.rs` | empty |
| 12 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 5154+ run, 0 failed, FLOOR=0 |

**Runtime prediction:** 60–120 minutes. The fold is small; the exact-suffix accounting and its gate
carry the cost.

## Trap doors, named in advance

- **An off-by-one in the written count.** Silent in both directions — duplicates or losses, neither
  of which any other row sees. Row 2 is the only guard and it is the row I would not ship without.
- **Copying the span's `>=`** into the chunker — the unflushable buffer, one scale down. Row 4.
- **Looping on a single over-cap item** — an empty chunk, round again, forever. Row 3.
- **Building `Stream`** because the DESIGN's equivalence mentions it. Row 8.
- **Firing on nothing:** rows 3–12 all pass if the writer just calls `write-*` once with everything.
  Only rows 1 and 2 catch it.
