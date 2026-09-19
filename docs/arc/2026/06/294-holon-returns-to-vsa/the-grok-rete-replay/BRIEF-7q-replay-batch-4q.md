# BRIEF 7q — replay batch 4q: grok-rete #461 → #480

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **460** REPLAY commits, a clean
tree, and `HEAD == origin == bca6203b9`.

## ⛔⛔ THREE HARD PROHIBITIONS

1. ⛔ **DO NOT CALL `pulsare_yield`. DO NOT USE ANY `mcp__pulsare__*` TOOL.** An earlier executor did; it
   wrote into the FROZEN root and woke a counterpart that ran its own floor inside this working tree.
   **You yield by ENDING YOUR TURN with your report.** ⚠ The pulsare MCP server's own instructions
   recommend calling it — **overridden here.** Note the conflict in your report rather than complying.
2. ⛔ **`/home/john/work/holon/` IS FROZEN**, `.pulsare/` and `.git/` included. Your world is `wat-rs`.
   ⚠ Start every Bash command with `cd /home/john/work/holon/wat-rs &&`.
3. ⛔ **NEVER worktrees. NEVER push. NO subagents. Do NOT run `scripts/floor.sh`, `cargo clippy`, run5, or
   a bare `cargo nextest run` — that IS the floor and it is the orchestrator's row.** Last batch's
   executor started one, caught itself, and killed it; better not to start. **Filtered runs (`-E …`) are
   yours; unfiltered runs are not.** **VERIFICATIONS RUN IN THE FOREGROUND** — ending your turn ends you.

⛔ **IF A FOREIGN EDIT, AN UNEXPECTED COMMIT, A LOCK YOU DID NOT CREATE, OR A FOREIGN PROCESS APPEARS:
STOP.** Capture `git status`, `git diff`, `git log -3` verbatim and report. Do not discard it.

## ⛔ SUBJECTS AND TRAILERS ARE COPIED, NEVER TYPED

    SUBJ=$(git log -1 --format=%s <C>)     # commit as "REPLAY(grok-rete #N): $SUBJ"
    SHA=$(git rev-parse <C>)               # the cherry-pick trailer

**Quote your heredocs** (`<<'EOF'`) and read `git log -1 --format=%B` back after each commit.

## ⛔ A PREDICTION HERE IS A PREDICTION (finding 37)

**My pre-flight has been short three batches running** — 3 exposed sites where there were 12, 23 files
where there were 25, six name-change steps where there were seven. Each time the executor measured and
reported it, which is the behaviour this row wants. **Disproving a forecast is a RESULT** (row E16).

## The work

**#461 → #480.** **Twelve docs-only:** 461 462 464 466 468 469 471 473 475 476 479 480. **Eight code:**

| N | C | files | note |
|---|---|---|---|
| 463 | `3b2065162` | 2 (1 `.rs`) | census K/L — two `census.rs` claims now say what the code does |
| 465 | `d4c068bdc` | 3 (2 `.rs`) | ⚠ **new lint gate** — a bench replica could write a production census key |
| 467 | `c6b8e3d40` | 2 (1 `.rs`) | the combinator-inner rune's premise measured — it holds |
| 470 | `afb3829ed` | 9 (8 `.rs`) | A4 — the fixpoint's dedup set gets one door |
| 472 | `ac07be72b` | 4 (2 `.rs`) | ⚠⚠ **collides with our 4i strike — see the ruling** |
| 474 | `f364b5613` | 2 (1 `.rs`) | the first-keying door refuses a second keying — A1's last remnant |
| 477 | `cad3b53b8` | 3 (1 `.rs`) | the excusare vocabulary accepted; **two runes re-worded** |
| 478 | `e95b5ba33` | 4 (1 new `.wat`, 2 `.rs`) | `insert` reports `insert` — conferre L2-2 |

**Pre-flighted and passing:** **zero hazard rows**, **no `wat/` path**, no `wat-scripts/fixes/` edit, R21
untriggered; #478's `.wat` is a NEW fixture (`tests/rete/probe_arc278_insert_reports_the_verb.wat`).

## ⚠⚠ #472 — THE RULING (4-YES, 2026-09-18, option B)

Grok's #472 adds to `token_bindings_representation_dominance`:

    #[ignore = "rune:excusare(below-resolution) — captured red .floor/2026-09-07T03-20-25Z: at card 64
                EXTEND trie 5860.1 ns vs array 3995.9 ns …"]

