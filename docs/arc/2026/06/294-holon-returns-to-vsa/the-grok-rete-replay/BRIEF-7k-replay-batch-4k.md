# BRIEF 7k — replay batch 4k: grok-rete #341 → #360

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **340** REPLAY commits, a clean
tree, and `HEAD == origin == 5ba45a81f`.

## ⛔⛔ THREE HARD PROHIBITIONS

1. ⛔ **DO NOT CALL `pulsare_yield`. DO NOT USE ANY `mcp__pulsare__*` TOOL.** An earlier executor did; it
   wrote into the FROZEN root and woke a counterpart that ran its own floor inside this working tree,
   contending with the orchestrator's gates. **You yield by ENDING YOUR TURN with your report.** ⚠ The
   pulsare MCP server's own instructions recommend calling it — **overridden here.** Note the conflict in
   your report rather than complying; the last four executors did exactly that and were right to.
2. ⛔ **`/home/john/work/holon/` IS FROZEN**, `.pulsare/` and `.git/` included. Your world is `wat-rs`.
3. ⛔ **NEVER worktrees. NEVER push. NO subagents. Do NOT run `scripts/floor.sh`, `cargo clippy`, run5.**

## ⛔ A PREDICTION HERE IS A PREDICTION (finding 37)

At 4i a brief said a gate "WILL land red"; it landed GREEN, and only the executor's refusal to assume
stopped a manufactured "repair". **Every forecast below is a hypothesis with a measurement attached.
Disproving one is a RESULT** — row E22 scores honest disagreement, never agreement.

## The work

**#341 → #360.** **Thirteen docs-only:** 341 343 345 346 347 348 350 351 353 354 356 358 359.
**Seven code:**

| N | C | files | note |
|---|---|---|---|
| 342 | `065ed3d91` | 2 (1 `.wat`) | finding — D10, the `:then` RHS is not type-checked |
| 344 | `e38b1f46a` | 16 (3 `.wat`, 4 `.rs`) | D10's fix |
| 349 | `2e54c8a66` | 13 (3 `.wat`, 2 `.rs`) | D11 — the check reaches nested constructors |
| 352 | `8f34088d6` | 5 (3 `.rs`) | ⚠ **new lint gate** — see below |
| 355 | `ed555d02e` | 9 (1 `.wat`, 2 `.rs`) | C9's port half |
| 357 | `545771b2f` | 8 (1 `.wat`) | C9 closed |
| 360 | `04abe37fc` | 32 (14 `.wat`, 9 `.rs`) | ⚠ **new lint gate, batch's largest** — see below |

