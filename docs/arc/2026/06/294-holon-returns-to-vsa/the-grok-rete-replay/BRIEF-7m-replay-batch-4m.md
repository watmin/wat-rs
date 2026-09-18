# BRIEF 7m — replay batch 4m: grok-rete #381 → #400

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **380** REPLAY commits, a clean
tree, and `HEAD == origin == f00eed601`.

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

⛔ **IF A FOREIGN EDIT, AN UNEXPECTED COMMIT, A LOCK YOU DID NOT CREATE, OR A FOREIGN PROCESS APPEARS IN
THE TREE: STOP.** Do not discard it. Capture `git status`, `git diff` and `git log -3` verbatim and report.

## ⛔ THE SUBJECT IS GROK'S, VERBATIM — finding 39, last batch

Nine of 4l's twenty steps landed with a paraphrased subject, and one re-classified grok's `fix(tests):`
as `perf:`. **The record gate cannot catch this**: it reads the `REPLAY(grok-rete #N)` prefix and the
trailer, not the words. The orchestrator caught it after the yield and rebuilt twenty-one commits.

**Before you commit each step**, take the subject from the data:

    SUBJ=$(git log -1 --format=%s <C>)     # never retyped, never paraphrased, never re-classified

and commit as `REPLAY(grok-rete #N): $SUBJ`. Same for the trailer SHA — copy it from `git rev-parse`.
Both defects last batch came from *re-authoring* a message instead of *re-copying* it.

## ⛔ A PREDICTION HERE IS A PREDICTION (finding 37)

Every forecast below is a hypothesis with a measurement attached. **Disproving one is a RESULT** — row
E20 scores honest disagreement, never agreement. At 4k every corpus figure in the brief was wrong and the
executor was right to measure; at 4l the brief undercounted #362's files and that, too, was a result.

## The work

**#381 → #400.** **Twelve docs-only:** 382 385 386 389 391 392 393 394 395 396 397 399. **Eight code:**

| N | C | files | note |
|---|---|---|---|
| 381 | `974e0d859` | 47 (3 `.rs`, 40 docs) | F2 closed — a docs sweep with a small `src/rete` tail |
| 383 | `f4a271cb3` | 10 (6 `.rs`) | ⚠ **D2 is LIVE — a FINDING, not a fix.** Banks a red acceptance test `#[ignore]`, assertion INTACT |
| 384 | `21530efab` | 14 (11 `.rs`) | **D2's cure** — un-ignores that test; `right_index_counter_invariant.rs` is created at #383 |
| 387 | `f785e2430` | 8 (3 `.wat`) | ⚠⚠ **the mode-parity gate — RED BY DESIGN until #388. See the ruling below** |
| 388 | `8bca0f7fe` | 1 (`src/distribution/mod.rs`) | **#387's cure** — hoists `RLIMIT_STACK` above every mode return; adds the `:user::main` signature check to `--check` |
| 390 | `0ee56325f` | 11 (9 `.rs`) | A1 — a `:where` query returned 1 row where the spec says 2 |
| 398 | `c7b4ce30d` | 7 (4 `.wat`, 3 under `wat/`) | ⚠ **new lint gate** `no_raw_network_keys_in_oracle.rs` |
| 400 | `7f4bb3699` | 6 (5 `.rs`) | D3 — the door's structural claim is now true |

**Pre-flighted and passing:** **zero hazard rows**; both M-status-absent paths are created earlier in the
range (`right_index_counter_invariant.rs` #383→#384; `strike-explain-order/SCORE.md` #396→#398); #398 is
the batch's only `wat/` step and its three files are `wat/rete/oracle/{explain,fire,pass}.wat`.

⛔ **#398's NEW GATE LANDS GREEN HERE, MEASURED.** `no_raw_network_keys_in_oracle` bans a raw
`PersistentMap/keys network` walk in code under `wat/rete/oracle/**`. **Our tree carries 6 oracle files
(grok touches 3) and ZERO of them trips the ban** — `fire.wat:152` mentions `PersistentMap/keys` only
inside a `;;` comment, and not in the banned phrase. This is a prediction: run the gate and report what
it actually says. Three of the last five new grok gates landed red here.

## ⚠⚠ #387 AND #388 — A DELIBERATE RED-THEN-CURE PAIR. RULED 4-YES, OPTION B

#387 lands `tests/cli/mode_parity.rs`, a gate grok *intends* to fail until #388 cures it. Our rule forbids
a knowingly-red REPLAY commit. **Both arms are LIVE on this tree — the orchestrator measured them with
grok's own fixtures before release:**

- **SOUNDNESS**: `mode_parity__empty.wat` → `--check` rc **0** (Accepted), run rc **4** (Rejected). Our
  `--check` accepts a program with no `:user::main` that the run path refuses.
- **LIVENESS**: the deep fixture → run rc **0**, `--check` **SIGABRT (134)**. Our `RLIMIT_STACK` raise is
  at `src/distribution/mod.rs:~395`, BELOW the `--check` short-circuit at `~346` — precisely what #388
  cures.

⛔ **GROK'S DEEP FIXTURE IS STALE HERE AND WOULD PASS FOR THE WRONG REASON.** Its generator
`tests/cli/gen_mode_parity_deep.sh:13` emits the retired `:wat::core::i64::+`. Unfixed, both modes reject
with 1000 type errors and LIVENESS passes vacuously. **Re-spell the generator AND the generated fixture to
`:wat::i64::+` at #387**, and say so in the body. (Measured: with the spelling fixed, run rc 0 and
`--check` SIGABRTs — the real defect appears.)

