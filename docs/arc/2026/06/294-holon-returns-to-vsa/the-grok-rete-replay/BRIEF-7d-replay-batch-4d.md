# BRIEF 7d — replay batch 4d: grok-rete #221 → #225

> Ends BEFORE **#226** (a divergent stdlib MACRO + 5 shared `.rs`, the G1 class 2a4c closed) — that step
> gets its own brief.

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd` and
`git rev-parse --abbrev-ref HEAD`. HEAD should be batch 4c's tip (220 REPLAY commits, tree clean).
⛔ Do NOT touch `/home/john/work/holon/` (FROZEN) or `main`. **Never use worktrees. Never push. Do not
spawn subagents. Do NOT run `scripts/floor.sh`, clippy or run5** — the orchestrator weighs those centrally
and uncontended; a gate run while you work in the tree is a FALSE result.

⚠ `wat-rs/CLAUDE.md` does not reach an executor. The load-bearing doctrine is carried here.

## Doctrine — every line was paid for

- **R21:** `.wat` corpus rewrites go through a recorded wat-fix codemod (`wat-scripts/fixes/*.wat`), NEVER
  hand edits or sed. The one exception is wat embedded in `.rs`/`.sh` string literals, which no codemod
  reaches — hand-fix and LOG each edit. Scratch `.wat` → `wat-scripts/scratch-pad/`.
- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything — redirect to a file and read the
  file** (finding 28). "Capture the red verbatim" is unactionable if the RUN was truncated. BOTH sides of
  this merge hit it independently (our #190; grok's own #198, *"a trap door that is mine"*).
- ⛔ **THERE IS NO KNOWN FLAKE.** On any red: do NOT re-run; copy the whole stdout+stderr block verbatim;
  name the exact assertion; STOP. **An isolated re-run is the weakest evidence against a failure seen under
  load**, and on a timing test it answers a different question.
- ⛔ **Ending your turn ENDS you.** Run every verification in the FOREGROUND and block on it, however slow.
- ⛔ **After ANY commit or `--amend`: assert `git status --porcelain` is EMPTY and that the commit's own
  diff names every path its body claims** (finding 27). Read every COUNT off `git show --stat`/the diff —
  a count in a commit body is a measurement.
- ⛔ **Any repair must be visible to `push`** (finding 29). No `git replace`, no overlay, nothing that lives
  in a local ref. If a repair is blocked, STOP and report — do not reach for a mechanism whose result a
  fresh clone would not see.
- ⛔ **A mutation proof must falsify the proposition you rely on** (finding 30). Stripping a rune proves the
  GATE notices a missing rune, not that the rune's reason is the real cause. **`rc=1` says nothing about
  WHY — read the error text.**
- Every step, docs-only included, commits as `REPLAY(grok-rete #N): <C's subject>` with the
  `(cherry picked from commit <sha>)` trailer kept in the body (finding 26).

## The work

Replay **#221 → #225** (5 steps) per `BRIEF-1-pilot-first-ten-commits.md` § "One step", with every wall it
names and the five verdict lines where required.

| N | C | kind | files | note |
|---|---|---|---|---|
| 221 | `16f504e14` | shared | 4 | **THE TRAP — see below** |
| 222 | `a49b68608` | docs | 1 | |
| 223 | `cd2ab4b37` | docs | 1 | |
| 224 | `85043bbab` | docs | 1 | |
| 225 | `c8f1f7839` | docs | 3 | |

`absent-on-main.tsv` has **no row in this range** — no moved-home hazard.

## ⚠ #221 — the first MULTI-stdlib-file step since stone 2a4d

`fix(oracle): an accumulate result is SUPERSEDED, not extended — stop accreting the stale ones`. Four
files: `wat/rete/oracle/fire.wat` (+125), `wat/rete/oracle/stratify.wat` (+14), a new 210-line
`tests/rete/probe_arc278_oracle_accumulate_supersedes.wat`, and a new 106-line `.rs` test.

1. **TWO-PHASE stdlib convert (the 2a4 rule).** A step touching `wat/*.wat` converts in two phases: the
   stdlib files first with the door's stdlib mode, then **rebuild**, then the consumers. The stdlib half is
   baked (`include_str!`), so a stale binary measures the OLD world — `cargo build --release` must precede
   any probe or re-check after a `wat/` file changes.
2. **This is stone 2a4d's FIRST REAL EXERCISE.** 2a4d made the declaration door answer a step's files from
   ONE world, unioning the KEPT `TypeInfo` rows per member so one member's refusal cannot strip the world
   its siblings are answered from. #221 is the first step since with **two** stdlib files in one set. Read
   `convert.sh`'s output for both files rather than assuming the union behaved.
3. **STOP-9:** `convert.sh` reporting `UNREGISTERABLE` for any `wat/…` path is a STOP. Report it with the
   verbatim line; do not work around it.
4. **Finding 24 applies:** #221 adds a NEW `.rs` test file, so run the test-hygiene walls —
   `no_inlined_edn`, `no_loose_string_assert`, `no_inlined_wat`. They are triggered by the file's
   existence, not by the step's topic.
5. Every `.wat` the step produces passes `./target/release/wat --check`, and #221's named tests run by name.

## ⛔ STOP triggers — rejections, not permission to defer

STOP-1 … STOP-13 as BRIEF-1 states them; in particular **STOP-8** (a file going rc 0 → non-zero that the
step did not produce), **STOP-9** (above), **STOP-10** (stone 3's nested-program gate), **STOP-11**
(`kind(lib)` or doctests red).

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it. **Never
a repair commit after the batch, and never a knowingly-red REPLAY commit.** A fold's proof must name WHICH
paths it carries and be checked with `git diff <old-tip> <new-tip>`. A gate that did not exist at step N is
not a reason to fold into N (the #184 precedent); repair it where the gate lands.

## Tier

Commit each step on green. **Do not push.** Before yielding, run in the FOREGROUND and paste the output:

    scripts/replay/verify-step-record.sh <batch-start> HEAD 221 225

It must exit 0 — the range form requires exactly one `REPLAY(grok-rete #N)` per N and cross-checks each
commit's cited source against `commits.tsv`. Yield after #225, or at the first STOP, with
`SCORE-7d-replay-batch-4d.md` and a `REPLAY-LOG.md` section, leaving the tree CLEAN.