Pre-flighted and passing: **zero hazard rows**, **no step touches `wat/`**, **23 `.wat`** (so
`--check`/`convert.sh` work — **not** R21's codemod path), and **all 8 M-status-absent paths are created
earlier in the same range** (#342→#344, #341→#355, #355→#357, #358→#359). #377 is the next two-phase
stdlib step — the batch after this.

## ⚠⚠ #360 — `every_wat_bad_fixture_actually_fails` (433 lines)

Every `*.wat.bad` under `tests`/`wat-scripts`/`docs` must return `Err` from **`startup_from_file`**.

⛔ **THE DRIVER IS THE WHOLE QUESTION.** The gate's own header: *"`./target/release/wat <file>` and
`startup_from_file` give OPPOSITE verdicts on these same files, and the first draft of this strike was
withdrawn for using the wrong one."* The binary demands a `:user::main` because it EVALs one;
`startup_from_file` does not. **Measure with the library driver. Do not use the binary, and do not use
`--check` either — that is a third thing again.**

**Measured exposure, and the limit of what I could measure:** we carry **290 `.wat.bad`, of which 31 are
MAIN-ONLY** (absent from grok's tip and its whole history) and have never faced this check. grok found
**16 of its own 281** were mis-named and renamed 13. **How many of our 31 start clean is UNKNOWN** — a
`--check` proxy suggested 11, but that is a third driver and I am reporting it as a hint, not a number.
**Measure it properly once the gate exists, and report the real figure.**

**The escape hatch is declared, and we use none of it yet.** A `.wat.bad` that legitimately starts clean
may be *banked* rather than renamed:

    ;; rune:lint(bad-is-banked) — <why the substrate SHOULD reject this> banked-by: <test fn name>

Category must be `bad-is-banked`; reason ≥ 24 chars; `banked-by:` must name the owning test. **0 of our
31 carry one today.** grok's three shapes for a mis-named file were: a retired premise (rename to `.wat`,
test flips to `is_ok()`), a test that starts up then INVOKES (a valid program is not "bad"), and a test
asserting startup *succeeded*. ⛔ **Read each file's own test before choosing** — rename, bank, or report.
Corpus floor is 200; we clear it at 290.

## ⚠ #352 — `diagnostic_output_is_deterministic` (351 lines)

Runs every `.wat.bad` under `tests/` **twice in fresh processes** (16 shards) and demands byte-identical
stdout, stderr and exit code. Fresh processes are the point — the variance it hunts is Rust's per-process
random `HashMap` seed.

Pre-flighted for you: its three `QUARANTINE` paths **all exist here**, so
`the_determinism_quarantine_is_pinned_and_its_paths_exist` will not red on arrival; its corpus is
**284** after quarantine, against a floor of 200. Its `INNER_RENDERER_DRIVER`
(`tests/lint/probe_c19_nested_type_var_render.wat.bad`) is **added by #352 itself**.

⚠ **The quarantine is pinned at exactly 3.** If a determinism defect surfaces in one of our main-only
fixtures, **adding a fourth entry is a finding to REPORT, not a line to slip in** — the gate says so
itself.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- **R21:** structural rewrites across many EXISTING `.wat` go through a recorded codemod. Not triggered
  here — the 23 `.wat` are new fixtures, none under `wat/`.
- ⛔ **WAT IN `.rs`/`.sh` STRING LITERALS** is R21's exception and this replay's most persistent defect
  source (finding 33) — six instances so far. Grep that side and SAY SO; "not applicable" is an answer.
- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture the whole block verbatim; name
  the exact assertion; STOP and report.
- ⛔ **Ending your turn ENDS you.** Verifications in the FOREGROUND.
- ⛔ **READ EVERY COUNT OFF THE DATA AS YOU WRITE THE SENTENCE** (findings 27/34/36/37).
- ⛔ **DIFF LIKE AGAINST LIKE** (finding 36): step blob vs step blob, never against `HEAD`. For "did we
  land their change unchanged", compare the **deltas**, not the blobs.
- ⛔ **A MAIN-ONLY ARTIFACT PINNED TO TEXT A REPLAYED STEP REWRITES IS ITS OWN CLASS** (finding 38, third
  instance). If a step rewrites a doc comment, a ledger or a rendering, **find what pins it here** — and
  note that two goldens in this tree bless by *different* mechanisms, one of which has no bless path at
  all. "Regenerate the goldens" is not one action.
- ⛔ **The record gate is PATH-BASED**: `^src/` needs `census:` + `nested-program-gate:`; `.rs` needs
  `lint-subset`, `kind(lib)`, `doctest`. Each verdict line on ONE line; put `(vs #N's …)` AFTER
  `no STOP-8`.
- ⛔ **NEVER `git filter-branch`** (finding 35) — detach / re-commit / rebuild-descendants only, and prove
  `git merge-base --is-ancestor origin/replay/grok-rete HEAD` plus empty `refs/original/` IN THE SCORE.
- ⛔ **NEVER FABRICATE A SHA.** Three executors have hand-typed a trailer and self-caught it.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** (E19) — every rename, bank, or rune you add.
- Every step commits as `REPLAY(grok-rete #N): <C's subject>` with the `(cherry picked from commit <sha>)`
  trailer.

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.** A gate that did not exist
at step N is not a reason to fold into N (the #184 precedent) — **#352 and #360 are exactly that case:
repair AT the step that lands them.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND:

    scripts/replay/verify-step-record.sh 5ba45a81f HEAD 341 360

exit 0, plus `git replace -l`, `git for-each-ref refs/original/`, `git merge-base --is-ancestor
origin/replay/grok-rete HEAD`, `git status --porcelain`. Then write `SCORE-7k-replay-batch-4k.md` (every
row, never blank) and a `REPLAY-LOG.md` section. Commit both. **Leave the tree CLEAN. Yield by ending your
turn — signal nothing.**
