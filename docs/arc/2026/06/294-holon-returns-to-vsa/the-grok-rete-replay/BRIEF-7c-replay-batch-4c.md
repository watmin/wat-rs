# BRIEF 7c — replay batch 4c: grok-rete #212 → #220

> Released after batch 4b (#160–#211) is closed, floored and pushed. Ends BEFORE #221 — see § Why it
> stops at #220.

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd` and
`git rev-parse --abbrev-ref HEAD`. **Never use worktrees. Do not touch `~/work/holon/` or `main`. Never
push. Do not spawn subagents. Do not run `scripts/floor.sh`, clippy or run5** — the orchestrator weighs
those centrally and uncontended; a gate run while you work in the tree produces a FALSE result.

⚠ `wat-rs/CLAUDE.md` does not reach an executor, so the load-bearing doctrine is carried here:
- **R21** — `.wat` corpus rewrites go through a recorded wat-fix codemod (`wat-scripts/fixes/*.wat`,
  framework `wat/fix.wat`), NEVER hand edits, never sed/python. The ONE exception is **wat embedded in
  `.rs`/`.sh` string literals**, which no codemod reaches — hand-fix those and LOG each edit.
- **THERE IS NO KNOWN FLAKE. A RED IS A RED.** "timing", "environmental", "pre-existing", "unrelated to my
  change", "passes in isolation" are not dispositions. On any red: do NOT re-run, copy the whole
  stdout+stderr block verbatim, name the exact assertion, STOP.
- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything. Redirect its full output to a
  file and read the file.** This is the rule that makes the one above ACTIONABLE — "capture the red
  verbatim" is impossible if the run was already truncated, because by the time you know it is red the
  panicked block is gone. It cost a real diagnosis at #190 (finding 28): a `tail -10` discarded the
  failure's own evidence table, which the assert message had been deliberately built to carry so that
  "a red arrives with its own evidence". Context pressure is exactly when the shortcut is tempting, so
  make the redirect the habit rather than relying on vigilance:

      cargo nextest run --release -E '<filter>' > /tmp/gate.out 2>&1; echo "EXIT=$?"
      grep -aE "Summary|FAIL|panicked" /tmp/gate.out     # then READ the file for the block

  **Rung, stated honestly: this is a CONVENTION** (`[[feedback_a_rule_written_in_prose_that_nothing_ever_runs]]`).
  Nothing executes it — no gate can see how an agent invoked a command — so it is carried in every
  executor prompt and here, and it is worth re-reading rather than assuming it held.
- **An isolated re-run is the WEAKEST evidence against a failure seen under load**, and on a timing test it
  is evidence about a different question entirely: one test alone in a fresh process is not the measurement
  that 13-of-87 under parallel load performed. Never let an isolated green dispose of a loaded red.
- **Ending your turn ENDS you.** Run every verification in the FOREGROUND and read its output.
- Scratch `.wat` lives in `wat-scripts/scratch-pad/`, never a temp dir.
- **After ANY commit or `--amend`: assert `git status --porcelain` is EMPTY and that the commit's own diff
  names every path its body claims** (finding 27 — three occurrences across two agents). **Read every COUNT
  off `git show --stat`/the diff before writing it into a body.**

## The work

Replay grok-rete **#212 → #220** (9 steps) by `BRIEF-1-pilot-first-ten-commits.md` § "One step", every row.
Each commit: `REPLAY(grok-rete #N): <C's subject>` — **docs-only steps included** — with the `-x` trailer
`(cherry picked from commit <sha>)` kept in the body, and the five verdict lines verbatim where required.

Census (`bootstrap/era/replay-plan/commits.tsv`):

| N | C | kind | files | wat | rs | note |
|---|---|---|---|---|---|---|
| 212 | `9ee04f945` | shared | 7 | 5 | 1 | **the docs-wat gate — see below** |
| 213 | `78c0435ab` | docs | 1 | 0 | 0 | |
| 214 | `e6858e858` | docs | 3 | 0 | 0 | |
| 215 | `119214aef` | code | 11 | 0 | 10 | largest step; adds `tests/lint/minimum_label_matches_its_estimator.rs` |
| 216 | `c75b0152c` | docs | 1 | 0 | 0 | |
| 217 | `4914b0d18` | docs | 6 | 0 | 0 | |
| 218 | `2733b9bd9` | code | 5 | 3 | 2 | 3 new `tests/rete/probe_arc278_enum_variant_typo*.wat` |
| 219 | `69dcf2c06` | docs | 2 | 1 | 0 | **a docs step carrying a `.wat` — see below** |
| 220 | `93ea0c618` | docs | 6 | 0 | 0 | |

**Zero stdlib-touch rows and zero moved-home rows in this range** — no two-phase stdlib convert. If a
`wat/`, `wat-scripts/fixes/`, or `absent-on-main.tsv` path appears anyway, that is STOP-9/E8: stop and
report.

## ⚠ #212 — the docs-wat gate. Its disposition is ALREADY RULED; do not re-derive it

`#212` adds `tests/lint/docs_wat_loads_or_declares_why_not.rs`, which walks every `.wat` under `docs/arc/`
and requires each to LOAD on the current runtime **or** carry `;; rune:lint(red-by-design|historical) —
<reason>`. The SEAM's ruling (§ Batches) governs, and `docs/arc/2026/06/294-holon-returns-to-vsa/the-grok-rete-replay/STEP-NOTES-212-and-226.md` carries
the measurements that confirm it is executable:

1. **KEEP main's `.wat.bad` rename. DROP grok's two `historical` runes.** The arc-130 pair
   (`docs/arc/2026/05/130-…/complected-2026-05-02/{substrate,test}.wat`) exists HERE as `.wat.bad`, because
   main's `every_tracked_wat_parses` wall takes a RENAME as its escape hatch, deliberately "not an
   exemption list". grok's gate filters `p.extension() == "wat"`, so a `.wat.bad` file is invisible to it —
   both walls stay green and nothing is lost. Un-renaming them to apply grok's runes would put main's wall
   RED, by main's own recorded demonstration.
2. **Merge the README prose; do not import grok's rune-explaining section as written** — those runes are not
   landing, and our README already carries the `.wat.bad` story and the corrected "line 64" citation.
3. **grok's `probes/surface-field-dispatch.wat` `:holder` → `:nature` migration DOES apply** — our copy
   still reads `:holder` at line 11, so that ~8-week rot is live here too. One file, one keyword: NOT a
   corpus migration, so not an R21 codemod; it is grok's own content landing. It carries no rune by design.
4. `probes/red-owner-signals-child.wat`'s `red-by-design` rune and header corrections land normally; so does
   `harness-experiri/experiri-then-match.wat`'s rune (that directory arrives at #195, inside 4b).
5. **RE-TAKE the docs-`.wat` census at the 4b tip before you start** — `find docs -name '*.wat' | wc -l`.
   It was 4 before 4b and should be 8 after #195 lands `harness-experiri/`. The gate carries a NON-VACUITY
   guard ("no .wat found under docs/arc/ — the gate is measuring nothing"), so the number matters.
6. **DRIVE the gate. Do not trust any of the three documents about it — they disagree with each other.**
   #211 lands grok's own `strike-docs-graveyard/{DESIGN,EXPECTATIONS}.md` for this very gate, and the
   shipped #212 contradicts both in two places. The commit is the authority for what LANDS; OUR TREE is the
   authority for what the gate SAYS.

   **⛔ THE COUNT. grok's EXPECTATIONS row 2 requires the walk to find TEN files and says "fewer means the
   walk is missing a directory." On this tree the correct answer is EIGHT** — measured at the 4b tip: the
   4 `probes/` plus the 4 `harness-experiri/` that #195 landed. The 2 arc-130 files are `.wat.bad` here and
   are invisible to the gate by construction (it filters `extension == "wat"`). **8 is right; do not "fix"
   the walk, and do not read grok's 10 as a bar.**

   **⛔ `experiri-acc-head.wat` is classified THREE different ways.** The DESIGN's driven table says
   *"refuses — red by design (the A3 repro)"*; EXPECTATIONS row 1 lists it among the five expected reds;
   **the shipped #212 body says it gets NO RUNE, because its designed red fires at RULE COMPILATION, so the
   file LOADS and the gate is satisfied** — and adds that runing it *"would blind the gate to its future rot
   for nothing."* Follow the commit. Then verify against our own tree, because the same distinction
   (refuses-at-load vs refuses-later) may land differently here.

   **⛔ RUNE PLACEMENT flipped during implementation too.** EXPECTATIONS row 7 expects *"header comment
   only"*; the shipped commit appends the `historical` runes at the **FOOT**, because the arc-130 README
   cites *"the original line 64 of substrate.wat"* and a header insertion would silently falsify that
   citation. Moot here (those runes are not landing per the ruling) but it is the tell that the planning
   docs are stale relative to the commit.

   **THE CONTRACT DECISION, which is the one thing that must not be got wrong** — grok's own words:
   *"The marker states WHY the file must fail, and a ROTTED file may not wear it."* And its stated failure
   condition: *"What would make this a failure even if every test passes: marking `surface-field-dispatch.wat`
   instead of migrating it. That is rot wearing a declaration, which rebuilds the graveyard INSIDE the gate
   and leaves it looking enforced."* So: **migrate rot; rune only when the failure IS the artifact; a reason
   that says "it fails" is not a reason.**

   **Expected dispositions on our 8** (grok's driven verdicts, to be re-driven here, NOT assumed):
   alive — `probes/enum-holds-record.wat`, `probes/red-send-cause-is-not-matchable.wat`,
   `harness-experiri/experiri-acc-wrapped.wat`, `harness-experiri/experiri-when-match.wat`;
   MIGRATE — `probes/surface-field-dispatch.wat` (`:holder` → `:nature`; **the bar is that it PRINTS 142**,
   its own header's promise — grok's row 5 says explicitly *"Not 'it loads'"*, so run the binary and read
   stdout); rune `red-by-design` — `probes/red-owner-signals-child.wat`,
   `harness-experiri/experiri-then-match.wat`; and `experiri-acc-head.wat` per the divergence above.

   **Mutation-prove the gate, both arms** (grok did, and its note is load-bearing): strip the rune from one
   `red-by-design` file → **that file alone** reddens, restore; revert `:nature` → `:holder` →
   **`surface-field-dispatch` alone** reddens, restore. ⚠ *"If a mutation reddens nothing, that is a finding
   about coverage, not a null result."*

   **If the walk finds a `.wat` not in that list of 8, that is a FINDING to surface — never a reason to
   narrow the walk** (grok's own affirmative scope cut).

## ⚠ #219 — a docs step whose `.wat` must satisfy the gate #212 just installed

`#219` adds `docs/arc/…/probes/red-acc-refire-native-vs-oracle.wat`. Once #212's gate is in the tree, that
file must LOAD on the current runtime or carry a closed rune. Run the gate at #219 and record the verdict.
The census classifies by DIRECTORY, not extension — a `docs` row can still carry `.wat`, and it then needs
`--check` plus the `census: … --diff no STOP-8` and `nested-program-gate: PASS` verdict lines like any
other `.wat`-touching step.

## #215 — a new `.rs` TEST FILE meets main's test-hygiene walls (finding 24)

`#215` adds `tests/lint/minimum_label_matches_its_estimator.rs` (446 lines). A step that ADDS OR CHANGES a
`.rs` test file must run the test-hygiene walls — `no_inlined_edn`, `no_loose_string_assert`,
`no_inlined_wat` — because they are triggered by the FILE'S EXISTENCE, not by the step's topic. Finding 24
is exactly this: a stone's targeted gate set passed while the floor went red on two walls its subject never
suggested.

## Why it stops at #220

**#221 (`16f504e14`) is the first MULTI-stdlib-file step since stone 2a4d** — it touches
`wat/rete/oracle/fire.wat` AND `wat/rete/oracle/stratify.wat`, i.e. the first real exercise of the per-SET
stdlib world 2a4d was built for. The SEAM's batch rule requires a boundary only where a NEW POLICY is
needed, and #221 needs none — the machinery exists. This boundary is the orchestrator's scheduling call, so
that if the per-SET world misbehaves the failure is isolated rather than buried mid-batch.
⚠ **This is a recommendation, not a ruling — the builder may fold #221 into this batch instead.**
(#226 is the next one to watch after that: one stdlib file, but it changes the stdlib MACRO
`:wat::rete::defquery` — finding 19's G1 class, closed by 2a4c.)

## ⛔ STOP triggers — rejections, not permission to defer. Report the verbatim evidence

STOP-1 … STOP-13 as BRIEF-1 and BRIEF-7a state them, and in particular:
- **STOP-8** — a file going rc 0 → non-zero that the step did not produce (`census.sh --diff`).
- **STOP-9** — `convert.sh` reporting `UNREGISTERABLE` for a `wat/…` path.
- **STOP-10** — stone 3's nested-program gate red. **STOP-11** — `kind(lib)` or doctests red.

## The fold rule

A composition defect belonging to an earlier step FOLDS INTO that step; rebuild the steps after it. **Never
a repair commit after the batch, and never a knowingly-red REPLAY commit** — this branch becomes main. A
fold's proof must name WHICH paths it is expected to carry, checked with `git diff <old-tip> <new-tip>`.
A gate that does not yet exist at step N is not a reason to fold into N (precedent: #184).

## Tier

Commit each step on green. **Do not push.** Before yielding, run in the FOREGROUND:

    scripts/replay/verify-step-record.sh <batch-start> HEAD 212 220

It must exit 0 — the range form asserts exactly one `REPLAY(grok-rete #N)` per N and cross-checks each
commit's cited source against `commits.tsv`, so a mis-subjected or skipped step goes RED BY ABSENCE.
Yield after #220, or at the first STOP, with `SCORE-7c-replay-batch-4c.md` and a `REPLAY-LOG.md` section.
