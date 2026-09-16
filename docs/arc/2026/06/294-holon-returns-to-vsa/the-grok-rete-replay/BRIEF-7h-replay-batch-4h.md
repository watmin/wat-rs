# BRIEF 7h — replay batch 4h: grok-rete #281 → #300

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd` and
`git rev-parse --abbrev-ref HEAD`. Expect **280** REPLAY commits and a clean tree.
⛔ Do NOT touch `/home/john/work/holon/` (FROZEN) or `main`. **Never use worktrees. Never push. Do not
spawn subagents. Do NOT run `scripts/floor.sh`, clippy or run5** — the orchestrator weighs those centrally
and uncontended; a gate run while you work in the tree is a FALSE result.

⚠ `wat-rs/CLAUDE.md` does not reach an executor. The load-bearing doctrine is carried here.

## Doctrine — each line was paid for

- **R21:** `.wat` corpus rewrites go through a recorded wat-fix codemod (`wat-scripts/fixes/*.wat`), NEVER
  hand edits or sed. Scratch `.wat` → `wat-scripts/scratch-pad/`.
- ⛔ **WAT EMBEDDED IN `.rs`/`.sh` STRING LITERALS IS R21's EXCEPTION — AND THIS REPLAY'S MOST PERSISTENT
  DEFECT SOURCE** (finding 33). No codemod reaches it, `convert.sh` never sees it, no gate parses it. It has
  bitten at #162, #167, #238, #242 and #266. **When a step's subject mentions a rename or rehome — or when
  a step ADDS a `.rs` file containing wat program strings — grep that side too and SAY SO in the log.**
  "Not applicable" is an answer; silence is not.
- ⛔ **A rename census is built from the RECORDED MIGRATIONS, not the shape of a name** (finding 33).
  Check `wat-scripts/fixes/rename-*.wat` before calling a spelling stale.
- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28). Redirect to a
  file and read the file.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** On any red: do NOT re-run; copy the whole stdout+stderr
  block verbatim; name the exact assertion; STOP and report.
- ⛔ **Ending your turn ENDS you.** Every verification in the FOREGROUND, blocking.
- ⛔ **READ EVERY COUNT OFF THE DATA, AT THE MOMENT YOU WRITE THE SENTENCE** (findings 27, 34). This is the
  most recurrent defect in this replay and **a disclosure is not a cure**: batch 4f's SCORE disclosed the
  class at #242 and then committed it again in its own E11; batch 4g's REPLAY-LOG says "59 names across 9
  files" where the census says 63 rune lines / 44 distinct names / 11 files. After ANY commit: assert
  `git status --porcelain` EMPTY and that the diff names every path the body claims.
- ⛔ **The record gate's rule is PATH-BASED, not `.wat`-triggered.** A `^src/` change needs `census:` +
  `nested-program-gate:` EVEN IF every file is `.rs`. A `.rs` change needs `lint-subset`, `kind(lib)`,
  `doctest`. Misreading this cost batch 4f two rebuilds.
- ⛔ **Each verdict line must be ON ONE LINE** (finding 31). Batch 4g wrote
  `census: <f> files=N; --diff (vs #M's <f2>) no STOP-8` — the parenthetical between `--diff` and
  `no STOP-8` broke the gate's contiguous match. **Put any such detail AFTER `no STOP-8`.**
- ⛔ **ANY REPAIR MUST BE VISIBLE TO `push`** (finding 29). No `git replace`, no overlay.
- ⛔ **A mutation proof must falsify the proposition you rely on** (finding 30). `rc=1` says nothing about WHY.
- **Every step, docs-only included,** commits as `REPLAY(grok-rete #N): <C's subject>` with the
  `(cherry picked from commit <sha>)` trailer. Keeping grok's own subject makes the step INVISIBLE to the
  record gate — that cost history surgery at #160/#161.

## ⛔⛔ TWO NEW RULES, both paid for by batch 4g

**1. HISTORY-REWRITE MECHANISM.** 4g repaired four wrapped verdict lines with `git filter-branch
--msg-filter`. It was safe, but only because one rev-range argument was correct: filter-branch is a
WHOLE-HISTORY rewriter, and a missing range would have silently rewritten **280 PUSHED commits**, with the
next push publishing a divergent history to the DR site.

- **PREFER the detach / re-commit / rebuild-descendants pattern** (batch 4f's). It cannot reach below its
  own start point. Use it for any body repair.
- **If you use `filter-branch` anyway, the proof obligation is explicit and goes IN THE SCORE**:
  `git merge-base --is-ancestor origin/replay/grok-rete HEAD` succeeds, and `git for-each-ref
  refs/original/` is EMPTY. ⛔ **A tree-hash comparison does NOT establish this** — it speaks to content,
  never to the commit graph.

**2. A SCORE's GREEN MUST DISCLOSE WHAT BOUGHT IT.** 4g's E8 reported #278's gate green without saying the
green came from 63 rune declarations added at landing; #270's 8 ledger deletions and #278's `attested()`
exclusion appeared only in the REPLAY-LOG. That is disclosure PLACEMENT, not concealment — but the SCORE
is what gets scored. **Any landing-time action that changes a gate, its inputs, or its ledger must appear
in the SCORE row that gate satisfies**, not only in the log.

## The work

Replay **#281 → #300** (20 steps) per `BRIEF-1` § "One step". **Fifteen are docs-only:**

    281 282 284 285 286 287 289 290 291 292 293 295 296 297 299

**Five carry code:**

    283 288 294 298 300

That is 15 + 5 = 20. **Count both lists against the census rather than trusting this sentence.**

| N | C | files | note |
|---|---|---|---|
| 283 | `2c7200802` | 28 (28 `.rs`) | ⚠ **THE TRAP — a 913-line new gate. See below.** |
| 288 | `9d4b68088` | 2 (1 `.rs`, 1 `.md`) | two ward rune vocabularies copied in and gated |
| 294 | `00ca6b0eb` | 7 (7 `.rs`) | modifies `rete_citation_resolves.rs`, **created at #283** |
| 298 | `073546093` | 3 (2 `.rs`, 1 `.md`) | ⚠ **NOT docs-only** — grok's own #299 corrects this |
| 300 | `b41a63672` | 1 (1 `.rs`) | declares the deleted `EXEC_SP` so the citation gate can see it |

**This range is unusually clean: ZERO hazard rows of any kind** — no stdlib-touch, no `absent-on-main`, no
`future-macro-changes`, no `main-deleted` — and **not one step touches `wat/`**. The next two-phase stdlib
step is still #377. **There are no `.wat` files anywhere in the range**, so no `--check` work and no
codemod work is expected; if you find yourself running `convert.sh`, stop and re-read the diff.

## ⚠⚠ #283 — THE THIRD NEW GROK LINT GATE, AND IT WILL LAND RED

`tests/lint/rete_citation_resolves.rs` (913 lines) demands that **every backticked identifier and every
bare `*.rs`/`*.wat` filename cited in a comment under `src/rete/`** either RESOLVE or carry, in place:

    // rune:lint(cited-name-absent) <the name> — <why it is absent, and what the reader should know>

with a **40-character** reason floor. The other 27 files in #283's diff are **grok repairing its own
citations**. Our corpus diverged by the same renames and module splits that forced #270's ledger reseed at
batch 4g, so **our unresolved set will be different and probably larger.**

**Repair AT #283** — the #184 precedent. Never fold backward, never weaken or allowlist the gate.

⛔ **NEVER REWORD A CORRECT CITATION TO DODGE A RED.** The gate's own header warns that too narrow a
universe MANUFACTURES findings: six names cited in rete comments are legitimately attested only outside
`src/` (e.g. `spec_equals_native_on_every_where_family` is a `tests/rete/` fn). If a citation is right and
the gate is wrong about it, that is a finding to REPORT, not a comment to rewrite.

⛔ **A rune is a DECLARATION, not a suppression.** Each must name the exact token and a real mechanism —
batch 4g's 63 runes cite file:line for each. A blanket or vague reason is refused by the gate's own
`hollow` arm, and a rune covering undeclared rot is finding 30's defect.

**The pattern is three for three** — #274 landed 9 undeclared + 1 hollow; #278 landed 63 rune declarations
across 11 files; #283 is predicted. Budget the repair time.

## ⚠ #298 and #299 — grok corrects its own classification

#299's subject is *"curare: correction — 073546093 is not docs-only, it carries unweighed …"*. Our census
agrees: **#298 carries 2 `.rs`** and is a CODE step needing the full path-based verdict lines. Do not
classify #298 from its `curare:` subject.

## ⚠ #294 depends on #283

#294 modifies `tests/lint/rete_citation_resolves.rs`, which #283 creates. If it is missing when you reach
#294, #283 did not land correctly — **STOP and report rather than creating the file yourself.**

## ⛔ EVERY `-E` FILTER MUST SELECT A NON-ZERO COUNT

A nextest filterset matching nothing runs zero tests and **exits 0**. Read each run's own `N tests run`
line and require **N > 0**.

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.** Prove a fold with
`git diff <old-tip> <new-tip>` naming which paths it carries. **A gate that did not exist at step N is not
a reason to fold into N — the #184 precedent — and #283 is exactly that case.**

## Tier

Commit each step on green. **Do not push.** Before yielding, run in the FOREGROUND and paste:

    scripts/replay/verify-step-record.sh <BATCH-START-SHA> HEAD 281 300

It must exit 0. Yield after #300, or at the first STOP, with `SCORE-7h-replay-batch-4h.md` answering every
row, and a `REPLAY-LOG.md` section, leaving the tree CLEAN.
