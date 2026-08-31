# EXPECTATIONS — excursus 001 stone 6

**Written BEFORE the strike, 2026-08-31.** Blast radius derived from the BRIEF's own section.

## The scorecard

| # | what | expected |
|---|---|---|
| 1 | `sqs.wat` freezes | `--check` = **0** (it is `1` today, correctly) |
| 2 | ★ the summary is byte-identical | `"bound=x;r1=a,b;r2=c;r3=;redel=b"` — a move that changes behaviour is not a move |
| 3 | no rename | `:queue::Envelope` keeps its name; zero call-site edits |
| 4 | blast radius | `wat-scripts/queue/sqs.wat` only, plus the SCORE |
| 5 | `probe_ex001_queue` passes | the lifecycle test goes green |
| 6 | the loader gate passes | `every_wat_scripts_file_loads` — 528 files, 0 failures |
| 7 | floor | **`FLOOR=0`** |
| 8 | the guard still fires on the repros | both `repro/*.wat` still `--check = 1` — the fix to the queue must not weaken stone 5 |
| 9 | prior stones | topic `"3 3"`; all `probe_ex001_*` PASS |

## Runtime prediction

**15–30 minutes.** One block moved. Almost all of it is the ~1m20s build and ~5m floor.

## Trap-doors

1. **★ Row 8 is the one that could be missed.** The obvious way to make the queue freeze again
   is to move `Envelope` — but a *wrong* way is to weaken the guard. Both reproductions must
   still fail. If either goes to `0`, stone 5 was undone to make stone 6 pass.
2. **Row 2 is the move's proof.** The grep precedent: *"the counts are the proof it moved
   intact."* Re-run it; do not take it from a report.
3. **The circuit will start freezing again** (it `load-file!`s the queue). Its foreign-read
   workaround is now unnecessary and possibly wrong, but nothing in the floor runs it. That is
   stone 7's, not this one's — **note what you observe, change nothing.**
4. **Ordering inside `:messages` should not matter**, but `Envelope` is referenced by
   `ReceiveResponse` which is also in the vector. If placement turns out to matter, that is a
   finding about `defsurface`, not a detail to tune silently.

## Not in this stone

- `wat-scripts/fanout/circuit.wat` and the workaround inside it — stone 7.
- Re-attempting the fan-out proof — stone 7.
- Any change to `src/types/surface.rs` — stone 5 is done.
