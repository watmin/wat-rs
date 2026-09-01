# HANDOFF — perf 1: the incremental byte measure

`log`, `incr` and `timed` each decide their size trigger by encoding the **whole would-be batch**.
Measured: **1.85 ms per `log` call** at a 1000-entry buffer, and superlinear — 224 / 618 / 1848 ms
for 250 / 500 / 1000 logs, a ratio climbing 2.76× → 2.99× per doubling.

Start here, in order:

1. `DESIGN-STONE-the-incremental-byte-measure.md` — why this is wrong in the design's own terms, and
   the contract decision about the *direction* of any residual error.
2. `BRIEF-perf-1-incremental-byte-measure.md` — the rooms as exact `file:line`, four STOP triggers.
3. `wat-scripts/scratch-pad/probe-span-log-cost.wat` — the measurement. **Re-run it before and
   after**; its numbers are the stone's evidence and belong in the SCORE.

Three things to hold:

**The justification was for flush time, not per call.** The io-budgets design said *"exact
`edn::write` length beats an estimate (the encode is needed anyway)"* — and it is, ONCE, at flush.
Every log currently pays to re-encode every log before it.

**★ Never under-count.** A sum of per-item encodings is not identically the encoding of the whole
request: there is container framing. The running total must be exact, or conservatively HIGH — an
over-count costs an occasional early flush, an under-count ships an over-cap batch that the server
refuses. Those are not symmetric. Say in the SCORE which you achieved.

**The total must survive a partial drain.** Item (b) can leave an un-written suffix; the total must
then match that suffix, not the batch that was attempted. A drifting total is worse than a slow one,
because it is silent until a write is refused.

Do not touch `write-*-batched`. It measures at flush time, where the encode is genuinely needed.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-perf-1-incremental-byte-measure.md` when done. It will be graded by re-running.
