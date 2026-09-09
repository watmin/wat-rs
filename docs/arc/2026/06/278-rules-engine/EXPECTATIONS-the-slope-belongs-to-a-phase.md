# EXPECTATIONS — the slope belongs to a phase

Written **before** the strike. **Measurement only — no file is modified.** The SCORE is the deliverable.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **three points, ≥3 runs each** | the SCORE's table | n=1000, 2000, 4000 × ≥3 runs. **Nine rows minimum, none merged, none skipped** |
| 2 | ★ **every phase reported per run** | same | `setup fill arm drain collect stop total` and wall clock, per run — not medians only |
| 3 | ★ **medians AND spread** | same | per phase per n: median and observed min–max. **A ratio without a spread beside it is not a measurement** |
| 4 | ★ **the ratios are stated** | same | per phase: `n2000/n1000` and `n4000/n2000`. ≈2× is linear; >2× is the finding |
| 5 | ★ **`vis-ms=1000` at every point** | the command lines, quoted | pinned. An unpinned `vis` makes the drain `max(work, vis)` and voids row 4 for that phase |
| 6 | **correctness at every point** | same | `distinct = n×m`, `dup=0`, per run. A run that lost or duplicated messages is not a timing sample |
| 7 | **the box was quiet for every run** | same | load stated per run; never two things at once |
| 8 | **no file was modified** | `git status --porcelain` | **only the new SCORE is untracked.** Zero `M` lines |
| 9 | **the floor still holds** | `scripts/floor.sh` | **read the Summary line**: 5237 passed, 22 skipped, 0 FAIL, 0 TIMEOUT. Run it **after** the measurements, never beside them |

## The rows that carry it

★★ **Row 3 is the stone.** The `+55 %` claim that cost this arc eleven mechanisms came from single runs.
`fill` alone spans **15493–17333 ms** across today's measurements and `collect` spans **~800 ms**. A ratio
of medians with no spread beside it cannot distinguish a real slope from noise, and this arc has an earned
rule against banding a number measured once.

★ **Row 5 is why the drain can be compared at all.** `drain ≈ max(work, vis)` — measured. With `vis`
unpinned the drain reports a timeout, not work, and its ratio would be meaningless. `vis-ms=1000` is the
smallest swept value; `circuit.wat:2328` bounds attempts at `n×m`, so the patience scales with n while
`vis` does not.

★ **Row 8 is the integrity check.** This stone changes nothing. If `git status` shows a modified file, the
measurement was taken against a tree that is not the one on record.

⚠ **Row 6 exists because a fast run can be a broken run.** `distinct` must equal `n×m` — 4000 at n=1000,
8000 at n=2000, 16000 at n=4000 — with `dup=0`. A sample that lost messages is not a timing sample.

## Runtime prediction

**25–45 minutes of runs.** n=1000 ≈20 s, n=2000 ≈40 s, n=4000 ≈80 s (untested), ×3 = roughly 7 minutes of
circuit time, plus rebuild-free reruns and the floor at ~8 minutes. n=4000 is the unknown; if it takes
substantially longer than 80 s, say so — that is itself part of the answer.

## Trap-doors

- **n=4000 may not complete.** `circuit.wat:2328` bounds attempts at `n×m`, so the patience doubles with n
  and `vis=1000` should be well inside it — but that is reasoning, not a measurement. If it does not
  complete, **report the counters and the per-tier line**; it is a finding about the bound, not a failure.
- **The command form is `circuit.wat n m j sub-cap fill-first? [vis-ms]`.** Hold `m=4 j=3 sub-cap=8192
  fill-first?=true` fixed at every point; only `n` varies.
- **`workers=` counts distinct worker ids in outcomes**, not live processes. It spans 8–12 across this
  arc's runs and is **not** a defect signal.
- **`seen-skipped` and `redeliveries` vary run to run** from upstream visibility timing. Report them; do
  not treat movement as a regression.
- **Do not run the floor beside a measurement.** Two floors have gone red this session from exactly that.

## What this stone does NOT claim

⚠ It makes **no prediction** about which phase is superlinear, or that any is.
⚠ It does **not** fix anything, and does **not** touch `setup` — cold boot, the builder's ruling. Measuring
is not fixing.
⚠ It does **not** extend to n=8000. That is a later decision.
