# BRIEF 7w — replay batch 4w: grok-rete #581 → #600

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **580** REPLAY commits and a
clean tree.

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

## The work — back to real code

**#581 → #600.** **Fifteen docs-only.** **Five code steps**, 20 `.rs` files, **+5 tests**.

| N | C | files | note |
|---|---|---|---|
| 581 | `1b4e3c30e` | 3 (2 `.rs`) | close the quote door into `lower`; **+4 tests** |
| 586 | `4495aafcd` | 4 (3 `.rs`) | `ARM_BUILDS` thread-owned; **+1 test**; ⚠ **3-of-3 diverged here** |
| 591 | `4d96c7534` | 6 (5 `.rs`) | lift instrument-subtraction arithmetic into one fn; ⚠ **5-of-5 diverged** |
| 594 | `87285b5db` | 6 (5 `.rs`) | ⚠⚠ **eleven `too_many_arguments` allows — see below**; 0-of-5 diverged |
| 598 | `a8c233eb3` | 6 (5 `.rs`) | five small L1s; an `#[allow(unused_variables)]` deleted, a `rune:sequi` added; 2-of-5 diverged |

**Pre-flighted and passing:** **ZERO hazard rows, ZERO new gates, ZERO `.wat`, ZERO `wat/` paths, ZERO
`wat-scripts/fixes/`.** Floor prediction **5877 run / 22 skipped** (5872 + 5). Verify it; do not inherit
it.

## ⚠⚠ #594 — IT REMOVES A CLIPPY ALLOW, AND CLIPPY IS NOT YOURS TO RUN

#594 drops `#[allow(clippy::too_many_arguments)]` from `activate_deferred_mixed_classes`
(`src/rete/kernel/fire/pass/alpha.rs`) and gives the remaining ten allows explicit reasons.

**The removal is safe ONLY because the same step bundles nine arguments into a new `AlphaActivateCx`,
taking that function from 11 parameters to 3.** Measured before release: our `clippy.toml` is
byte-identical to grok's and sets **no** `too-many-arguments-threshold`, so clippy's default of **7**
applies. **11 > 7; 3 < 7.** The allow and the struct are one change.

⛔ **You cannot run clippy — it is the orchestrator's row — so verify the mechanism instead, AT #594:**
- the `AlphaActivateCx` struct landed;
- `activate_deferred_mixed_classes`'s parameter list is **3**, not 11;
- every call site passes the struct.

**If the allow is removed while the signature still has 11 parameters, clippy reds at the orchestrator's
checkpoint and the batch takes a fold.** Say in #594's body what you verified and what you counted.

## ⚠ HIGH DIVERGENCE — COMPOSE, AND COMPARE DELTAS

**#591 is 5-of-5, #586 3-of-3, #598 2-of-5** touched `.rs` already different here. Cherry-picks will
conflict. When you ask "did grok's change land unchanged", **compare the two DELTAS** (`git diff -U1` on
grok's pre→post and on ours, headers stripped), never blob against blob — finding 36, and the method the
4v executor used well.

## ⚠ #598 deletes an `#[allow]` and adds a rune

It removes an `#[allow(unused_variables)]` together with the binding it covered, and adds
`rune:sequi(ambient-context)`. ⛔ **Run the ward-vocabulary gate after it** (`-E
'test(no_unknown_ward_rune)'`, N > 0) — a rune whose category the vocabulary does not know is exactly
what that gate exists to catch. Quote its verdict.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture whole; name the assertion; STOP.
- ⛔ **READ EVERY COUNT OFF THE DATA**, and exclude comments when counting code.
- ⛔ **WAT IN `.rs` STRING LITERALS** is finding 33's class — 20 `.rs` files this batch. Sweep per code
  step and SAY SO.
- ⛔ **A CURE ANSWERS TO THE GATES OF ITS MEDIUM**: after a `.rs` edit, re-run the lint subset at that
  step.
- ⛔ **The record gate is PATH-BASED**: all five code steps touch `src/`, so each needs `census:` +
  `nested-program-gate:` + `lint-subset` + `kind(lib)` + `doctest`, each on ONE line.
- ⛔ **`census:` must be TRUE.** A step whose `src/` change alters `wat --check` for files it did not
  produce is a STOP-8 and a finding.
- ⛔ **NEVER `git filter-branch`** — detach / re-commit / rebuild-descendants only.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT.**

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh <batch-start> HEAD 581 600

(batch-start = the BRIEF commit's parent), exit 0, plus `git replace -l`, `git for-each-ref
refs/original/`, `git merge-base --is-ancestor origin/replay/grok-rete HEAD`, `git status --porcelain`,
and **a subject check for all 20 steps**. Then write `SCORE-7w-replay-batch-4w.md` (every row, never
blank) and a `REPLAY-LOG.md` section. Commit both. **Leave the tree CLEAN. Yield by ending your turn.**
