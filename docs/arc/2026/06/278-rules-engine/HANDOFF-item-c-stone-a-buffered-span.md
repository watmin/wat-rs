# HANDOFF — item (c) stone A: the buffered span

You are making `span'` buffer its logs and emit metrics as **deltas**, so a flush can happen more
than once without double-counting. **No timer in this stone.**

Start here, in order:

1. `DESIGN-STONE-the-buffered-span.md` — why there is no new service and no bracket, and the one
   contract decision.
2. `BRIEF-item-c-stone-a-buffered-span.md` — the rooms as exact `file:line`, the four STOP triggers.
3. `wat/telemetry/span.wat` — `incr` is the exemplar for the new `log`; `close` is the flush you are
   extracting.

The thing that carries all the risk: **`close` stops meaning "emit everything" and becomes "flush
the remainder."** A flush emits what accumulated since the last one and RESETS. If `close` and a
mid-life flush ever become two different emission paths, a span that flushes once will double-count
on close — and nothing goes red; a dashboard is just wrong six months later. One path, one reset.

Second: the span already holds **full fidelity** (`Samples` is `(Vector :- [i64])`) and `close`
throws it away. Keep `<name>/count` and `<name>/duration` exactly as they are and ADD
`<name>/sample` per sample. Both, not either — they answer different questions, and count+sum cannot
be un-aggregated later.

Third: the size threshold is **read from the op's declared `:max-request-bytes`**, never a literal.
If it is not reachable from the arm, STOP and say what is missing.

One thing you will notice and should NOT fix here: `LogResponse::Ok` now means "buffered", not
"written". That is a real weakening of a promise and a contract question the builder has not ruled
on. Note it in the SCORE; leave `Ok` alone.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-item-c-stone-a-buffered-span.md` when done. It will be graded by re-running.
