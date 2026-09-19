# BRIEF 7t — replay batch 4t: grok-rete #521 → #540

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **520** REPLAY commits, a clean
tree, and `HEAD == origin == 5f476ab4d`.

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
STOP.** Capture `git status`, `git diff`, `git log -3` verbatim and report. Do not discard it.

## ⛔ NEVER TYPE A SHA OR A SUBJECT — PIPE THEM

    SUBJ=$(git log -1 --format=%s <C>)
    SHA=$(git rev-parse <C>)

**Six trailers have been fabricated across the last three batches**, every one self-caught, every one
costing a repair. The cure is mechanical: build the message by piping those variables in, never by
retyping. Quote heredocs (`<<'EOF'`), read `git log -1 --format=%B` back after every commit, and keep
each `census:` / `lint-subset:` / `kind(lib):` verdict on ONE physical line.

⚠ `git commit --amend` may be refused by the harness's destructive-action classifier. To repair: detach +
`git reset --soft HEAD^` + re-commit + cherry-pick-forward, proven with a tree diff.

## The work

**#521 → #540.** **Eighteen docs-only.** **Two code steps.** The range is **39 `.md` + 9 `.wat` + 1
`.sh`** — grok's vigilia continuing.

| N | C | files | note |
|---|---|---|---|
| 537 | `628e6371d` | 11 (9 `.wat`) | ⚠⚠ **experiri — nine new scratch `.wat`, ALL pre-migration** |
| 540 | `2961853f2` | 3 (1 `.sh`) | peragrare — adds `wat-scripts/perf/grid/peragrare-census.sh` |

**Pre-flighted and passing:** **ZERO `src/`, ZERO hazard rows, ZERO new gates, ZERO
`wat-scripts/fixes/`, ZERO `wat/` paths, ZERO test delta** — no `#[test]` or `#[ignore]` moves anywhere
in the range, so the floor prediction is **unchanged: 5872 run, 22 skipped**. That is the claim to check.

## ⚠⚠ #537 WILL GO RED WITHOUT CONVERSION — MEASURED, NOT FEARED

#537 adds nine files under `wat-scripts/scratch-pad/experiri-then/`. **All nine** carry syntax this tree
retired:

1. **The positional `assertion-failed!`.** Our checker names it exactly: *"assertion-failed! takes kwargs
   `:message` / `:actual` / `:expected`; the positional (message actual expected) form is retired"*.
2. **`:wat::core::i64::+`** — nine occurrences across the set. The live spelling is `:wat::i64::+`.

⛔ **`wat-scripts/scratch-pad/` IS NOT A QUIET CORNER.** Two gates walk `wat-scripts/` **recursively**:
`every_wat_scripts_file_loads_on_the_current_runtime` (parses and type-checks every file) and
`every_rete_name_in_wat_scripts_code_resolves`. Both will see these nine.

**Cure AT #537 — the #184 precedent — with the recorded chain, never by hand:**

    scripts/replay/convert.sh 628e6371d <out> <path>      # the file's own introducing commit

This is the batch-4o / 4q / 4r precedent for retired-era syntax on a non-corpus-migration site. **Do not
hand-edit the nine.** Afterwards run both gates above (N > 0, green) and record their verdicts.

⚠ If conversion leaves a file that still fails, that is a finding: **capture the checker's own message
verbatim, name the file, and STOP** — do not improvise a hand fix.

## ⚠ #540 — a `.sh` that may carry wat

`peragrare-census.sh` is a shell script in a tree where **wat inside string literals is the replay's most
persistent defect class** (finding 33, five consecutive code-bearing batches). Grep that side and SAY SO
in your SCORE; "not applicable" is an answer, silence is not.

## ⛔ GROK'S MEASUREMENTS ARE GROK'S (the 4s rule, still standing)

The vigilia prose states figures grok measured on grok's tree. **Land them as written.** A figure false
of THIS tree is an observation for your SCORE, never an edit to grok's text. Last batch proved the
divergence is real — grok's "all 25 counters" is 45 here.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture whole; name the assertion; STOP.
- ⛔ **READ EVERY COUNT OFF THE DATA**, and **exclude comments when counting code**.
- ⛔ **The record gate is PATH-BASED**: `^src/` needs `census:` + `nested-program-gate:`; `.rs` needs
  `lint-subset`, `kind(lib)`, `doctest`. **Neither code step here touches `src/` or `.rs`** — but both
  touch `wat-scripts/`, so run the loader gates and record what they said.
- ⛔ **NEVER `git filter-branch`** — detach / re-commit / rebuild-descendants only.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** (E11) — including every converted file.

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh 5f476ab4d HEAD 521 540

exit 0, plus `git replace -l`, `git for-each-ref refs/original/`, `git merge-base --is-ancestor
origin/replay/grok-rete HEAD`, `git status --porcelain`, and **a subject check for all 20 steps**. Then
write `SCORE-7t-replay-batch-4t.md` (every row, never blank) and a `REPLAY-LOG.md` section. Commit both.
**Leave the tree CLEAN. Yield by ending your turn — signal nothing.**
