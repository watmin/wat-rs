# BRIEF 7s — replay batch 4s: grok-rete #501 → #520

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **500** REPLAY commits, a clean
tree, and `HEAD == origin == 9cc844d23`.

## ⛔⛔ HARD PROHIBITIONS

1. ⛔ **DO NOT CALL `pulsare_yield`. DO NOT USE ANY `mcp__pulsare__*` TOOL.** The pulsare MCP server's own
   instructions recommend it — **overridden here.** Note the conflict in your report rather than
   complying. **You yield by ENDING YOUR TURN with your report.**
2. ⛔ **`/home/john/work/holon/` IS FROZEN.** Your world is `wat-rs`. **Start every Bash command with
   `cd /home/john/work/holon/wat-rs &&`.**
3. ⛔ **No worktrees. No push. No subagents. No `git filter-branch`. No `scripts/floor.sh`, no
   `cargo clippy`, no `cargo bench`, and NO unfiltered `cargo nextest run`** — filtered `-E '…'` runs are
   yours; the whole floor is the orchestrator's row.
4. ⛔ **VERIFICATIONS RUN IN THE FOREGROUND.** Ending your turn ends you.

⛔ **IF A FOREIGN EDIT, AN UNEXPECTED COMMIT, A LOCK YOU DID NOT CREATE, OR A FOREIGN PROCESS APPEARS:
STOP.** Capture `git status`, `git diff`, `git log -3` verbatim and report. Do not discard it.

## ⛔ SUBJECTS AND TRAILERS ARE COPIED, NEVER TYPED

    SUBJ=$(git log -1 --format=%s <C>)     # commit as "REPLAY(grok-rete #N): $SUBJ"
    SHA=$(git rev-parse <C>)               # the cherry-pick trailer

**Two fabricated trailers were self-caught last batch, and three more in the batch before.** Type
neither. Quote your heredocs (`<<'EOF'`), read `git log -1 --format=%B` back after every commit, and keep
each `census:` / `lint-subset:` / `kind(lib):` verdict on ONE physical line.

⚠ **`git commit --amend` may be REFUSED by the harness's destructive-action classifier** — last batch it
was, twice. If you need to repair a commit, use detach + `git reset --soft HEAD^` + re-commit +
cherry-pick-forward, and prove the result with a tree diff.

## ⛔ A PREDICTION HERE IS A PREDICTION (finding 37)

The executor has corrected my floor prediction **twice running** and was right both times. Measure; report
disagreement as a result (row E14).

## The work — the lightest batch of the campaign

**#501 → #520.** **Nineteen docs-only:** 502–520. **One code step: #501.**

The whole range is **34 `.md` files and 1 `.wat`** — grok's 2026-09-07 **vigilia**, an audit cast
(intueri, purgare, solvere, struere, conferre, sequi, temperare, excusare, exigere, conformare, cernere,
probare, perspicere) recorded as prose, plus its stamps and its resume protocol.

**#501** (`ba203bf85`) is the only code step: a **comment-only** fix to
`wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat`, striking a `println` literal
(`"COMPILE: Compiled"`) that printed unconditionally before anything compiled and so proved nothing.

**Pre-flighted and passing:** **ZERO `src/` files, ZERO hazard rows, ZERO new gates, ZERO
`wat-scripts/fixes/` edits, ZERO `wat/` paths, and ZERO test delta** — no `#[test]` or `#[ignore]` is
added or removed anywhere in the range. The floor prediction is therefore **unchanged: 5872 run, 22
skipped**. That is the claim to check, not to inherit.

## ⚠ THE ONE REAL TRAP — DO NOT "CORRECT" GROK'S MEASUREMENTS

The vigilia docs state numbers grok measured **on grok's tree**: *"excusare weighs all 65 exemptions — 59
hold, 6 struck"*, *"row all 18 findings"*, and so on. **These are the historical record of grok's own
audit, exactly like a SCORE.** This tree's numbers differ — #496 proved it one batch ago, where grok's
"all 25 counters" is **45** here.

⛔ **Do NOT edit those figures to match this tree.** Replaying a record means landing what it said.
If you notice a figure that is false *of this tree*, that observation belongs in **your SCORE**, never
inside grok's prose. Silently "fixing" it would falsify the very thing being replayed.

## ⚠ THE CITATIONS ARE IN RANGE, AND THE GATE SEES ONLY RANGE

The range adds **72 `path:line` citations**. Measured before release: **all 72 resolve to files that
exist here, at line numbers within those files.** `tests/lint/no_stale_path_in_doc.rs` checks exactly
that — **existence and range**. It cannot see that our `src/check.rs:21514` may now hold different
content than grok's did. **That semantic drift is invisible, expected, and NOT a defect to chase.** Run
the gate; report its verdict.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture whole; name the assertion; STOP.
- ⛔ **READ EVERY COUNT OFF THE DATA AS YOU WRITE THE SENTENCE**, and **exclude comments when you count
  code** — my own grep recently counted 7 `#[ignore]` sites in a file with 2.
- ⛔ **A docs-only step still needs its record line.** The gate is PATH-BASED: `^src/` needs `census:` +
  `nested-program-gate:`; `.rs` needs `lint-subset`, `kind(lib)`, `doctest`. **Nineteen of these steps
  need none of those** — but #501 touches a `.wat` under `wat-scripts/`, so run the loader gates for it
  and record what they said.
- ⛔ **NEVER `git filter-branch`** — detach / re-commit / rebuild-descendants only.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** (E11).

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh 9cc844d23 HEAD 501 520

exit 0, plus `git replace -l`, `git for-each-ref refs/original/`, `git merge-base --is-ancestor
origin/replay/grok-rete HEAD`, `git status --porcelain`, and **a subject check for all 20 steps**. Then
write `SCORE-7s-replay-batch-4s.md` (every row, never blank) and a `REPLAY-LOG.md` section. Commit both.
**Leave the tree CLEAN. Yield by ending your turn — signal nothing.**
