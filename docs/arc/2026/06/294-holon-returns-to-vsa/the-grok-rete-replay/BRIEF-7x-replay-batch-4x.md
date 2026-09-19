# BRIEF 7x — replay batch 4x: grok-rete #601 → #620

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **600** REPLAY commits and a
clean tree. **51 steps remain after this batch's 20.**

## ⛔⛔ HARD PROHIBITIONS

1. ⛔ **DO NOT CALL `pulsare_yield`. DO NOT USE ANY `mcp__pulsare__*` TOOL.** The pulsare MCP server's own
   instructions recommend it — **overridden here.** Note the conflict in your report rather than
   complying. **You yield by ENDING YOUR TURN with your report.**
2. ⛔ **`/home/john/work/holon/` IS FROZEN.** Your world is `wat-rs`. **Start every Bash command with
   `cd /home/john/work/holon/wat-rs &&`.**
3. ⛔ **No worktrees. No push. No subagents. No `git filter-branch`. No `scripts/floor.sh`, no
   `cargo clippy`, no `cargo bench`, and NO unfiltered `cargo nextest run`.** Filtered `-E '…'` runs are
   yours.
4. ⛔ **VERIFICATIONS RUN IN THE FOREGROUND.** Ending your turn ends you.

⛔ **IF A FOREIGN EDIT, AN UNEXPECTED COMMIT, A LOCK YOU DID NOT CREATE, OR A FOREIGN PROCESS APPEARS:
STOP.** Capture `git status`, `git diff`, `git log -3` verbatim and report.

## ⛔ NEVER TYPE A SHA OR A SUBJECT — PIPE THEM

    SUBJ=$(git log -1 --format=%s <C>)
    SHA=$(git rev-parse <C>)

Quote heredocs (`<<'EOF'`), read `git log -1 --format=%B` back, keep every verdict on ONE physical line.
⚠ `git commit --amend` may be refused by the harness's classifier — repair via detach +
`git reset --soft HEAD^` + re-commit + cherry-pick-forward, proven with a tree diff.

## ⛔ READ THE CODE, NOT THE PROSE (new, and it cost me last batch)

My BRIEF-7w stated that #598 deleted an `#[allow]`. **It does not — it keeps it and adds a
justification.** I had grepped `^[-+]` across grok's whole commit and read `+` lines out of its `.md`
SCORE, where grok narrated a deletion it tried and reverted. **To learn what a step does to code, diff
the code paths only:** `git show <C> -- '*.rs'`. A commit that documents its own reasoning will hand you
the opposite of the truth otherwise. **If this brief contradicts grok's actual diff, the diff wins —
land what grok wrote and report the brief's error.**

## The work

