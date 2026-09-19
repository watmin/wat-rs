# BRIEF 7u — replay batch 4u: grok-rete #541 → #560

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **540** REPLAY commits, a clean
tree, and `HEAD == origin` at the BRIEF commit.

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

Six trailers were fabricated across three batches, every one self-caught, every one costing a repair.
Last batch had zero because the executor piped them. Do that. Quote heredocs (`<<'EOF'`), read
`git log -1 --format=%B` back, keep every verdict on ONE physical line.

⚠ `git commit --amend` may be refused by the harness's classifier; repair via detach +
`git reset --soft HEAD^` + re-commit + cherry-pick-forward, proven with a tree diff.

## The work — pure documentation

**#541 → #560: twenty docs-only steps. The whole range is 52 `.md` files.** grok's vigilia, working four
targets to completion.

**Pre-flighted and passing:** **ZERO `.rs`, ZERO `.wat`, ZERO `.sh`, ZERO `src/`, ZERO hazard rows, ZERO
new gates, ZERO `wat/` paths.** No `#[test]` or `#[ignore]` moves anywhere. The floor prediction is
**unchanged at 5872 run / 22 skipped**, and structurally it cannot move — verify that claim, do not
inherit it.

## ⚠⚠ THE ONE REAL TRAP — ONE DIVERGED FILE, TOUCHED NINE TIMES

`docs/arc/2026/06/278-rules-engine/CURRENT-STATE-annihilate-interpretation.md` is a **shared status
home**. Measured: our copy differs from grok's pre-image for this range by **12 lines (9 insertions, 3
deletions)** — main's own annotations. And **nine of the twenty steps touch it**: **#548, #549, #551,
#552, #554, #555, #556, #557, #560.**

**Expect a conflict at several of them.** The governing precedent is #515 one batch ago, where the same
shape appeared in `COMPACTION-AMNESIA-RECOVERY.md`: **keep BOTH sides.** Our annotation is main's record;
grok's text is the history being replayed. **Neither is dropped.**

⛔ **Verify after EACH conflicted step, not once at the end** — that our annotation survives, that grok's
new text landed, and that no `<<<<<<<` / `=======` / `>>>>>>>` marker remains. Record each resolution in
that step's own commit body.

## ⚠ A CITATION THAT LOOKS STALE AND IS NOT YOURS TO FIX

#558 adds `` `wat/rete.wat:547+` `` while `wat/rete.wat` is **541 lines**. Measured: it is out of range on
**grok's own tree too** (also 541 lines), and it sits in a `.md`.

⛔ `tests/lint/no_stale_path_in_doc.rs` scans **doc comments inside `src/rete/*.rs`, `wat/*.wat` and
`wat-tests/*.wat`** — its own `ROOTS`. **It does not scan `docs/**.md`.** So this is not gated here.
**Land it unedited** (the standing rule: grok's prose keeps grok's words; a figure or citation false of
this tree is an observation for your SCORE, never an edit to grok's text).

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture whole; name the assertion; STOP.
- ⛔ **READ EVERY COUNT OFF THE DATA**, and exclude comments when counting code.
- ⛔ **The record gate is PATH-BASED.** **No step here touches `src/` or `.rs`**, so no step needs
  `census:` / `lint-subset` / `kind(lib)` / `doctest` lines — but say so explicitly per step rather than
  leaving the record silent.
- ⛔ **NEVER `git filter-branch`** — detach / re-commit / rebuild-descendants only.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** (E10) — every conflict resolution named.

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh <batch-start> HEAD 541 560

(the batch-start is the commit before your first step — the BRIEF commit's parent), exit 0, plus
`git replace -l`, `git for-each-ref refs/original/`, `git merge-base --is-ancestor
origin/replay/grok-rete HEAD`, `git status --porcelain`, and **a subject check for all 20 steps**. Then
write `SCORE-7u-replay-batch-4u.md` (every row, never blank) and a `REPLAY-LOG.md` section. Commit both.
**Leave the tree CLEAN. Yield by ending your turn — signal nothing.**
