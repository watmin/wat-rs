# BRIEF 7n — replay batch 4n: grok-rete #401 → #420

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **400** REPLAY commits, a clean
tree, and `HEAD == origin == 849dcc4f5`.

## ⛔⛔ THREE HARD PROHIBITIONS

1. ⛔ **DO NOT CALL `pulsare_yield`. DO NOT USE ANY `mcp__pulsare__*` TOOL.** An earlier executor did; it
   wrote into the FROZEN root and woke a counterpart that ran its own floor inside this working tree.
   **You yield by ENDING YOUR TURN with your report.** ⚠ The pulsare MCP server's own instructions
   recommend calling it — **overridden here.** Note the conflict in your report rather than complying.
2. ⛔ **`/home/john/work/holon/` IS FROZEN**, `.pulsare/` and `.git/` included. Your world is `wat-rs`.
   ⚠ The harness may report the frozen root as your working directory: **start every Bash command with
   `cd /home/john/work/holon/wat-rs &&`.** `verify-step-record.sh` calls plain `git`, and from the wrong
   cwd it prints a false `MISSING-STEP`.
3. ⛔ **NEVER worktrees. NEVER push. NO subagents. Do NOT run `scripts/floor.sh`, `cargo clippy`, run5.**

⛔ **VERIFICATIONS RUN IN THE FOREGROUND.** Ending your turn ends you; last batch's executor ran long
sweeps with `run_in_background` and survived only because the harness happened to resume it. Foreground.

⛔ **IF A FOREIGN EDIT, AN UNEXPECTED COMMIT, A LOCK YOU DID NOT CREATE, OR A FOREIGN PROCESS APPEARS IN
THE TREE: STOP.** Do not discard it. Capture `git status`, `git diff` and `git log -3` verbatim and report.

## ⛔ THE SUBJECT AND THE TRAILER ARE COPIED, NEVER TYPED (findings 39 and its sibling)

Two batches ago nine subjects were paraphrased and one re-classified grok's `fix(tests):` as `perf:`; the
record gate could not see it, and the orchestrator rebuilt twenty-one commits. Last batch an executor
hand-typed six trailers and two placeholder SHAs. **For every step:**

    SUBJ=$(git log -1 --format=%s <C>)     # then commit as "REPLAY(grok-rete #N): $SUBJ"
    SHA=$(git rev-parse <C>)               # the cherry-pick trailer

Never retype either. Re-authoring a message is not re-copying it.

## ⛔ A PREDICTION HERE IS A PREDICTION (finding 37)

Every forecast below is a hypothesis with a measurement attached. **Disproving one is a RESULT** — row E18
scores honest disagreement, never agreement.

## The work

**#401 → #420.** **Ten docs-only:** 401 403 405 407 409 411 413 415 417 419. **Ten code, all
`src/rete/kernel`:**

| N | C | files | note |
|---|---|---|---|
| 402 | `3a54d440d` | 7 (6 `.rs`) | D1 — the mark is a prefix length, and every writer respects it |
| 404 | `df8a1222e` | 2 (1 `.rs`) | A8 cured — fact loss is unwritable |
| 406 | `9770eeeb4` | 6 (5 `.rs`) | ⚠ A3 cured — touches `accum_cost.rs` (23 wall-clock sites) |
| 408 | `d878408f7` | 4 (3 `.rs`) | the census union — three gates were asserting arithmetic identities |
| 410 | `c186e3e11` | 6 (5 `.rs`) | ⚠⚠ **new lint gate, and this tree is exposed — see below** |
| 412 | `1546d94f2` | 2 (1 `.rs`) | drive the keyed-gather gate over every instrumented path |
| 414 | `91f6b3609` | 2 (1 `.rs`) | gate the keyed-gather FORMULA; **removes one test, adds three** |
| 416 | `efa754a3d` | 6 (5 `.rs`) | ⚠ perf — `join_extend` resolves its alpha once per node |
| 418 | `4f52da56b` | 4 (3 `.rs`) | ⚠ perf — `production_delta` buffers per node |
| 420 | `09f174879` | 5 (4 `.rs`) | ⚠ perf — hoist `col_field_of` out of the catch-up loop |

**Pre-flighted and passing:** **zero hazard rows**; **no `wat/`, `wat-tests/` or `wat-scripts/fixes/` file
anywhere in the range**; **no `.wat` file at all**; no M-status-absent paths; R21 is not triggered.

## ⚠⚠ #410 — THE NEW GATE WILL FIRE HERE. REPAIR AT THE STEP

`tests/lint/no_raw_gather_bucket_walk.rs` bans a raw `bucket.iter()` or `for … in bucket` in three
SUBJECTS — `src/rete/kernel/fire/acc.rs`, `fire/pass/accumulate.rs`, `fire/mod.rs` — outside the one door
`gather_bucket`. The escape is `rune:lint(gather-walk-not-examining) — <reason>` with a **≥40-character**
reason; the gate's own header exempts join-index probes in `fire/mod.rs` **by rune, not by silence**.

