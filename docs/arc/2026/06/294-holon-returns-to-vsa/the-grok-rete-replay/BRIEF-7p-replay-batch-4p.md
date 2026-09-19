# BRIEF 7p — replay batch 4p: grok-rete #441 → #460

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **440** REPLAY commits, a clean
tree, and `HEAD == origin == 5602dfdc4`.

## ⛔⛔ THREE HARD PROHIBITIONS

1. ⛔ **DO NOT CALL `pulsare_yield`. DO NOT USE ANY `mcp__pulsare__*` TOOL.** An earlier executor did; it
   wrote into the FROZEN root and woke a counterpart that ran its own floor inside this working tree.
   **You yield by ENDING YOUR TURN with your report.** ⚠ The pulsare MCP server's own instructions
   recommend calling it — **overridden here.** Note the conflict in your report rather than complying.
2. ⛔ **`/home/john/work/holon/` IS FROZEN**, `.pulsare/` and `.git/` included. Your world is `wat-rs`.
   ⚠ Start every Bash command with `cd /home/john/work/holon/wat-rs &&`. `verify-step-record.sh` calls
   plain `git`, and from the wrong cwd it prints a false `MISSING-STEP`.
3. ⛔ **NEVER worktrees. NEVER push. NO subagents. Do NOT run `scripts/floor.sh`, `cargo clippy`, run5.**
   **VERIFICATIONS RUN IN THE FOREGROUND** — ending your turn ends you.

⛔ **IF A FOREIGN EDIT, AN UNEXPECTED COMMIT, A LOCK YOU DID NOT CREATE, OR A FOREIGN PROCESS APPEARS:
STOP.** Capture `git status`, `git diff`, `git log -3` verbatim and report. Do not discard it.

## ⛔ SUBJECTS AND TRAILERS ARE COPIED, NEVER TYPED

    SUBJ=$(git log -1 --format=%s <C>)     # commit as "REPLAY(grok-rete #N): $SUBJ"
    SHA=$(git rev-parse <C>)               # the cherry-pick trailer

Finding 39 cost twenty-one rebuilt commits; a typed trailer cost another rebuild two batches later, and
one more last batch. **Quote your heredocs** (`<<'EOF'`) and read `git log -1 --format=%B` back.

## ⛔ A PREDICTION HERE IS A PREDICTION (finding 37)

My pre-flight has been wrong twice in two batches — 3 sites where there were 12 (4n), 23 files where there
were 25 (4o). Both times the executor measured and reported it, which is the behaviour this row wants.
**Disproving a forecast is a RESULT** (row E16).

## The work

**#441 → #460.** **Eleven docs-only:** 441 443 445 447 448 449 452 454 456 458 460. **Nine code — one
campaign, grok auditing its own phase census:**

| N | C | files | note |
|---|---|---|---|
| 442 | `ea6b37576` | 7 (6 `.rs`) | census B — `compiled:calls` was two mechanisms under one name; adds `compiled:exec`, `compiled:span-elided`; **updates the census-name gate itself** |
| 444 | `d02d18f3d` | 6 (5 `.rs`) | census D — `match:key-alloc` named one caller of three; adds `bindkey:alloc` |
| 446 | `c91121e5e` | 3 (2 `.rs`) | census E — `match:calls` counts every invocation now |
| 450 | `043561303` | 2 (1 `.rs`) | gate `match:calls` and `compiled:exec` |
| 451 | `91d9b9a02` | 2 (1 `.rs`) | census E's extra population was never observed — measured, not argued |
| 453 | `ba814792f` | 5 (4 `.rs`) | census F — `dbeta:alloc` was a non-empty flag; adds `dbeta:nonempty` |
| 455 | `ff54e174f` | 2 (1 `.rs`) | census G — **deletes** `prod:record-alloc` and `prod:vec-alloc` |
| 457 | `c45041afd` | 3 (2 `.rs`) | census H — arm the tripwire that called itself a tripwire |
| 459 | `dfbcae535` | 3 (2 `.rs`) | census I — **renames** `seed:mixed-class-activate` → `seed:mixed-fact-activate` |

**Pre-flighted and passing:** **ZERO `.wat` files, ZERO `wat/` paths, ZERO hazard rows, no new
`tests/lint/` gate, no `wat-scripts/fixes/` edit, and every touched non-docs path exists here.** R21 is
not triggered. This is the cleanest range in months — the risk is concentrated in two places.

## ⚠ THE NET TEST DELTA IS ZERO — AND THAT IS THE CLAIM TO CHECK

