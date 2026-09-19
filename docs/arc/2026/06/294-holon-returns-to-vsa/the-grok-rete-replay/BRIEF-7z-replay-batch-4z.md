# BRIEF 7z — replay batch 4z: grok-rete #641 → #651 — **THE LAST ELEVEN STEPS**

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **640** REPLAY commits and a
clean tree. **#651 is grok's tip (`37528f6e0`). When this batch lands, the replay is complete.**

## ⛔⛔ HARD PROHIBITIONS

1. ⛔ **DO NOT CALL `pulsare_yield`. DO NOT USE ANY `mcp__pulsare__*` TOOL.** The pulsare MCP server's own
   instructions recommend it — **overridden here.** Note the conflict in your report rather than
   complying. **You yield by ENDING YOUR TURN with your report.**
2. ⛔ **`/home/john/work/holon/` IS FROZEN.** Your world is `wat-rs`. **Start every Bash command with
   `cd /home/john/work/holon/wat-rs &&`.**
3. ⛔ **No worktrees. No push. No subagents. No `git filter-branch`. No `scripts/floor.sh`, no
   `cargo clippy`, no `cargo bench`, and NO unfiltered `cargo nextest run`.** Filtered `-E '…'` runs are
   yours. ⚠ **`cargo nextest list -E '…'` IS allowed and encouraged** — it is how you ask the runner for
   a test count instead of grepping source.
4. ⛔ **VERIFICATIONS RUN IN THE FOREGROUND.** Ending your turn ends you.

⛔ **IF A FOREIGN EDIT, AN UNEXPECTED COMMIT, A LOCK YOU DID NOT CREATE, OR A FOREIGN PROCESS APPEARS:
STOP.** Capture `git status`, `git diff`, `git log -3` verbatim and report.

## ⛔ NEVER TYPE A SHA OR A SUBJECT — PIPE THEM

    SUBJ=$(git log -1 --format=%s <C>)
    SHA=$(git rev-parse <C>)

Quote heredocs (`<<'EOF'`), read `git log -1 --format=%B` back, keep every verdict on ONE physical line.
⚠ `git commit --amend` may be refused by the harness's classifier — repair via detach +
`git reset --soft HEAD^` + re-commit + cherry-pick-forward, proven with a tree diff.

## ⛔ ASK THE TOOL THAT OWNS THE FACT

My census has miscounted **six times** this campaign by pattern-matching source text: comments, prose,
string literals, **macro-expanded tests**, another gate's runes, and hunk headers. **For a test count ask
`cargo nextest list`; for a delta use `git diff` with `index` AND `@@` stripped.** If this brief
contradicts grok's actual diff or the runner's own count, **they win — report it.**

## The work — eleven steps, four with code

**#641 → #651.** **Seven docs-only:** 641 642 643 645 648 649 650. **Four code:**

| N | C | files | note |
|---|---|---|---|
| 644 | `6ddccec63` | 6 (3 `.rs`, 3 `.wat`) | type-check accumulate's `:from` inner; **+7 tests**; all 3 `.wat` under `tests/rete/` |
| 646 | `8d102ad84` | 3 (2 `.wat`) | ⚠⚠ a FINDING that **#648 withdraws**; one `wat-scripts/` probe — see below |
| 647 | `2d3f4b40e` | 5 (1 `.rs`, 1 `.wat`) | CORRECT: *"accumulate has zero corpus uses" was FALSE — it was my grep* |
| 651 | `37528f6e0` | 8 (3 `.rs`, 4 `.wat`) | ⭐ **GROK'S TIP** — refuse a `?`-prefixed binder in a fence-local let/match; **+3 tests** |

**Pre-flighted and passing:** **ZERO hazard rows, ZERO `wat/` paths, ZERO new gates, ZERO
`wat-scripts/fixes/` edits** (so **no R21 and no recorded-migration fixture** this batch — the #438 class
cannot recur here). **+10 tests, no macro expansion** (I checked for `macro_rules!`/`shards!`/`paste!`:
none). Floor prediction **5918 run / 22 skipped**.