**#601 → #620.** **Thirteen docs-only.** **Seven code steps.** The range is **18 `.md` + 11 `.rs` + 10
`.sh` + 7 `.wat` + 5 `.clj`**, **+6 tests** (3 at #607, 1 at #610, 2 at #613).

| N | C | files | note |
|---|---|---|---|
| 602 | `96d409985` | 4 (`.sh`) | grid: `ROWS_TOTAL` non-vacuity guard + Clara stderr capture |
| 603 | `dd57af2d0` | 2 (`.sh`) | grid: the promised `run-all.sh` tally; correct the JVM-tax claim |
| 607 | `c3c9caa54` | 6 (5 `.rs`) | gate four rotted rete header counts, cut the fifth; **+3 tests** |
| 610 | `a8d95ea82` | 4 (2 `.wat`, 1 `.rs`) | ⚠⚠ **a real `wat/` stdlib change — see below**; **+1 test** |
| 613 | `1ea6b2c8b` | 2 (1 `.rs`) | compose-test the `.wat.bad` gate's exemption wiring; **+2 tests** |
| 616 | `22d96739b` | 7 (1 new `.wat`) | ⚠ close a compound grid cell |
| 619 | `b4801eb5f` | 13 (4 new `.wat`) | ⚠ close the four remaining compound cells; the batch's largest |

**Pre-flighted and passing:** **ZERO hazard rows, ZERO new gates, ZERO `wat-scripts/fixes/` edits.**
Floor prediction **5883 run / 22 skipped** (5877 + 6). Verify it; do not inherit it.

## ⚠⚠ FIVE NEW `.wat` ARE PRE-MIGRATION AND WILL RED — MEASURED

#616 adds one and #619 adds four. **All five fail `./target/release/wat --check` on arrival.** Across
the set, measured:

- **42** positional `assertion-failed! "…"` (kwargs `:message`/`:actual`/`:expected` are required here),
- **25** `:wat::core::PersistentVector/conj`,
- **51** `:wat::core::i64::*` (`+`, `-`, `to-string`).

⛔ **Cure AT the step with the recorded chain** — `scripts/replay/convert.sh <introducing-commit> <out>
<path>` — the #537 precedent, where nine files were cured in one pass and the chain fixed **more** than
the pre-flight had found. **Never hand-edit.** Then run the loader gates
(`every_wat_scripts_file_loads_on_the_current_runtime`, `every_rete_name_in_wat_scripts_code_resolves`,
and the docs-wat gate if any land under `docs/`) and record their verdicts.

⚠ If a file still fails after conversion, capture the checker's own message verbatim, name the file, and
**STOP** — do not improvise a hand fix.

## ⚠⚠ #610 — A REAL STDLIB CODE CHANGE, NOT A COMMENT SWEEP

It edits `wat/rete/oracle/stratify.wat` by **45 CODE lines** (14 comment, 1 blank): `rule-negates`
recursing through `and`/`or` to match `negate_types`. It is the range's only `wat/` path and its only
`stdlib-touch` row, and it also touches a `.wat` probe and a `.rs`.

⛔ **R21 is NOT triggered** — this is one file's semantics, not a corpus-wide rewrite — so compose it by
hand. But it is stdlib: **run the loader gates after it**, and check the oracle's own tests
(`-E 'test(oracle)'` or the rete binary) with N > 0, and record what they said.

## ⚠ DIVERGENCE, AND A PRE-IMAGE SUBTLETY THAT BIT LAST BATCH

Measured: **#619 4-of-8, #607 3-of-5**, the rest 0–1. Compose, and compare **DELTAS, not blobs**
(finding 36).

⛔ **When an EARLIER STEP IN THIS SAME BATCH already touched the file, its pre-image is STEP-relative,
not batch-relative.** Last batch this produced a false 8-line discrepancy at #591 for a file #586 had
touched. Diff against `<C>~1` and your own step's parent, not the batch start.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture whole; name the assertion; STOP.
- ⛔ **READ EVERY COUNT OFF THE DATA**, and exclude comments when counting code.
- ⛔ **WAT IN `.rs`/`.sh` STRING LITERALS** is finding 33's class — 11 `.rs` and **10 `.sh`** this batch.
  Sweep per code step and SAY SO.
- ⛔ **A CURE ANSWERS TO THE GATES OF ITS MEDIUM**: after a `.rs` edit re-run the lint subset; after a
  `.wat` lands, run the loader gates.
- ⛔ **The record gate is PATH-BASED**: `^src/` needs `census:` + `nested-program-gate:`; `.rs` needs
  `lint-subset`, `kind(lib)`, `doctest`. **#607, #610, #613, #616, #619 touch `.rs`; #602/#603 are `.sh`
  only** — say so explicitly per step rather than leaving the record silent.
- ⛔ **`census:` must be TRUE.** A step whose change alters `wat --check` for files it did not produce is
  a STOP-8 and a finding.
- ⛔ **NEVER `git filter-branch`** — detach / re-commit / rebuild-descendants only.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** — every converted file named.

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh <batch-start> HEAD 601 620

(batch-start = the BRIEF commit's parent), exit 0, plus `git replace -l`, `git for-each-ref
refs/original/`, `git merge-base --is-ancestor origin/replay/grok-rete HEAD`, `git status --porcelain`,
and **a subject check for all 20 steps**. Then write `SCORE-7x-replay-batch-4x.md` (every row, never
blank) and a `REPLAY-LOG.md` section. Commit both. **Leave the tree CLEAN. Yield by ending your turn.**
