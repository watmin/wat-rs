# BRIEF 7r — replay batch 4r: grok-rete #481 → #500

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **480** REPLAY commits, a clean
tree, and `HEAD == origin == 0038bbf83`.

## ⛔⛔ HARD PROHIBITIONS

1. ⛔ **DO NOT CALL `pulsare_yield`. DO NOT USE ANY `mcp__pulsare__*` TOOL.** The pulsare MCP server's own
   instructions recommend it — **overridden here.** Note the conflict in your report rather than
   complying. **You yield by ENDING YOUR TURN with your report.**
2. ⛔ **`/home/john/work/holon/` IS FROZEN.** Your world is `wat-rs`. **Start every Bash command with
   `cd /home/john/work/holon/wat-rs &&`.**
3. ⛔ **No worktrees. No push. No subagents. No `git filter-branch`. No `scripts/floor.sh`, no
   `cargo clippy`, and NO unfiltered `cargo nextest run`** — filtered `-E '…'` runs are yours; the whole
   floor is the orchestrator's row. ⚠ **`cargo bench` is also NOT yours** — #498 adds one; do not run it.
4. ⛔ **VERIFICATIONS RUN IN THE FOREGROUND.** Ending your turn ends you.

⛔ **IF A FOREIGN EDIT, AN UNEXPECTED COMMIT, A LOCK YOU DID NOT CREATE, OR A FOREIGN PROCESS APPEARS:
STOP.** Capture `git status`, `git diff`, `git log -3` verbatim and report. Do not discard it.

## ⛔ SUBJECTS AND TRAILERS ARE COPIED, NEVER TYPED

    SUBJ=$(git log -1 --format=%s <C>)     # commit as "REPLAY(grok-rete #N): $SUBJ"
    SHA=$(git rev-parse <C>)               # the cherry-pick trailer

Quote your heredocs (`<<'EOF'`) and read `git log -1 --format=%B` back. ⛔ **And keep every `census:` /
`lint-subset:` / `kind(lib):` verdict on ONE physical line** — a wrapped line is invisible to the gate's
regex and cost last batch a rebuild.

## ⛔ A PREDICTION HERE IS A PREDICTION (finding 37)

**My pre-flight has been short four batches running**, most recently a floor forecast the executor
corrected by measuring a `#[cfg(all(test, debug_assertions))]` module out of the release build. **I
adopted its number over mine.** Measure everything below; disproving a forecast is a RESULT (row E16).

## The work

**#481 → #500.** **Eight docs-only:** 483 485 489 491 493 495 497 499. **Twelve code:**

| N | C | note |
|---|---|---|
| 481 | `c07af0af4` | strike: conferre L2-3 — the two stratifiers disagree on numbers |
| 482 | `aa10ef8bd` | the disagreement, tested (+1 test) |
| 484 | `05d33d022` | grid: accum-over-derived, checked three ways |
| 486 | `66a24d288` | ⚠ `wat/` (1 file) — stratify headers stop claiming lockstep, **comments only** |
| 487 | `e5aa21f4a` | strike: `produced_type` — the oracle names the FUNCTION, native names the fact |
| 488 | `34ee46ce9` | the oracle DROPS a derived fact when the `:then` head is a user fn (+2 tests) |
| 490 | `21a5f8514` | ⚠ `wat/` (1 file) — `rule-produces` RESOLVES the head; the dropped fact is back |
| 492 | `caeef4793` | grid: userfn-head — the standing three-way |
| 494 | `7b51ac717` | grid: accum-lead-rule-cascade — the matrix's fourth cell |
| 496 | `27ea7276d` | ⚠ **new lint gate** — EMITTED ⇒ READ for census counters (+5 tests) |
| 498 | `bb306bd3c` | ⚠⚠ **Stone K — benches move; see the ruling** (−3 tests, −3 ignores) |
| 500 | `ce6c1e35c` | strike the probe's false purity claim |