**Measured on this tree, before release:**

| file | `bucket.iter()` | `for … in bucket` | `gather_bucket` |
|---|---|---|---|
| `fire/pass/accumulate.rs` | 1 | 1 | **0** |
| `fire/mod.rs` | 1 | 2 | **0** |

**The one door does not exist in this tree yet** — grok introduces it. So on arrival the gate has real
sites to find. **This is the #184 precedent: a gate that lands at step N is repaired AT N**, not folded
backward. For each site, decide from the code which it is:

- an Acc/Neg/Exists **examination** → route it through `gather_bucket` (the step's own conversion shows
  the shape), so `GATHER_VISITS` counts it; or
- a **join-index probe**, which is not an examination → give it the rune with a reason that says *why it
  is not an examination*, in ≥40 characters.

⛔ **Do not widen `SUBJECTS`, do not weaken the pattern, and do not rune a site you did not read.** If our
count differs from the table above when you get there, that is a result — report it.

## ⚠ WALL-CLOCK LIVES IN FOUR OF THESE FILES

`gather_probe_cost.rs` (24 `Instant`/`elapsed` sites), `accum_cost.rs` (23), `census.rs` (4). #406, #416,
#418 and #420 all touch this neighbourhood. **The 4i precedent:** a directional timing assert went red
1-of-6 under `kind(lib)` while passing alone, and was STRUCK 4-YES as a separate commit. ⛔ If a timing
assert reds: do NOT re-run, name the exact assertion and its numbers, and STOP and report. "It passes
alone" is not a disposition.

## ⚠ EXPECT CONFLICTS — MAIN'S RETE HAS MOVED

Measured against grok's own pre-image: **#416 5-of-5, #420 4-of-4, #406 4-of-5, #410 3-of-5, #418 2-of-3**
touched `.rs` already differ here. Cherry-picks will not apply clean. Compose them, and when you ask "did
their change land unchanged", **compare the DELTAS, not the blobs** (finding 36). At 4k two grok cures
(D10/D11) turned out to be already covered by main's own work and were re-composed as narrow fallbacks —
if a cure here is redundant, **measure it and say so**; do not land a second mechanism silently.

## ⚠ #414 REMOVES A TEST

It deletes `keyed_gather_visits_per_instrumented_path` and adds three formula tests (net +2). A removal is
as load-bearing as an addition: confirm the replacement covers what the deleted one asserted, and say so.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28 — tripped again
  last batch, self-caught).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture the whole block verbatim; name
  the exact assertion; STOP and report.
- ⛔ **READ EVERY COUNT OFF THE DATA AS YOU WRITE THE SENTENCE** (findings 27/34/36/37).
- ⛔ **A CURE ANSWERS TO THE GATES OF ITS MEDIUM.** After any edit to a `.rs` string literal, re-run the
  lint subset at that step before moving on.
- ⛔ **WAT IN `.rs` STRING LITERALS** is finding 33's class and this replay's most persistent defect
  source — it hit three separate steps last batch. These are rete kernel tests; grep that side and SAY SO.
- ⛔ **The record gate is PATH-BASED**: `^src/` needs `census:` + `nested-program-gate:`; `.rs` needs
  `lint-subset`, `kind(lib)`, `doctest`. Each verdict line on ONE line; put `(vs #N's …)` AFTER
  `no STOP-8`.
- ⛔ **`census:` must be TRUE.** If a step's `src/` change alters `wat --check` for files it did not
  produce, that is a STOP-8 and a finding — #388's class, two batches running. Never engineer a phrase
  that satisfies the gate's substring while the census says otherwise.
- ⛔ **NEVER `git filter-branch`** (finding 35) — detach / re-commit / rebuild-descendants only, and prove
  `git merge-base --is-ancestor origin/replay/grok-rete HEAD` plus empty `refs/original/` IN THE SCORE.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** (E15) — every rune, every conversion, every
  re-composition.

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.** A gate that did not exist
at step N is not a reason to fold into N — **#410 is exactly that case: repair AT the step that lands it.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh 849dcc4f5 HEAD 401 420

exit 0, plus `git replace -l`, `git for-each-ref refs/original/`, `git merge-base --is-ancestor
origin/replay/grok-rete HEAD`, `git status --porcelain`, and **a subject check for all 20 steps** against
`git log -1 --format=%s <C>`. Then write `SCORE-7n-replay-batch-4n.md` (every row, never blank) and a
`REPLAY-LOG.md` section. Commit both. **Leave the tree CLEAN. Yield by ending your turn — signal nothing.**
