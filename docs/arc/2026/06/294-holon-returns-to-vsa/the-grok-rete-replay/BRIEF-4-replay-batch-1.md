# BRIEF 4 — replay batch 1: grok-rete #11 → #60

> Built on 2a2 (closed at `f36c2386c`: the codemods ask the door; `convert.sh` runs once per commit).
> RULED 2026-09-13 (4 YES): the replay resumes when 2a2 closes; stone 3 runs between batches. Batch size
> chosen by four questions in the main chat: #11–#60 first (the first at-scale use of the new tooling,
> and the most code-heavy stretch), then #61–#125.

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd` and
`git rev-parse --abbrev-ref HEAD`. **Never use worktrees.** Do not touch `~/work/holon/` (the frozen
root). Do not touch `main`.

## The work

Replay grok-rete commits **#11 through #60** (index after `de827fb4c`; `#11 = eebf75374`), one at a
time, in order, by the recipe in `BRIEF-1-pilot-first-ten-commits.md` § "One step" — as 2a2 updated
it (`convert.sh <rev> <out-dir> <path>…`, called once for the C^ set and once for the C set).

The census for this range (`bootstrap/era/replay-plan/commits.tsv`): 3 docs-only, 37 code touching only
grok-rete files, 10 touching a file main also changed. None touches `wat-scripts/fixes/`; none touches a
file main deleted (those start at #126 and #153).

## Checkpoints

After **#35** and after **#60**: `scripts/floor.sh` (read the Summary line) and
`cargo clippy --release --all-targets -- -D warnings`. A red floor is reported verbatim and never
re-run.

## The log

`REPLAY-LOG.md` in this directory, one row per step with PILOT-LOG's columns: N, C → replayed hash,
kind, wall time, files, conflicts and their resolutions, hand edits (wat in `.rs` strings), tests run by
name and their result, and every report line `convert.sh` printed for a produced file
(`UNREGISTERABLE`, `UNRESOLVED …`, `SPLICE`, `UNREADABLE`).

## ⛔ STOP triggers — each is a REJECTION. Commit nothing further; report the verbatim evidence.

- **STOP-1 … STOP-5:** exactly as `BRIEF-1-pilot-first-ten-commits.md` § "STOP triggers" (rete behaviour
  lost; a chain defect in a converted `.wat`; an `.rs` conflict needing a third behaviour; a checkpoint
  floor red inside the replayed files; `convert.sh` non-deterministic or the chain order disagreeing).
- **STOP-6:** a step touches `wat-scripts/fixes/` or a file main deleted. The census says none in this
  range does; if one does, the census is wrong.

## Tier

Commit each replayed step on green: `REPLAY(grok-rete #N): <C's subject>`. **Do not push.** Yield after
#60, or at the first STOP, with `SCORE-4-replay-batch-1.md` and `REPLAY-LOG.md`.

## Expectations

`EXPECTATIONS-4-replay-batch-1.md` (written before the strike): E1–E9, the runtime prediction, and the
trap doors (#59 and #60 are the large shared steps).