**THE RULING (4-YES, 2026-09-17):** land #387 with its two failing arms **`#[ignore]`d, assertions
INTACT**, each carrying a rune that names #388 as the cure and the measured rc values above; record the
reds in #387's body. **#388 removes the ignores** as part of the cure — that restoration is what proves
it. Rejected: folding #388 into #387 (makes #388 an empty `fix(cli)`), committing #387 red, reordering.

⛔ **This is the tree's OWN idiom, not an invention** — and grok reaches for it two steps earlier: #383
banks its D2 acceptance test `#[ignore]` with the assertion intact and #384 un-ignores it, citing
`probe_undefined_builtin_resolves.rs:17` and `probe_arc255_reflection_parity.rs:70`. **Read #383's
handling first and match its wording.** ⛔ Do NOT weaken an assertion to get green — banking keeps the
assertion whole; weakening is what the idiom exists to prevent.

## ⚠ #383/#384 — a finding step, then its cure

#383 is a FINDING: D2 is live, and its acceptance test lands banked. Do not pull #384's fix backward.
#384 un-banks it; if the test does not go green there, that is a real result — STOP and report rather
than adjusting the assertion.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture the whole block verbatim; name
  the exact assertion; STOP and report.
- ⛔ **Ending your turn ENDS you.** Verifications in the FOREGROUND.
- ⛔ **READ EVERY COUNT OFF THE DATA AS YOU WRITE THE SENTENCE** (findings 27/34/36/37).
- ⛔ **DIFF LIKE AGAINST LIKE** (finding 36): step blob vs step blob, never against `HEAD`.
- ⛔ **A MAIN-ONLY ARTIFACT PINNED TO TEXT A REPLAYED STEP REWRITES IS ITS OWN CLASS** (finding 38, four
  instances). #387's stale fixture is the same family.
- ⛔ **A CURE ANSWERS TO THE GATES OF ITS MEDIUM.** After any edit to a `.rs` string literal, re-run the
  lint subset at that step before moving on (4k's fold cost two rebuilds for exactly this).
- ⛔ **WAT IN `.rs`/`.sh` STRING LITERALS** is finding 33's class and this replay's most persistent defect
  source. `#387` ships a `.sh` generator that WRITES wat — check its spelling against this tree.
- ⛔ **The record gate is PATH-BASED**: `^src/` needs `census:` + `nested-program-gate:`; `.rs` needs
  `lint-subset`, `kind(lib)`, `doctest`. Each verdict line on ONE line; put `(vs #N's …)` AFTER
  `no STOP-8`.
- ⛔ **NEVER `git filter-branch`** (finding 35) — detach / re-commit / rebuild-descendants only, and prove
  `git merge-base --is-ancestor origin/replay/grok-rete HEAD` plus empty `refs/original/` IN THE SCORE.
- ⛔ **NEVER FABRICATE A SHA.** Copy every trailer from live git output (finding 39's sibling defect).
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** (E17).
- Every step commits as `REPLAY(grok-rete #N): <C's subject, copied>` with the
  `(cherry picked from commit <sha>)` trailer.

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.** A gate that did not exist
at step N is not a reason to fold into N (the #184 precedent) — **#398 is exactly that case: repair AT
the step that lands it.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh f00eed601 HEAD 381 400

exit 0, plus `git replace -l`, `git for-each-ref refs/original/`, `git merge-base --is-ancestor
origin/replay/grok-rete HEAD`, `git status --porcelain`, and **a subject check for all 20 steps against
`git log -1 --format=%s <C>`**. Then write `SCORE-7m-replay-batch-4m.md` (every row, never blank) and a
`REPLAY-LOG.md` section. Commit both. **Leave the tree CLEAN. Yield by ending your turn — signal nothing.**
