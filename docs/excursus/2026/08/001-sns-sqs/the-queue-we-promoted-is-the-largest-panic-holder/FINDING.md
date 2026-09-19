# The queue we just promoted is the largest panic-holder in the stdlib

**Measured 2026-09-19, HEAD `0d8bada7b`** (queue promoted at `6e6bb2464`, bracket relocated at
`0d8bada7b`). Census only — nothing changed.

## The census

`assertion-failed!` occurrences in stdlib `.wat`, **comments stripped** (an un-stripped grep counts
prose; `wat/*.wat` does not recurse, so this walks `find wat -name '*.wat'` — 55 files):

| file | sites |
|---|---:|
| ⛔ **`wat/queue.wat`** | **78** |
| `wat/service.wat` | 65 |
| `wat/kernel/services/stdio.wat` | 38 |
| `wat/bracket.wat` | 30 |
| `wat/telemetry/journal.wat` | 17 |
| `wat/spawn.wat` | 12 |
| `wat/telemetry/span.wat` · `wat/fix.wat` | 10 each |
| `wat/test.wat` | 9 |
| `wat/core.wat` | 7 |
| `wat/query.wat` | 5 |
| 6 more files | 10 |
| **total** | **291** |

⛔ **The file we promoted one commit ago is the biggest single holder.** The parked promotion
correctly refused the 13 "CLIENT API" panic-gate *helpers* — *"promoting a panicking gate to
manifest position 50 would have enshrined the exact ungraceful failure this excursus is a crusade
against"* — but the 78 sites inside `queue.wat` itself came with it.

⚠ **This is a count, not a verdict.** A defensive assert on a genuinely unreachable state is not the
same defect as a momentary transport failure treated as fatal. Which of the 78 are which is
**unmeasured**, and is the next stone — not a conclusion to draw from this table.

## The state of `a-momentary-failure-is-not-fatal` (drawn 2026-09-10, never struck)

That DESIGN is the governing one for this class. Its stones have landed unevenly:

| stone | state |
|---|---|
| **1a** — `RecvOutcome` gains `Malformed [cause]` | ✅ **landed** — `bracket.wat` matches `(:wat::kernel::RecvOutcome::Malformed _cause)` at 5 sites |
| ⛔ **1b** — the failure leaves `<S>::Reply` | ❌ **NOT landed.** `RESERVED_FAILURE_VARIANT = "Failed"` is still synthesized (`src/types.rs:3941`), and `:1941`'s own comment still calls it *"a transport fact wearing an op type"* |
| **2** — owner methods stop raising | ✅ partially — `:wat::service::StopOutcome` exists and is matched in `service.wat` |
| **the gateable invariant** | ❌ never built |

⚠ **And the number that gate would hold has grown.** The DESIGN recorded *"`wat/service.wat` holds
43 `assertion-failed!` calls today"* on 2026-09-10. It holds **65** now — **+51%** while the crusade
was running. Not an indictment of any one stone; a statement that the debt accrues faster than it is
paid, which is an argument for the gate rather than for more hand-repair.

## The five self-labelled placeholders

`bracket.wat` carries its own instruction, five times over (lines 54, 100, 154, 222, 508):

> `"recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED
> PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)"`

and five more panic on `RecvOutcome::TimedOut` with the message *"recv: timed out — the peer is
alive and silent"* — a momentary failure, named as momentary in its own panic string, killing the
process. That is the DESIGN's central complaint verbatim: **"The code states that nothing died and
then kills the process."**

## Why this lands here rather than as a repair

Step 3 of the current work list is *"rewrite bracket's transport onto the queue."* This census says
that framing is too small. Bracket's ungraceful arms are not a bracket defect — they are the
**unstruck remainder of a substrate DESIGN**, and they are waiting on it by name. Rewriting
bracket's transport before 1b and the gate land would move 30 panic sites onto a queue that carries
78 of its own.

**Order that follows from the measurement:** classify the panic sites by the DESIGN's three
dispositions (RETRY / REPORT-FINAL / REPORT-GONE), starting with `queue.wat` because it is the
largest and the newest; then 1b; then the gate; and bracket's transport falls out of those rather
than being rewritten against them.

## Method notes

- Comments stripped before counting, and `find` used rather than the `wat/*.wat` glob — that glob
  covers 27 of 55 files and has produced two wrong counts in this excursus already.
- `grep -c` counts LINES; `grep -o | wc -l` counts occurrences. This table is occurrences.
