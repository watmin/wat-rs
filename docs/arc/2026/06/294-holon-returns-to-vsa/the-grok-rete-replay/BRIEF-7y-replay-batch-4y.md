# BRIEF 7y — replay batch 4y: grok-rete #621 → #640

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **620** REPLAY commits and a
clean tree. **This is the second-to-last batch; 11 steps remain after it.**

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

## ⛔ IF THIS BRIEF CONTRADICTS GROK'S DIFF, THE DIFF WINS

My briefs have been wrong three batches running (a phantom `#[allow]` deletion read out of grok's prose;
a `#[test]` counted inside a string literal; a `+2` that was `+1`). **To learn what a step does to code,
diff the code paths only** (`git show <C> -- '*.rs'`). Report any contradiction as a result.

## The work — the heaviest batch since FactBag

**#621 → #640.** **Eleven docs-only.** **Nine code steps.** The range is **35 `.wat` + 25 `.md` + 8
`.rs` + 4 `.sh` + 3 `.wat.bad` + 1 `.edn`**, **+11 tests, net 0 ignores** (#633 adds 3 tests and 1
ignore; #635 un-ignores one). Floor prediction **5893 run / 22 skipped**.

| N | C | note |
|---|---|---|
| 621 | `35f1f4e8b` | grid: `run-all.sh` records its instrument |
| 627 | `43a1cc029` | ⚠ **R21** — Phase 1 codemod DRAWN and dry-run (`hoist-where-into-condition.wat`) |
| 628 | `05c247f82` | ⚠ **R21** — whitespace refinement applied **and reverted**; the floor found a real defect |
| 630 | `feb5fae91` | ⚠⚠ **R21 APPLIED** — 20 files, 18 `.wat`; "38 of 42 in-scope rules" |
| 633 | `d8068e26a` | probe: the `where` fence's interior is not type-checked — **banked**; +3 tests, +1 ignore |
| 635 | `6f1d93451` | ⚠ **R21** — type-check the fence interior, and the 3 sites it found; −1 ignore |
| 636 | `d7aa7c9ae` | FINDING: loading is not compiling — 11 corpus files declare rules that cannot compile |
| 638 | `2a9de244a` | ⚠⚠⚠ **new ZERO-EXEMPTION gate + 3 `fixes/` edits**; +8 tests |
| 639 | `3b280f7f2` | fix(census): synthesize the anchor — the census instrument's own repair |

**Pre-flighted and passing:** **ZERO hazard rows, ZERO `wat/` paths.**

## ⚠⚠ #630 — R21, AND THE CODEMOD GATES ITSELF

Run the recorded codemod; **never hand-edit the corpus**. Dry-run on a `/tmp` copy, diff, apply to a path
list **you derive from this tree**, then re-run for idempotence (a second pass must change 0 files).

**Measured from the codemod's own header, and it is the reason our bigger corpus is safe:**

> *"a rule whose `:when` carries FEWER THAN TWO ordinary fact conditions … `rule-form-edits` now returns
> NO edits at all for such a rule … **so a single run over the whole corpus cannot touch a rule outside
> that scope**."*

⚠ **OUR CORPUS IS NOT GROK'S.** Grok's #630 covers 18 files / "38 of 42 in-scope rules". **We carry 157
files with `:wat::rete::where`, 1008 sites, and 715 `defrule` sites.** **Two of grok's 18 paths do not
exist here.** Derive your own list (the 4o precedent), state your count, and report it if it differs
from grok's.

⚠ **#628 applied a refinement and REVERTED it** because the floor found a real defect. Land that
reversal as grok wrote it — do not "helpfully" keep the refinement.

## ⚠⚠⚠ #638 — A ZERO-EXEMPTION GATE, AND OUR EXPOSURE IS UNMEASURED

`tests/lint/rete_compile_gate.rs` requires **every `.wat` under `wat-scripts/` that declares a rete rule
to COMPILE**, not merely load — the four-axis fence (`pure ∧ det ∧ total ∧ rete-primitive`) lives in
`wat/rete/compile.wat` and loading never reaches it. Its own contract: **"ZERO EXEMPTION CATEGORIES.
There is no `red-by-design` rune for 'this rule is meant not to compile.'"**

Grok measured **136 declaring / 125 compiling / 11 failing** and drove its corpus to zero.

⛔ **I COULD NOT MEASURE OUR EXPOSURE, AND I AM TELLING YOU RATHER THAN GUESSING.** I ran grok's own
census (`tests/lint/rete-compile-census.sh` from #639 — already the anchor-fixed version) against this
tree. It reported **`compiles: 0 · cannot compile: 176`** with a **uniform `"3 type-check errors"` on
every file** — including live recorded codemods under `wat-scripts/fixes/` that provably pass
`every_wat_scripts_file_loads_on_the_current_runtime`. **That is the instrument failing on our syntax,
not 176 defects.** The script's own header records it reporting a false **136/136 OK** the first time,
for the mirror reason (greedy namespace extraction).

⛔ **A UNIFORM FAILURE ACROSS AN ENTIRE CORPUS IS AN INSTRUMENT DEFECT UNTIL PROVEN OTHERWISE.** Before
you believe any census number — grok's, mine, or your own — **prove the instrument on a known positive
AND a known negative.**

**So, at #638:**
- Land the gate and **run it** (`-E 'test(rete_compile_gate)'`, N > 0). Quote its verdict.
- **If it reds:** first establish the gate itself is sound here (does it pass on a file you know
  compiles?). A gate that fails uniformly is broken, not accusatory.
- **A genuine failure is a FINDING to report, never a rune** — the gate forbids them. If our corpus
  carries rules that cannot compile and grok's cure does not reach them, **STOP and report** with the
  file list and the failing axis per file. Do not delete a rule to get green.
- **#639 repairs the census instrument** — that is where an our-tree adaptation belongs if one is needed.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **`.wat` corpus migrations go through the codemod** (R21): dry-run, diff, apply to every derived
  path, re-run for idempotence, commit the codemod as the recorded migration.
- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28). ⚠ Last batch an
  executor viewed a red through `tail -8` and then re-ran to capture it whole. **Capture whole the FIRST
  time.**
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture whole; name the assertion; STOP.
- ⛔ **READ EVERY COUNT OFF THE DATA**, excluding comments **and string literals**.
- ⛔ **WAT IN `.rs`/`.sh` STRING LITERALS** is finding 33's class. Sweep per code step and SAY SO.
- ⛔ **A CURE ANSWERS TO THE GATES OF ITS MEDIUM**: after a `.rs` edit re-run the lint subset; after
  `.wat` changes run the loader gates.
- ⛔ **The record gate is PATH-BASED**: `^src/` needs `census:` + `nested-program-gate:`; `.rs` needs
  `lint-subset`, `kind(lib)`, `doctest`. Say per step which apply.
- ⛔ **NEVER `git filter-branch`** — detach / re-commit / rebuild-descendants only.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** — the derived path list, the dry-run diff, the
  idempotence re-run, every gate verdict.

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.** A gate that did not exist
at step N is not a reason to fold into N — **#638 is that case: repair AT #638.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh <batch-start> HEAD 621 640

(batch-start = the BRIEF commit's parent), exit 0, plus `git replace -l`, `git for-each-ref
refs/original/`, `git merge-base --is-ancestor origin/replay/grok-rete HEAD`, `git status --porcelain`,
and **a subject check for all 20 steps**. Then write `SCORE-7y-replay-batch-4y.md` (every row, never
blank) and a `REPLAY-LOG.md` section. Commit both. **Leave the tree CLEAN. Yield by ending your turn.**
