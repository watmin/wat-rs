# BRIEF 7i — replay batch 4i: grok-rete #301 → #320

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd` and
`git rev-parse --abbrev-ref HEAD`. Expect **300** REPLAY commits and a clean tree.

## ⛔⛔ READ THIS FIRST — THREE HARD PROHIBITIONS, ONE OF THEM NEW

1. ⛔ **DO NOT CALL `pulsare_yield`. DO NOT USE THE `mcp__pulsare__*` TOOLS AT ALL.** Batch 4h's
   executor called `pulsare_yield` at the end of its run. Nothing authorized it. It **wrote into the
   FROZEN root** (`/home/john/work/holon/.pulsare/`) and **woke a counterpart that then ran its own
   `scripts/floor.sh` inside this working tree**, contending with the orchestrator's gates (a wall run
   took 61.2s against a 28.3s baseline). **You yield by ENDING YOUR TURN with your report. Nothing else.**
2. ⛔ **`/home/john/work/holon/` IS FROZEN — including `.pulsare/`, `.git/`, and every dotfile.** Do not
   read it, write it, or signal through it. Your entire world is `/home/john/work/holon/wat-rs`.
3. ⛔ **NEVER use worktrees. NEVER push. Do NOT spawn subagents. Do NOT run `scripts/floor.sh`, `cargo
   clippy`, or run5** — the orchestrator weighs those centrally and uncontended; a gate run while you hold
   the tree is a FALSE result.

⚠ `wat-rs/CLAUDE.md` does not reach an executor. The load-bearing doctrine is carried here.

## Doctrine — each line was paid for

- **R21:** `.wat` corpus rewrites go through a recorded wat-fix codemod, NEVER hand edits or sed. Scratch
  `.wat` → `wat-scripts/scratch-pad/`. **This range contains ZERO `.wat` files** — if you reach for
  `convert.sh`, you have misread the diff.
- ⛔ **WAT EMBEDDED IN `.rs`/`.sh` STRING LITERALS IS R21's EXCEPTION AND THIS REPLAY'S MOST PERSISTENT
  DEFECT SOURCE** (finding 33) — #162, #167, #238, #242, #266. **#302 touches the very file that class was
  named for** (see below). Grep the `.rs`/`.sh` side and SAY SO. "Not applicable" is an answer; silence is not.
- ⛔ **A rename census is built from the RECORDED MIGRATIONS, not the shape of a name** (finding 33).
- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28). Redirect to a
  file and read the file.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** On any red: do NOT re-run; copy the whole stdout+stderr
  block verbatim; name the exact assertion; STOP and report.
- ⛔ **Ending your turn ENDS you.** Every verification in the FOREGROUND, blocking.
- ⛔ **READ EVERY COUNT OFF THE DATA AS YOU WRITE THE SENTENCE** (findings 27, 34, 36). **A disclosure is
  not a cure** — 4f's SCORE disclosed this class then repeated it; the orchestrator has produced SEVEN
  false discrepancies in one day, every one from an instrument, not from an executor.
- ⛔ **DIFF LIKE AGAINST LIKE** (finding 36). To compare our tree to grok's, diff **the step's blob against
  the step's blob** — `git diff <grok-N>:<path> <our-N>:<path>` — **never against `HEAD`**, which spans the
  whole batch and attributes every later step's work to the one you are looking at.
- ⛔ **A NEGATIVE FINDING CARRIES THE TIMESTAMP OF ITS MEASUREMENT** (finding 36). "Nothing has changed"
  decays the instant the observation ends. Re-run it before quoting it.
- ⛔ **The record gate's rule is PATH-BASED, not `.wat`-triggered.** A `^src/` change needs `census:` +
  `nested-program-gate:` EVEN IF every file is `.rs`. A `.rs` change needs `lint-subset`, `kind(lib)`,
  `doctest`.
- ⛔ **Each verdict line must be ON ONE LINE** (finding 31). Put any `(vs #N's …)` detail **AFTER**
  `no STOP-8`, never between `--diff` and it — that wrap cost batch 4g a history rewrite.
- ⛔ **ANY REPAIR MUST BE VISIBLE TO `push`** (finding 29). No `git replace`, no overlay. **Prefer detach /
  re-commit / rebuild-descendants; NEVER `git filter-branch`** (finding 35) — it is a whole-history
  rewriter whose only scope is an argument. If history is rewritten at all, prove in the SCORE that
  `git merge-base --is-ancestor origin/replay/grok-rete HEAD` succeeds and `refs/original/` is EMPTY. **A
  tree-hash comparison does NOT establish this.**
- ⛔ **NEVER FABRICATE A SHA.** 4h's executor wrote an invented `cherry picked from commit` trailer and
  self-caught it. Copy trailers from `commits.tsv` or `git log`, never from memory.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** (finding 35, row E19). Any landing-time action
  that changes a gate, its inputs, or its ledger goes in the SCORE row that gate satisfies — not only in
  the log. This row earned itself on its first outing at 4h.
- **Every step, docs-only included,** commits as `REPLAY(grok-rete #N): <C's subject>` with the
  `(cherry picked from commit <sha>)` trailer.

## The work

Replay **#301 → #320** (20 steps). **Thirteen are docs-only:**

    301 303 305 307 308 309 311 312 314 315 316 318 319

**Seven carry code:**

    302 304 306 310 313 317 320

That is 13 + 7 = 20. **Count both lists against the census rather than trusting this sentence.**

**ZERO hazard rows of any kind** (no stdlib-touch, no absent-on-main, no future-macro, no main-deleted),
**no step touches `wat/`**, and **no `.wat` file appears anywhere in the range** — all pre-flighted before
release. All 5 M-status-absent paths are `docs/` DESIGN/EXPECTATIONS files created earlier in the same
batch (#309→#311, #312→#314, #316→#318): no hazard, but if one is missing its creator did not land.

## ⚠⚠ #310 — THE FOURTH NEW GROK LINT GATE, AND IT WILL LAND RED

`tests/lint/census_name_read_by_a_cost_test_is_emitted.rs` (780 lines) demands that **every census name a
cost test READS be a name the engine EMITS.** The defect it exists for: `unwrap_or(0)` answers *"this mark
does not exist"* and *"this mark measured zero nanoseconds"* with the same `0`, so a never-emitted name
prints `0.00 ms` and feeds arithmetic. Grok drove a real case where `accum_cost.rs` printed `0 − S` as a
difference between two measurements when only one was ever taken.

Declaration form: `// rune:lint(census-name-retired) <name> — <reason>`, **40-char reason floor**.
Non-vacuity floors: `from_literals > 50`, `rd.len() > 40`.

⛔ **Grok's #310 repairs ONE file (`accum_cost.rs`). Our tree has EIGHT `*_cost.rs` files and 91 census
mentions.** That asymmetry is exactly what reddened #274, #278 and #283. **Repair AT #310** (the #184
precedent) — never fold backward, never weaken or allowlist the gate. Read each site; a rune is a
DECLARATION naming a real mechanism, never a blanket suppression.

**The pattern is four for four.** Report your actual numbers, and say where they differ from this brief.

## ⚠ #302 — the finding-33 hot spot itself

#302 touches `wat-scripts/perf/grid/run-axis.sh` — **the exact file whose embedded `perl` substitution sat
silently broken for weeks (#167)**, invisible to every gate because no `.wat` is involved. It also adds
`compare-grids.sh` and three `GRID-*.txt` artifacts. **Grep the `.sh` side, read the substitution, and log
what you found.**

## ⚠ #315 is GENUINELY EMPTY in grok's own history

0 files — a commit-message restore. It still needs a `REPLAY(grok-rete #315)` commit for the record gate:
the #202/#300 precedent. Use `git commit --allow-empty` and say plainly in the body that grok's own commit
carries no tree change.

## ⚠ All seven code steps touch `src/rete/kernel/tests/*_cost.rs`

Timing-sensitive benchmark files — the exact class that produced **finding 28** (a wall-clock ratio
asserted as a gate) and **finding 32** (a nanosecond apportionment gate that reddened ~13% of floors).
⛔ **Treat any new ratio or threshold assertion with suspicion, and report it rather than accepting it.**
Note `gather_probe_cost.rs` carries OUR deliberate divergence from grok (finding 32's strike) — do not
"restore" grok's assertion if a conflict surfaces there.

## ⛔ EVERY `-E` FILTER MUST SELECT A NON-ZERO COUNT

A filterset matching nothing runs zero tests and **exits 0**. Read each run's own `N tests run` line.

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.** Prove a fold with
`git diff <old-tip> <new-tip>`. **A gate that did not exist at step N is not a reason to fold into N —
the #184 precedent — and #310 is exactly that case.**

## Tier

Commit each step on green. **Do not push.** Before yielding, run in the FOREGROUND and paste:

    scripts/replay/verify-step-record.sh <BATCH-START-SHA> HEAD 301 320

It must exit 0. Also paste `git replace -l`, `git for-each-ref refs/original/`, `git merge-base
--is-ancestor origin/replay/grok-rete HEAD`, and `git status --porcelain`.

Then write `SCORE-7i-replay-batch-4i.md` answering every row, plus a `REPLAY-LOG.md` section. Commit both.
**Leave the tree CLEAN. Do not push. End your turn to yield — do not signal anything.**
