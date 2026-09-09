# BRIEF — the slope belongs to a phase

## The work, in one paragraph

Run the circuit at **n=1000, 2000 and 4000**, at least **three times each**, with `vis-ms=1000` pinned and
everything else held fixed, on a quiet box, one run at a time. Report every phase for every run, then
medians with their observed spread, then the doubling ratios. **Modify no file.** The SCORE is the entire
deliverable.

## Why this measurement and not another

Eleven mechanisms died in this arc chasing *"the drain is superlinear"* — a figure taken from `n=500→1000`,
from single runs, before `vis` was a parameter. Then the drain turned out to be **6.4 %** of the run,
`fill` **45 %**, `setup` **33 %**, and **no phase's slope has ever been measured.**

## Read in order

1. **`DESIGN.md`** — why `vis` must be
   pinned, why `setup` is measured but not targeted, and what each outcome means.
2. **`docs/excursus/2026/08/001-sns-sqs/vis-is-swept-not-chosen/SCORE-v2.md`** — the sweep table's shape,
   and the measured fact that `drain ≈ max(work, vis)`.
3. **`docs/excursus/2026/08/001-sns-sqs/the-summary-counts-with-a-set/SCORE.md`** — the most recent
   baseline numbers at n=2000, and an example of reporting a term measured in isolation rather than hunted
   inside variance.
4. **`wat-scripts/fanout/circuit.wat:2328`** — `poll-until-drained … (:wat::i64::* n m)`. The attempt bound
   is `n × m`, which is why `vis=1000` should remain safely inside the patience at n=4000.

## The runs

```
./target/release/wat wat-scripts/fanout/circuit.wat <n> 4 3 8192 true 1000
   n ∈ {1000, 2000, 4000},  ≥3 runs each,  one at a time
```

Before **every** run: `ps -eo args | grep -E 'cargo|nextest|release/wat' | grep -v grep` and
`cut -d' ' -f1 /proc/loadavg`. Record the load in the SCORE beside each run.

Capture per run: `setup fill arm drain collect stop total`, wall clock, `distinct`, `dup`, `workers`,
`seen-skipped`, and the per-tier line. Then compute per phase per n: **median and min–max**, and the
ratios `n2000/n1000` and `n4000/n2000`.

## Blast radius

**None.** No file is modified. `git status --porcelain` must show only the new SCORE as untracked, with
**zero `M` lines** — that is row 8, and it is how I know the numbers came from the tree on record.

## STOP triggers

**STOP-1** — if any run reports `distinct ≠ n×m` or `dup ≠ 0`, that run is **not a timing sample**. Report
it in full with its counters and per-tier line, and do not average it into a median.

**STOP-2** — if n=4000 does not complete, **STOP and report the counters and the per-tier line.** That is a
finding about the `n×m` attempt bound, not a failed errand. Do **not** raise `vis` to force completion —
that would change the variable under study.

**STOP-3** — do **not** modify any file, including "harmless" ones. If a measurement seems to need a code
change, **STOP and say what and why**; that is a finding about the instrument.

**STOP-4** — do **not** report a ratio without its spread. If three runs of a phase span more than the
difference you are reporting, **say so explicitly** rather than presenting the median as the result.

**STOP-5** — do **not** run the floor beside a measurement, and do not run two measurements at once. Two
floors have gone red this session from concurrent load alone.

**STOP-6** — on any red floor arm: capture it whole, name the exact arm, **do not re-run it.**

## What "done" looks like

A table with **at least nine measurement rows** (three n × three runs), every phase present per row, then a
medians-with-spread table, then a ratios table. `distinct=n×m` and `dup=0` on every counted run. Load
stated per run. `git status --porcelain` showing zero modified files. Floor Summary read from
`scripts/floor.sh`, run **after** all measurements: 5237 / 22 skipped / 0 FAIL / 0 TIMEOUT.

**Write the SCORE to
`SCORE.md`** in the shape of the
neighbouring SCORE files. **Do not commit.**

State plainly which phase, if any, is superlinear — **and if none is, say that.** *"Every phase is linear
and the original +55 % was an artifact"* is a complete and valuable answer, not a null result.