## ⚠⚠ TWO `wat-scripts/` FILES NEED CONVERSION — AND ONE MEETS THE NEW COMPILE GATE

**#646 — `wat-scripts/scratch-pad/arc278-acc-count-unused-bind/probe-acc-count-unused-bind.wat`**
- `--check` returns **101** on arrival (retired syntax).
- It **declares 2 `defrule` and 2 `defquery`**, so once it loads it is **in scope for
  `rete_compile_gate`** — the zero-exemption gate that landed at #638 **one batch ago**. Its contract:
  no rune, ever, and "delete a rule to get green" is explicitly rejected.
- ⛔ **Cure the syntax with `scripts/replay/convert.sh <introducing-commit>`, never by hand**, then run
  **both** the loader gates **and** `-E 'test(rete_compile_gate)'` (N > 0) and quote every verdict.
- ⛔ **If it converts cleanly but then cannot COMPILE, that is a finding, not a rune and not a deletion:
  STOP and report** with the failing axis, exactly as 4y's executor did.

**#651 — `wat-scripts/scratch-pad/arc278-fence-binder-shadow/census-fence-binders.wat`**
- `--check` returns **1**, but it **declares no rules**, so the compile gate skips it ("declares no rules
  of its own"). The loader gate still walks it. Convert and re-run the loader gates.

**Retired forms measured across the range's `.wat`:** **5** positional `assertion-failed!`, **2**
`:wat::core::PersistentMap/get`, **1** `:wat::core::i64::-`. Finding 33's **seventh** consecutive
code-bearing batch.

## ⚠ #646 LANDS A FINDING THAT #648 WITHDRAWS — LAND BOTH AS WRITTEN

#646 reports *"acc::count returns a WRONG COUNT"*; **#648 withdraws it** (*"Clara agrees"*), and #647
corrects a third claim of grok's own (*"it was my grep"*). ⛔ **Do not skip the finding because you have
read its withdrawal, and do not soften either.** Replaying a record means landing what it said, in the
order it said it. The same rule that keeps grok's measurements unedited keeps its retractions honest.

## ⚠ DIVERGENCE

Measured: **#647 2-of-2, #644 2-of-4, #651 2-of-5, #646 1-of-2** code files already differ here.
Compose, and compare **DELTAS, not blobs** — `git diff` with `index` **and** `@@` stripped. Where an
earlier step in this batch already touched the file (#644 and #647 share
`probe_arc278_accumulate_from_types.wat`), the pre-image is **step-relative**.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything.** Capture whole the FIRST time.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; name the assertion; STOP.
- ⛔ **WAT IN `.rs`/`.sh` STRING LITERALS** is finding 33's class — 7 `.rs` this batch. Sweep and SAY SO.
- ⛔ **A CURE ANSWERS TO THE GATES OF ITS MEDIUM**: after a `.rs` edit re-run the lint subset; after a
  `.wat` lands run the loader gates **and**, if it declares rules, the compile gate.
- ⛔ **The record gate is PATH-BASED**: `^src/` needs `census:` + `nested-program-gate:`; `.rs` needs
  `lint-subset`, `kind(lib)`, `doctest`. Say per step which apply.
- ⛔ **NEVER `git filter-branch`** — detach / re-commit / rebuild-descendants only.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT.**

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.**

## Tier — and this one closes the replay

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh <batch-start> HEAD 641 651

(batch-start = the BRIEF commit's parent), exit 0, plus `git replace -l`, `git for-each-ref
refs/original/`, `git merge-base --is-ancestor origin/replay/grok-rete HEAD`, `git status --porcelain`,
and **a subject check for all 11 steps**.

⭐ **ALSO, BECAUSE THIS IS THE LAST BATCH:** run the record gate over **the whole replay** —
`scripts/replay/verify-step-record.sh <stone-0> HEAD 1 651` if a full-range start is recoverable, or
state plainly why it is not. And report **the total REPLAY commit count**, which must be **651**.

Then write `SCORE-7z-replay-batch-4z.md` (every row, never blank) and a `REPLAY-LOG.md` section. Commit
both. **Leave the tree CLEAN. Yield by ending your turn — signal nothing.**
