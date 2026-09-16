# BRIEF 7e — replay batch 4e: grok-rete #226 → #240

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd` and
`git rev-parse --abbrev-ref HEAD`. HEAD should be batch 4d's records tip; 225 REPLAY commits; tree clean.
⛔ Do NOT touch `/home/john/work/holon/` (FROZEN) or `main`. **Never use worktrees. Never push. Do not
spawn subagents. Do NOT run `scripts/floor.sh`, clippy or run5** — the orchestrator weighs those centrally
and uncontended; a gate run while you work in the tree is a FALSE result.

⚠ `wat-rs/CLAUDE.md` does not reach an executor. The load-bearing doctrine is carried here.

## Doctrine — each line was paid for

- **R21:** `.wat` corpus rewrites go through a recorded wat-fix codemod (`wat-scripts/fixes/*.wat`), NEVER
  hand edits or sed. The one exception is wat embedded in `.rs`/`.sh` string literals — hand-fix those and
  LOG each edit. Scratch `.wat` → `wat-scripts/scratch-pad/`.
- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything — redirect to a file and read the
  file** (finding 28). "Capture the red verbatim" is unactionable if the RUN was truncated.
- ⛔ **THERE IS NO KNOWN FLAKE.** On any red: do NOT re-run; copy the whole stdout+stderr block verbatim;
  name the exact assertion; STOP. An isolated re-run is the weakest evidence against a failure seen under
  load.
- ⛔ **Ending your turn ENDS you.** Every verification in the FOREGROUND, blocking, however slow.
- ⛔ **After ANY commit or `--amend`: assert `git status --porcelain` is EMPTY and that the commit's diff
  names every path its body claims** (finding 27). Read every COUNT off `git show --stat`/the diff.
- ⛔ **ANY REPAIR MUST BE VISIBLE TO `push`** (finding 29). No `git replace`, no overlay, nothing in a local
  ref. If a repair is blocked, STOP and report.
- ⛔ **A mutation proof must falsify the proposition you rely on** (finding 30). `rc=1` says nothing about
  WHY — read the error text.
- ⛔ **Each verdict line must be ON ONE LINE** (finding 31). `verify-step-record.sh` matches within a single
  line; a wrapped `census: … --diff no STOP-8` fails a wall that genuinely ran. Unwrap the line — never
  reword the claim to satisfy the gate.
- **Every step, docs-only included, commits as** `REPLAY(grok-rete #N): <C's subject>` with the
  `(cherry picked from commit <sha>)` trailer in the body (finding 26). ✅ **BRIEF-1's item 1 was AMENDED
  2026-09-16** and now reads `git cherry-pick -x --no-commit C` — follow it as written; it no longer
  conflicts with item 4.

## The work

Replay **#226 → #240** (15 steps) per `BRIEF-1-pilot-first-ten-commits.md` § "One step", every wall, with
the verdict lines each step's own diff requires.

| N | C | kind | note |
|---|---|---|---|
| 226 | `7319c1ea4` | shared, 6 files | **THE TRAP — see below** |
| 227 228 229 | | docs | |
| 230 | `bb0256e38` | shared, 2 files (2 `.rs`) | |
| 231 232 | | docs | |
| 233 | `99120fc8b` | shared, 4 files (**1 `.wat`, 0 `.rs`**) | census + nested-gate lines required; **no `.rs` walls** |
| 234 | `057f9d494` | code, 2 files (2 `.rs`) | |
| 235 236 237 | | docs | |
| 238 | `17fc5fb3e` | shared, 3 files (3 `.rs`) | |
| 239 240 | | docs | |

**No `absent-on-main.tsv` row in this range** (no moved-home hazard) and **no stdlib-touch row after #226** —
the next two-phase step is #377.

⚠ **#233 is the path-based trap.** It carries a `.wat` but no `.rs`, so the record gate requires
`census: … --diff no STOP-8` and `nested-program-gate: PASS` but NOT `lint-subset`/`kind(lib)`/`doctest`.
The gate derives requirements from the commit's own diff — an executor misjudged exactly this distinction
at #215 and the gate caught it.

## ⚠ #226 — a DIVERGENT STDLIB MACRO, plus five shared `.rs`

`fix(rete): with-network's scope is closed by a Drop, not a release call`. Six files: `wat/rete/syntax.wat`
(the stdlib half), `src/check.rs`, `src/runtime.rs`, `src/rete/kernel/arm.rs`, `src/rete/purity.rs`, and
`src/rete/kernel/tests/arm_lease.rs`.

1. **TWO-PHASE stdlib convert (the 2a4 rule):** convert the `wat/` file first with the door's stdlib mode,
   then **`cargo build --release`**, then the consumers. `wat/` is `include_str!`ed — any probe or `--check`
   before that rebuild measures the OLD world.
2. **It changes a stdlib MACRO** (`:wat::core::defmacro :wat::rete::defquery`) — finding 19's **G1 class**,
   closed by stone **2a4c** (`retract_divergent_stdlib_macros`). It is the only entry in
   `future-macro-changes.txt` in this range. The class has been exercised before (#155/#157/#159), so this
   is not first-of-kind — but read `convert.sh`'s output for the file rather than assuming.
3. **STOP-9:** `convert.sh` reporting `UNREGISTERABLE` for any `wat/…` path. Report the verbatim line.
4. ⚠ **`--check` on a `wat/` stdlib file is UNSATISFIABLE BY CONSTRUCTION** — it defines into `:wat::`,
   which the plain loader `--check` uses refuses categorically. Do NOT treat its `rc=1` as a defect and do
   NOT invent a pass. If you need to show the step did no harm, **disprove it the way 4d did**: run
   `--check` on the file's pre-step content and on the post-step content from identical path shapes and
   compare the `ReservedPrefix` counts; the delta should equal the defns the step adds. The real gate is
   `cargo build --release` re-embedding the content plus any test that boots the runtime.
5. `src/check.rs` and `src/runtime.rs` are main's most-diverged files — expect re-expression, not clean
   application, and log each hand edit.

## ⛔ STOP triggers — rejections, not permission to defer

STOP-1 … STOP-13 per BRIEF-1; in particular **STOP-8** (a file going rc 0 → non-zero the step did not
produce), **STOP-9** (above), **STOP-10** (stone 3's nested-program gate), **STOP-11** (`kind(lib)` or
doctests red).

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it. **Never
a repair commit after the batch, never a knowingly-red REPLAY commit.** A fold's proof must name WHICH
paths it carries, checked with `git diff <old-tip> <new-tip>`. A gate that did not exist at step N is not a
reason to fold into N (the #184 precedent) — repair it where the gate lands.

## Tier

Commit each step on green. **Do not push.** Before yielding, run in the FOREGROUND and paste the output:

    scripts/replay/verify-step-record.sh <batch-start> HEAD 226 240

It must exit 0. Yield after #240, or at the first STOP, with `SCORE-7e-replay-batch-4e.md` and a
`REPLAY-LOG.md` section, leaving the tree CLEAN.