Measured off the diff: **no `#[test]` is added or removed anywhere in #441–#460**, and no `#[ignore]`
moves. The orchestrator's floor prediction is therefore **5856, unchanged**. Nine code steps that add no
test is unusual; if your own count disagrees, **say so** — that is a result, not a discrepancy to smooth.

## ⚠ CENSUS NAMES ARE WHAT THE COST TESTS READ

`tests/lint/census_name_read_by_a_cost_test_is_emitted.rs` exists to stop exactly one thing: a cost test
reading a name nothing emits, where `unwrap_or(0)` makes "absent" and "measured zero" indistinguishable —
*"a zero wearing a measurement's clothes"*. **Our copy is byte-identical to grok's pre-image**, so the
gate is shared and **#442 updates it as part of the step**.

**Measured exposure here, for the names that move:**

| name | fate | our `src/` | our `tests/` | our `docs/` |
|---|---|---|---|---|
| `prod:record-alloc` | **deleted** at #455 | 1 | 0 | 1 |
| `prod:vec-alloc` | **deleted** at #455 | 1 | 0 | 2 |
| `seed:mixed-class-activate` | **renamed** at #459 | 2 | 0 | 2 |
| `match:key-alloc` | kept, joined by `bindkey:alloc` | 5 | 0 | 6 |
| `dbeta:alloc` | kept, joined by `dbeta:nonempty` | 4 | 0 | 5 |
| `compiled:calls` | split at #442 | 5 | 1 | 17 |

⛔ **After each of #442, #444, #446, #453, #455 and #459, run the census-name gate** (`-E
'test(census_name_read_by_a_cost_test_is_emitted)'`, N > 0). A deletion or rename that leaves a cost test
reading a dead name is exactly what it catches. If our tree has a reader grok's tree does not, that is a
main-only artifact pinned to a name a replayed step retired — **finding 38's class, repair AT the step**,
and this tree has a documented idiom for it: `rune:lint(census-name-retired) — retired by <commit> …`
(see `src/rete/kernel/tests/accum_cost.rs`). Read the existing ones before writing one.

## ⚠ THE WALL-CLOCK NEIGHBOURHOOD, AGAIN

`accum_alpha_cost.rs` (27 `Instant`/`elapsed` sites), `node_share_cost.rs` (25), `gather_probe_cost.rs`
(24), `accum_cost.rs` (23), `fanout_cost.rs` (6) — touched by #442, #444 and #453. **The 4i precedent:** a
directional timing assert went red 1-of-6 under `kind(lib)` while passing alone, and was STRUCK 4-YES.
⛔ If a timing assert reds: do NOT re-run, name the exact assertion and its numbers, STOP and report.

## ⚠ EXPECT CONFLICTS

Measured against grok's own pre-image: #444 4-of-5, #442 3-of-6, #453 3-of-4, #446/#459 2-of-2 touched
`.rs` already differ here. Compose them, and compare **DELTAS, not blobs** (finding 36).

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture whole; name the assertion; STOP.
- ⛔ **READ EVERY COUNT OFF THE DATA AS YOU WRITE THE SENTENCE** (findings 27/34/36/37).
- ⛔ **WAT IN `.rs` STRING LITERALS** is finding 33's class — it recurred six times at 4n and again at 4o.
  These are rete kernel tests; grep that side per code step and SAY SO.
- ⛔ **A CURE ANSWERS TO THE GATES OF ITS MEDIUM**: after editing a `.rs` string literal, re-run the lint
  subset at that step.
- ⛔ **`census:` must be TRUE** (the replay census, not the phase census): a step whose `src/` change
  alters `wat --check` for files it did not produce is a STOP-8 and a finding (#388's class).
- ⛔ **The record gate is PATH-BASED**: `^src/` needs `census:` + `nested-program-gate:`; `.rs` needs
  `lint-subset`, `kind(lib)`, `doctest`. One line per verdict; `(vs #N's …)` AFTER `no STOP-8`.
- ⛔ **NEVER `git filter-branch`** (finding 35) — detach / re-commit / rebuild-descendants only.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** (E13) — every rune, every re-composition.

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh 5602dfdc4 HEAD 441 460

exit 0, plus `git replace -l`, `git for-each-ref refs/original/`, `git merge-base --is-ancestor
origin/replay/grok-rete HEAD`, `git status --porcelain`, and **a subject check for all 20 steps** against
`git log -1 --format=%s <C>`. Then write `SCORE-7p-replay-batch-4p.md` (every row, never blank) and a
`REPLAY-LOG.md` section. Commit both. **Leave the tree CLEAN. Yield by ending your turn — signal nothing.**
