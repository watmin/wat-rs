# BRIEF 7f — replay batch 4f: grok-rete #241 → #260

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd` and
`git rev-parse --abbrev-ref HEAD`. Expect 240 REPLAY commits and a clean tree.
⛔ Do NOT touch `/home/john/work/holon/` (FROZEN) or `main`. **Never use worktrees. Never push. Do not
spawn subagents. Do NOT run `scripts/floor.sh`, clippy or run5** — the orchestrator weighs those centrally
and uncontended; a gate run while you work in the tree is a FALSE result.

⚠ `wat-rs/CLAUDE.md` does not reach an executor. The load-bearing doctrine is carried here.

## Doctrine — each line was paid for

- **R21:** `.wat` corpus rewrites go through a recorded wat-fix codemod (`wat-scripts/fixes/*.wat`), NEVER
  hand edits or sed. Scratch `.wat` → `wat-scripts/scratch-pad/`.
- ⛔ **WAT EMBEDDED IN `.rs`/`.sh` STRING LITERALS IS THE EXCEPTION — AND THIS REPLAY'S MOST PERSISTENT
  DEFECT SOURCE** (finding 33). No codemod reaches it, `convert.sh` never sees it, no gate parses it. It has
  bitten #162 (7 sites), #167 (a `perl` substitution broken for weeks), and #238. **When a step's subject
  mentions a rename or rehome, grep the `.rs`/`.sh` side too.** Hand-fix and LOG each edit.
- ⛔ **A rename census must be built from the RECORDED MIGRATIONS, not the shape of the name** (finding 33).
  `rete::core::{i64,f64,string}` were rehomed; `rete::core::keyword::=` was deliberately NOT — it is a live
  `#[wat_special_form]`. Check `wat-scripts/fixes/rename-*.wat` before calling a spelling stale.
- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything — redirect to a file and read the
  file** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE.** On any red: do NOT re-run; copy the whole stdout+stderr block verbatim;
  name the exact assertion; STOP and report. **This is not a formality — at #234 it caught a gate that was
  reddening ~13% of all floors, and the step that hit it was exonerated by measurement.**
- ⛔ **Ending your turn ENDS you.** Every verification in the FOREGROUND, blocking.
- ⛔ **After ANY commit or `--amend`: assert `git status --porcelain` is EMPTY and that the commit's diff
  names every path its body claims** (finding 27). Read every COUNT off `git show --stat`/the diff.
- ⛔ **ANY REPAIR MUST BE VISIBLE TO `push`** (finding 29). No `git replace`, no overlay. If blocked, STOP.
- ⛔ **A mutation proof must falsify the proposition you rely on** (finding 30). `rc=1` says nothing about WHY.
- ⛔ **Each verdict line must be ON ONE LINE** (finding 31) — the gate matches within a single line.
- **Every step, docs-only included**, commits as `REPLAY(grok-rete #N): <C's subject>` with the
  `(cherry picked from commit <sha>)` trailer. BRIEF-1's item 1 is correct as written — follow it.

## The work

Replay **#241 → #260** (20 steps) per `BRIEF-1` § "One step". **Fourteen are docs-only**
(#243 #244 #245 #247 #248 #249 #251 #252 #253 #255 #256 #257 #259 #260 — that is 14; count them against
the census rather than trusting this sentence). The other six are #241 #242 #246 #250 #254 #258.

| N | C | kind | files | note |
|---|---|---|---|---|
| 241 | `ccb39b82a` | code | 4 (**1 `.wat`, 0 `.rs`**) | census + nested-gate lines; **no `.rs` walls** |
| 242 | `7e24c3257` | shared | 5 (5 `.rs`) | |
| 246 | `b0e3377e9` | shared | 7 (**4 `.wat`**, 3 `.rs`) | |
| 250 | `f22704f1f` | code | 6 (1 `.wat`, 3 `.rs`) | |
| 254 | `c9cdd9d32` | shared | 11 (11 `.rs`) | ⚠ **see the divergence warning** |
| 258 | `1efb42fc7` | shared | 17 (**5 `.wat`**, 8 `.rs`) | **the batch's trap — largest step** |

**No stdlib-touch row, no `absent-on-main.tsv` row, and no `future-macro-changes.txt` entry in this range.**
The next two-phase stdlib step is #377, so nothing here needs the door's stdlib mode.

## ⚠ #254 — you will meet a DELIBERATE divergence from grok. Do not "fix" it.

#254 touches `src/rete/kernel/tests/gather_probe_cost.rs`. **We deliberately diverge from grok in that
file**: commit `0fa6948da` struck `probe_extend_cost_split`'s `h >= (b + m + e) * 0.5` apportionment
assert, which measured 33–87 ns and reddened ~13% of floors (finding 32). grok keeps that assertion to its
tip and never records hitting it — so a conflict or a surprising diff there is EXPECTED.

**Keep our struck version. Do NOT restore grok's assertion**, and do not treat its absence as a merge
error. grok's #254 change to that file is a 1-line edit, so a clean apportionment is likely; if the hunks
overlap our comment block, keep our block and apply grok's substantive change around it. Log it.

## ⚠ The path-based verdict-line rule

The record gate derives requirements from each commit's OWN diff: a `.wat` or `src/` change needs
`census: … --diff no STOP-8` and `nested-program-gate: PASS`; a `.rs` change needs `lint-subset`,
`kind(lib)` and `doctest`. **#241 carries a `.wat` and zero `.rs`**, so it needs the first pair and NOT the
second. Misjudging this in the inverse direction cost a record repair at #215.

## ⛔ STOP triggers — rejections, not permission to defer

STOP-1 … STOP-13 per BRIEF-1; in particular **STOP-8** (a file going rc 0 → non-zero the step did not
produce), **STOP-10** (stone 3's nested-program gate), **STOP-11** (`kind(lib)` or doctests red).

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it. **Never
a repair commit after the batch, never a knowingly-red REPLAY commit.** A fold's proof must name WHICH
paths it carries, checked with `git diff <old-tip> <new-tip>`. A gate that did not exist at step N is not a
reason to fold into N (the #184 precedent) — repair it where the gate lands.

## Tier

Commit each step on green. **Do not push.** Before yielding, run in the FOREGROUND and paste:

    scripts/replay/verify-step-record.sh <batch-start> HEAD 241 260

It must exit 0. Yield after #260, or at the first STOP, with `SCORE-7f-replay-batch-4f.md` and a
`REPLAY-LOG.md` section, leaving the tree CLEAN.
