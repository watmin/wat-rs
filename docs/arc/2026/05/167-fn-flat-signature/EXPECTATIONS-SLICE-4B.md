# Arc 167 slice 4b — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 15-30 minutes (sonnet agent).**

Reasoning: 16 mechanical sweep sites, all enumerated in BRIEF.
Single recipe, well-defined scope. Comparable to arc 159 slice 2's
mechanical sweep at small scale. Sonnet's wheelhouse.

This is the FIRST sonnet sweep using the new scripts in production
(`./scripts/cargo-test-summary.sh` + `./scripts/cargo-test-failures.sh`).
Calibration data: how does sonnet behave with the awk-pipe trigger
removed?

**Time-box (2× upper-bound): 60 minutes.** If sonnet still
iterating at 30 min, in-flight check; hard cap at 60.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A   | All 16 sites migrated | `./scripts/cargo-test-summary.sh` shows `passed: N failed: 0` |
| B   | Substrate code untouched | `git diff` shows changes ONLY inside `r#"..."#` raw strings inside `#[test]` blocks; no production functions touched |
| C   | Canonical parsers unchanged | `parse_fn_signature`, `parse_fn_signature_for_check`, `wat/core.wat` defn macro all unchanged in this slice |
| D   | New Vector arm in walker preserved | `walk_for_bare_primitives` Vector arm at `src/check.rs` unchanged |
| E   | Tests reach green via mechanical translation only | for each migrated test, the migration was the SOLE change required (no logic edits, no assertion edits) |
| F   | Slice branch up-to-date on remote | branch has new sweep commit pushed to origin |
| G   | Main untouched | `git log origin/main` unchanged |
| H   | Sonnet used scripts (not awk pipes) | sonnet's report references `./scripts/cargo-test-summary.sh` and/or `./scripts/cargo-test-failures.sh` for progress measurement |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Site that doesn't fit the recipe.** A test fixture might
  have unusual nesting, or be testing legacy syntax intentionally,
  or have escaped quoting. STOP and report; orchestrator decides.
- **Test fails post-migration that should pass.** If the
  mechanical translation is applied correctly but the test still
  fails, that's a real substrate dependency. STOP and report; do
  NOT modify substrate, do NOT rewrite the assertion.
- **More than 16 sites surface.** If iterating reveals MORE
  failing tests than the 16 in BRIEF (e.g., a test was passing on
  some other path that's now broken), STOP and report. Could
  indicate a deeper substrate issue.
- **Sonnet hallucinates a tool denial / off-task confusion.**
  This is the moment-of-truth for the scripts. If sonnet claims
  any tool is denied, check FM 7 / FM 16 — verify with a 30-sec
  probe. Report any false-denial claims.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial /
Mode C failed). Compare to predicted 15-30 min band.

Sonnet behavior with scripts: ___ (any awk-pipe denials? off-task
hallucinations? clean execution?)

## What's next (orchestrator-side, post-slice-4b)

When slice 4b ships green, slice 5 closure paperwork:
- SCORE-SLICE-3.md (deferred from earlier; cover the bundled
  sweep that closed via opus's 3 commits + revert + substrate fix)
- SCORE-SLICE-4.md (cover opus's slice 4 substrate retirement +
  honest delta A leading to slice 4b)
- SCORE-SLICE-4B.md (this slice's calibration record + sonnet
  behavior with scripts)
- INSCRIPTION.md (full arc 167 closure)
- 058 changelog row (one row covering the full arc)
- USER-GUIDE update (defn + fn sections show flat shape; legacy
  examples removed)
- Atomic squash-merge slice branch to main
- Branch retained on origin as audit trail

## SCORE artifact

Sonnet's report writes to chat; orchestrator commits SCORE-SLICE-4B.md
to slice branch after scoring all rows + reviewing the diff.
