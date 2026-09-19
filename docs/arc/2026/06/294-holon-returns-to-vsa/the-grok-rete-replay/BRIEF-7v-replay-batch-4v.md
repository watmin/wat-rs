# BRIEF 7v — replay batch 4v: grok-rete #561 → #580

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **560** REPLAY commits and a
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

Six trailers were fabricated across three batches; the last two batches had zero because their executors
piped them. Quote heredocs (`<<'EOF'`), read `git log -1 --format=%B` back, keep every verdict on ONE
physical line. ⚠ `git commit --amend` may be refused by the harness's classifier — repair via detach +
`git reset --soft HEAD^` + re-commit + cherry-pick-forward, proven with a tree diff.

## The work

**#561 → #580.** **Eighteen docs-only.** **Two code steps.** The range is **62 `.md` + 3 `.rs` + 2 `.sh`
+ 1 `.wat`**.

| N | C | files | note |
|---|---|---|---|
| 565 | `c84280de6` | 6 (5 docs, 1 `.sh`) | adds `tests/lint/peragrare-bad-census.sh` — a standalone instrument |
| 576 | `690c35522` | 6 (3 `.rs`, 1 `.wat`) | ⚠ the only real code step — see below |

**Pre-flighted and passing:** **ZERO hazard rows, ZERO new gates, ZERO `wat-scripts/fixes/` edits, ZERO
test delta** (no `#[test]`/`#[ignore]` moves anywhere). The floor prediction is **unchanged at 5872 run /
22 skipped** — verify it, do not inherit it.

## ⚠ #576 — small, but it touches three diverged files and the rune vocabulary

Its parts:
- `src/rete/clause.rs` (+6/−4) and `src/rete/eval_test.rs` (+2/−1) — **both already differ from grok's
  pre-image here.** Expect conflicts. **Compare DELTAS, not blobs** (finding 36).
- `wat/rete/oracle/fire.wat` — **comments-only, 3 lines** (measured: 3 comment, 0 code). The range's only
  `wat/` path and its only `stdlib-touch` row. If you find a non-comment line there, that contradicts the
  measurement: **STOP and report**.
- `tests/lint/no_unknown_ward_rune.rs` — **one word added to the vocabulary: `"shape-contract"`.** Our
  copy of this gate is **byte-identical to grok's pre-image**, so it should compose cleanly.
- `docs/CONVENTIONS.md` and `wat-scripts/perf/grid/run-all.sh`.

⛔ **Run the vocabulary gate at #576 and quote its verdict** (`-E 'test(no_unknown_ward_rune)'`, N > 0).
**This tree carries 7 excusare runes against grok's floor of 5** — a divergence that predates this batch
— so do not assume the gate's arithmetic matches grok's. If it reds, that is a finding: capture it
verbatim and STOP.

## ⚠ #565's `.sh` gates nothing — but finding 33 still reaches it

`tests/lint/peragrare-bad-census.sh` is a census instrument. **Measured: no `.rs` drives it**, so it is
prose with a shebang; and it cites `every_wat_bad_fixture_actually_fails.rs:242-301`, which
`no_stale_path_in_doc` does not check because that gate does not scan `.sh`. **Still sweep its string
literals for wat** (finding 33 has reached `.sh` before) and say so explicitly; "not applicable" is an
answer, silence is not.

## ⛔ GROK'S MEASUREMENTS ARE GROK'S (the standing rule)

The vigilia prose states figures grok measured on grok's tree. **Land them as written.** A figure false
of THIS tree is an observation for your SCORE, never an edit to grok's text.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture whole; name the assertion; STOP.
- ⛔ **READ EVERY COUNT OFF THE DATA**, and exclude comments when counting code.
- ⛔ **A CURE ANSWERS TO THE GATES OF ITS MEDIUM**: after a `.rs` edit, re-run the lint subset at that
  step.
- ⛔ **The record gate is PATH-BASED**: `^src/` needs `census:` + `nested-program-gate:`; `.rs` needs
  `lint-subset`, `kind(lib)`, `doctest`. **#576 needs all of them; #565 needs none** — say so explicitly
  per step rather than leaving the record silent.
- ⛔ **`census:` must be TRUE.** A step whose `src/` change alters `wat --check` for files it did not
  produce is a STOP-8 and a finding.
- ⛔ **NEVER `git filter-branch`** — detach / re-commit / rebuild-descendants only.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT.**

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh <batch-start> HEAD 561 580

(batch-start = the BRIEF commit's parent), exit 0, plus `git replace -l`, `git for-each-ref
refs/original/`, `git merge-base --is-ancestor origin/replay/grok-rete HEAD`, `git status --porcelain`,
and **a subject check for all 20 steps**. Then write `SCORE-7v-replay-batch-4v.md` (every row, never
blank) and a `REPLAY-LOG.md` section. Commit both. **Leave the tree CLEAN. Yield by ending your turn.**
