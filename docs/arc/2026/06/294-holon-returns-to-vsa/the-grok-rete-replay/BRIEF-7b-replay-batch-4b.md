# BRIEF 7b — replay batch 4b: grok-rete #160 → #211

> Built on 4a (the codemod-source policy's first use) and every per-step wall. Released after 4a is
> verified.

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd` and
`git rev-parse --abbrev-ref HEAD`. **Never use worktrees.** Do not touch `~/work/holon/` or `main`.
**Never push. Do not spawn subagents. Do not run `scripts/floor.sh`, clippy or run5** — the orchestrator
weighs those centrally, uncontended; your checks are the per-step ones below plus the step's named tests.
⚠ `wat-rs/CLAUDE.md` does not reach an executor: the load-bearing doctrine is carried here — `.wat`
rewrites go through a recorded wat-fix codemod (R21), never hand edits or sed; scratch `.wat` lives in
`wat-scripts/scratch-pad/`; THERE IS NO KNOWN FLAKE — on any red, do not re-run, copy the block verbatim,
name the assertion, and STOP.

## The work

Replay grok-rete **#160 through #211** (`#160` per `commits.tsv`) by `BRIEF-1-pilot-first-ten-commits.md`
§ "One step" with every row it carries — each commit body carrying the five verdict lines verbatim;
`scripts/replay/verify-step-record.sh <start> HEAD` exits 0 before you yield.

The census: 23 docs-only (#160 #161 #163 #170 #172 #179 #181 #185 #186 #189 #191 #195 #196 #197 #198 #200
#203 #205 #206 #208 #209 #210 #211), 14 code touching only grok-rete files, 15 touching a file main also
changed; 189 files, 117 `.rs`. **None touches `wat/`, `wat-scripts/fixes/`, a file main moved, or
positional-ctor's skip-listed paths.** The largest: #176 (19 files), #184 (19), #177 (16), #182 (9), #187 (9).

## Checkpoints

After **#185** and after **#211**: `scripts/floor.sh` and clippy. A composition repair folds into the
step that needs it (the 4b rule) — never a commit after the batch.

## ⛔ STOP triggers — rejections; report the verbatim evidence

STOP-1 … STOP-13 as BRIEF-1 and BRIEF-7a state them.

## Tier

Commit each step on green. **Do not push.** Yield after #211, or at the first STOP, with
`SCORE-7b-replay-batch-4b.md` and `REPLAY-LOG.md`.
