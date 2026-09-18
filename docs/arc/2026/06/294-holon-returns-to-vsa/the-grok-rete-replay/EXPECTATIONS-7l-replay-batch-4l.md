# EXPECTATIONS 7l — replay batch 4l, grok-rete #361 → #380 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

A row is RUN before it is demanded (finding 34 §2). Stated honestly:

**Executed against HEAD `268263be4` and ALL PASS:** E15 (origin an ancestor; `refs/original/` empty), E16
(0 replace refs), tree clean, 360 REPLAY steps, E2 (10 docs-only), E2b (10 code), E7 (zero hazard rows; no
new `tests/lint/` gate file; no `wat-scripts/fixes/` edit), **all 5 M-status-absent paths created earlier
in the range**, **#377/#379's `wat/`+`wat-tests/` diffs are 100% comment lines** (37 and 83, zero code),
**16 of #375's 17 non-src files differ from grok's pre-image here**, `tests/cli/wat_cli.rs` carries **13**
`include_str!` and **0** `assert_edn_matches_file!`, and grok's quarantine path is **3 → 2 at #362 → 0 at
#375**.

**Predictions, not verified rows:** E3–E6, E8–E14, E17–E22.

⛔ **One thing I could NOT measure and did not pretend to.** Whether C20's source-order cure reaches the
**four extra fixtures we quarantined at #352** is unknown until #375 exists. They are the same class on
their face — same errors every run, order varies — but that is a hypothesis. **E4 scores measuring it
properly, with the run count stated, not matching my guess.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh 268263be4 HEAD 361 380` | exit 0; `sources match` |
| E2 | docs-only steps are docs-only | the **10** (#361 #364 #366 #368 #370 #372 #374 #376 #378 #380) | only `docs/`/`.md` |
| E2b | **and the inverse** | the 10 code steps | each carries ≥1 non-docs file |
| E3 | **the quarantine DRAINS** | `QUARANTINE_LEN` at #362 and at #380 | **6** after #362; **0** after #375 — or the survivors, each with measured evidence and a reason line, reported as a finding at the top of the SCORE |
| E4 | ⛔ **the drain was MEASURED, not assumed** | the SCORE | states how many fresh-process runs per fixture bought the claim, and the result per fixture. **This row passes on a real number, even if it differs from my hypothesis** |
| E5 | ⛔ **#375's goldens were REGENERATED here, not copied** | `git show` on #375 | the 13 `.edn` blessed via `UPDATE_EDN=1`; `tests/cli/wat_cli.rs`'s `include_str!` goldens captured from the binary; `wat_cli__check_bad.wat` **adapted**, not overwritten with grok's text |
| E6 | ⛔ **only ORDER moved in those goldens** | the SCORE + diffs | every regeneration reorders errors; no content, span or count change. Any such change is reported, not absorbed |
| E7 | ZERO hazard paths in range | `git diff --name-only` vs `absent-on-main.tsv`, new `tests/lint/` files, `wat-scripts/fixes/` | none |
| E8 | **R21 stayed untriggered** | `git show` on #377/#379 for `wat/` and `wat-tests/` | every changed line there is a comment line; any code line is reported |
| E9 | **#362's corpus comment carries OUR numbers** | `.config/nextest.toml` at #362 | re-derived here (grok's "268 / 266 / 2" not copied); the command and the result are in the SCORE |
| E10 | **#369's embedded wat was read against this tree** | the SCORE | each embedded form in `node_share_cost.rs` checked for retired syntax, explicitly; "not applicable" is an answer, silence is not |
| E11 | finding 33's class swept per code step | the SCORE | each code step answers explicitly |
| E12 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings` | green; clippy 0 — **the orchestrator's row** |
| E13 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor | predicted == actual. My pre-flight arithmetic, read off the diff per step: **+1** #362, **+4** #365, **+1** #371, **+3** #375, **+7** #377 = **+16**, so **5808**. Zero removed, zero `#[ignore]` moved, no macro-generated tests in range. The executor's own count is the check on mine |
| E14 | spot re-run of the walls | lint-subset + `kind(lib)` + doctests vs the last code step's lines | identical |
| E15 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor origin/replay/grok-rete HEAD`; `for-each-ref refs/original/` | ancestor YES; empty |
| E16 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E17 | **the SCORE discloses what BOUGHT each green** | read the SCORE | every regenerated golden, adapted fixture, quarantine decision and re-derived count appears in the row it affects |
| E18 | no verdict line WRAPPED | grep each pattern | one-line matches; `--diff`/`no STOP-8` contiguous |
| E19 | every `-E` filter selected N > 0 | each run's own `N tests run` | N > 0 |
| E20 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes this row; agreement does not earn it |
| E21 | **NO COUNTERPART ACTIVITY** | `.floor/` for foreign runs; `git status` for foreign artifacts; `.pulsare/` mtimes | none; frozen root untouched; no `mcp__pulsare__*` call |
| E22 | **a timing red was never re-run into green** | the SCORE | if a cost test reds at #367/#369, the SCORE carries the verbatim block and the arm, and the batch STOPPED (the 4i precedent) |
| E23 | no knowingly-red commit | no repair commit after #380 | none |

## ⛔ E3, E4, E5 and E6 are the rows that catch a lazy 4l

The cheap way through #375 is to take grok's golden text wholesale, and the cheap way through #375's
quarantine is to pin `QUARANTINE_LEN` to whatever happens to be green. Both would look identical to a
correct landing from the outside. **The SCORE shows the regeneration mechanism per golden and the run
count per fixture, or those rows fail.**

## ⛔ Every `-E` filter must select a NON-ZERO count

A filterset matching nothing runs zero tests and **exits 0**.

**Runtime prediction:** ~120–170 min. The weight is #375 (22 files, 16 regenerations, 3 new process-heavy
tests, plus the quarantine sweep), then #379 (28 files, comment-only) and #362.

**Trap doors named in advance:**
- **#375 — regenerate, never copy.** 16 of 17 non-src files differ from grok's pre-image here, and the
  three mechanisms differ. `include_str!` has no bless path.
- **#375/#362 — the quarantine is OURS (7), not grok's (3), and must reach 0.** A survivor is a finding.
- **#362 — a count comment must be re-derived**, not transcribed from grok.
- **#367/#369 — wall-clock asserts.** A red there is the 4i class: stop, do not re-run.
- **#369 — wat in `.rs` strings under `src/`, where no gate looks.**
- **#377/#379 — comment-only `.wat` edits.** A code line there contradicts the pre-flight: stop.

**What would make me reject the batch:** grok's golden text copied in where ours differs; a quarantine
number pinned to a green rather than to a measurement; a timing red re-run instead of reported; grok's
corpus digits transcribed into our `nextest.toml`; a fabricated trailer SHA; any `refs/original/` entry;
any `pulsare_yield`.