**Pre-flighted and passing:** zero hazard rows; no `wat-scripts/fixes/` edit; both M-status-absent paths
created earlier in the range (#485→#486, #488→#490); #486's and #490's `wat/` edits are one file each.

## ⚠⚠ #498 — THE RULING (4-YES, 2026-09-18, option B)

Grok's Stone K moves three diagnostics out of the test binary into a new `benches/binding_repr.rs`
(`[[bench]]`, `harness = false`) and **deletes `token_bindings_representation_dominance` outright**.

**Measured before release:** grok's bench keeps a faithfulness `assert_eq!(rows.len(), cards.len())` and
the dead-clock `assert!(extend_array_wins + get_array_wins > 0)`, and says *"Timing-ordering assertions
are gone — they cannot separate the hypotheses under this floor's contention band."* **That is true of
the two LARGE-END assertions grok still carried. It is NOT true here:** this tree struck those two at the
4i strike (`4d5287a53`, 4-YES) and kept the **SMALL-END GET ordering assertion**, margin **4.53–10.15x**,
which has been green in every floor since — and at #472, twenty steps ago, we refused to silence it.

⛔ **`harness = false` means a bench NEVER runs on our floor.** Landing #498 verbatim would delete a live,
wide-margin gate and silently reverse a ruling made twenty steps earlier.

**Therefore, at #498:**
- ✅ **Land the bench move in full** — `benches/binding_repr.rs`, the `Cargo.toml` `[[bench]]` entry,
  `src/rete/matcher.rs`, the docs. Benchmarks do not belong in the test binary; grok is right about that.
- ✅ Let `binding_key_cost` and `binding_repr_microbench` leave the test binary as grok does — both are
  `#[ignore]`d here, prove nothing on the floor, and their excusare runes travel with them.
- ⛔ **KEEP a slim floor test carrying the surviving assertions**: the small-end GET ordering assertion
  AND its non-vacuity companion. Give it the smallest driver that feeds them honestly — reuse the
  existing helpers rather than re-deriving them — and record above it: the 4i strike, the #472 ruling,
  the measured margin, and that the diagnostics themselves now live in `benches/`.
- ⛔ **Do NOT run `cargo bench`.** Compilation of the bench target is checked by the orchestrator's
  clippy `--all-targets` run.
- If keeping that assertion proves impossible without dragging the whole harness back into the test
  binary, **STOP and report** with what you measured — do not improvise a third shape.

## ⚠ #496 — THE MIRROR GATE, AND ITS NUMBER IS GROK'S

`census_emitted_name_is_read_or_declared`: **EMITTED ⇒ READ**, counters only (`census_count` /
`census_count_n`), deliberately not covering `phase_end`. It is the mirror of 4p's READ ⇒ EMITTED gate,
and it reuses that sibling's own two sets rather than re-scanning.

Grok's own body says *"all 25 got readers, none runed"*. ⛔ **That is grok's 25.** This tree has diverged
— we ran the whole census campaign at 4p with our own counts. **Measure our emitted set, report the
number, and if a counter here has no reader, that is a FINDING**: say which, and dispose of it the way
the gate itself provides (a declaration, if it has one) — never by deleting a live counter to get green.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture whole; name the assertion; STOP.
- ⛔ **READ EVERY COUNT OFF THE DATA AS YOU WRITE THE SENTENCE** (findings 27/34/36/37) — **and exclude
  comments when you count code.** My own grep counted 7 `#[ignore]` sites in a file with 2.
- ⛔ **WAT IN `.rs` STRING LITERALS** is finding 33's class. Eight of these steps carry a `.wat`; several
  are new fixtures from grok's era, which predates syntax migrations here. **If a new `.wat` needs
  curing, use `scripts/replay/convert.sh <introducing-commit> …`, the batch-4o/4q precedent — never
  hand-edit.**
- ⛔ **A CURE ANSWERS TO THE GATES OF ITS MEDIUM**: after a `.rs` string-literal edit, re-run the lint
  subset at that step; after adding a `.wat`, run the loader gates.
- ⛔ **`census:` must be TRUE** (the replay census): a step whose `src/` change alters `wat --check` for
  files it did not produce is a STOP-8 and a finding.
- ⛔ **The record gate is PATH-BASED**: `^src/` needs `census:` + `nested-program-gate:`; `.rs` needs
  `lint-subset`, `kind(lib)`, `doctest`. One line per verdict; `(vs #N's …)` AFTER `no STOP-8`.
- ⛔ **NEVER `git filter-branch`** — detach / re-commit / rebuild-descendants only.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** (E13).

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.** A gate that did not exist
at step N is not a reason to fold into N — **#496 is that case: repair AT #496.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh 0038bbf83 HEAD 481 500

exit 0, plus `git replace -l`, `git for-each-ref refs/original/`, `git merge-base --is-ancestor
origin/replay/grok-rete HEAD`, `git status --porcelain`, and **a subject check for all 20 steps**. Then
write `SCORE-7r-replay-batch-4r.md` (every row, never blank) and a `REPLAY-LOG.md` section. Commit both.
**Leave the tree CLEAN. Yield by ending your turn — signal nothing.**