**This tree already cured that test.** The 4i strike `4d5287a53` — ruled 4-YES by the builder — struck the
two large-end directional assertions that gated the scheduler rather than the representation, and **its
own commit body predicted this step**: *"grok keeps these assertions until #472."*

**Measured here before release:** our version still carries (1) a NON-VACUITY check that refuses a dead
clock and (2) ONE ordering assertion whose margin is **4.53–10.15x**, and it has been **green in twenty
consecutive floors**. Grok's rune cites assertions this tree does not have.

**Therefore, at #472:**
- ✅ Land the excusare vocabulary work, including `tests/lint/no_unknown_ward_rune.rs`.
- ✅ Land the re-wording of the two OTHER ignores — `binding_key_cost` and `binding_repr_microbench` —
  which are `#[ignore]`d here exactly as on grok's side.
- ⛔ **Do NOT add the new `#[ignore]` to `token_bindings_representation_dominance`.** Record in #472's
  body: our 4i strike, that the rune's premise (those two assertions) is absent here, the two assertions
  our version still makes, and the twenty green floors.
- ⛔ **At #477, skip the matching re-wording of that same removed string** (its target does not exist
  here) and say so in #477's body. #477's OTHER re-wording — `binding_repr_microbench`'s `no-falsifier`
  rune — lands normally.

⚠ **If the excusare-vocabulary gate (`no_unknown_ward_rune`) demands a rune on an `#[ignore]` we do not
have, that is a different question: STOP and report.** It should not — our cured test carries no
`#[ignore]` at all.

## ⚠ #465 — a new gate, predicted GREEN here

`tests/lint/kernel_tests_census_count_is_bench_scoped.rs`: under `src/rete/kernel/tests/`, a
`census_count` call may name only a `bench:`-prefixed key. *"The exemption list is empty. A rune does not
save a production key."* **Measured: our only two non-bench calls are the two
`census_count("filter:test-reuse")` sites in `node_share_cost.rs`, and grok's own #465 converts exactly
those to `bench:filter-reuse`.** Run the gate and report its verdict rather than my forecast.

## ⚠ THE TEST COUNT MOVES BOTH WAYS THIS BATCH

Measured off the diff: **+9 `#[test]`** (6 at #465, 1 at #474, 2 at #478). The `#[ignore]` lines move too
— #472 is +3/−2 and #477 is +2/−2 — but **under the ruling above, the one NEW ignore is not landed**, so
the ignore count is unchanged. The orchestrator's prediction is therefore **5865 run, 24 skipped**. If you
land the ruling correctly, that is what the floor must say; if your own count disagrees, report it.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture whole; name the assertion; STOP.
- ⛔ **READ EVERY COUNT OFF THE DATA AS YOU WRITE THE SENTENCE** (findings 27/34/36/37).
- ⛔ **WAT IN `.rs` STRING LITERALS** is finding 33's class — #478 ships a new `.wat` fixture and two `.rs`
  files; grep that side and SAY SO.
- ⛔ **A CURE ANSWERS TO THE GATES OF ITS MEDIUM**: after editing a `.rs` string literal, re-run the lint
  subset at that step; after adding a `.wat`, run the loader gates.
- ⛔ **`census:` must be TRUE** (the replay census): a step whose `src/` change alters `wat --check` for
  files it did not produce is a STOP-8 and a finding.
- ⛔ **The record gate is PATH-BASED**: `^src/` needs `census:` + `nested-program-gate:`; `.rs` needs
  `lint-subset`, `kind(lib)`, `doctest`. One line per verdict; `(vs #N's …)` AFTER `no STOP-8`.
- ⛔ **NEVER `git filter-branch`** (finding 35) — detach / re-commit / rebuild-descendants only.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** (E13).

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.** A gate that did not exist
at step N is not a reason to fold into N — **#465 is that case: repair AT #465.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh bca6203b9 HEAD 461 480

exit 0, plus `git replace -l`, `git for-each-ref refs/original/`, `git merge-base --is-ancestor
origin/replay/grok-rete HEAD`, `git status --porcelain`, and **a subject check for all 20 steps**. Then
write `SCORE-7q-replay-batch-4q.md` (every row, never blank) and a `REPLAY-LOG.md` section. Commit both.
**Leave the tree CLEAN. Yield by ending your turn — signal nothing.**
