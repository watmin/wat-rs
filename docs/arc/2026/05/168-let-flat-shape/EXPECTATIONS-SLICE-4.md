# Arc 168 slice 4 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 60-120 minutes (sonnet agent).**

Reasoning:
- Arc 167 slice 4b precedent: 16 fixtures, ~24 min sonnet
- Arc 168 slice 4: 81 fixtures = ~5× scale
- Linear scaling: ~120 min upper bound
- BUT slice 2 sonnet calibration (delta D in SCORE-2): ~2 tool
  calls per site for Edit-tool-per-fixture approach; 81 × 2 = ~162
  tool calls ≪ slice 2's 1107 calls for 563 sites
- Wider band than arc 167 slice 4b reflects the two-legacy-shape
  retirement (outer-list + typed-single binder); fixtures may
  vary in shape mid-sweep

**Time-box (2× upper-bound): 240 minutes.** If sonnet still
iterating at 120 min, in-flight check; hard cap at 240.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — `cargo test` count drops to 5 failed | inline pipeline shows `passed: 2080 failed: 5` (the 5 pre-existing kernel/signal unrelated) | ✓ |
| B — `src/runtime.rs` swept | post-sweep grep `'\(:wat::core::let\b.*\(\('` src/runtime.rs: 0 hits inside `#[test]` blocks | ✓ |
| C — `src/check.rs` swept | post-sweep grep equivalent: 0 hits inside `#[test]` blocks | ✓ |
| D — Substrate untouched | `git diff src/runtime.rs src/check.rs` shows ONLY changes inside `#[test]` raw-string fixtures (no eval/infer/parser/walker edits) | ✓ |
| E — Tests/assertions untouched | NO test logic edits, NO assertion edits — translation-only | ✓ |
| F — Mechanical translation only | each migration is binder-shape change + body unchanged; no semantic reshapes | ✓ |
| G — Slice branch on remote | branch carries the sweep commit(s); main untouched | ✓ |
| H — Inline pipeline verifies clean | sonnet's report references the inline pipeline; no script invocations | ✓ |
| I — All three legacy shapes covered | bare-symbol legacy `((name expr) ...)`; typed-single `((name :T) expr)`; List-destructure `((a b c) rhs)`; empty `()` | ✓ |
| J — `wat/core.wat` defn macro untouched | `git diff wat/core.wat`: no changes | ✓ |
| K — FM 5 held | no parser arms re-added; no test fixtures rewritten to "make passing"; legitimate STOP-and-report on any non-mechanical site | ✓ |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Hidden test depending on legacy shape semantics.** Fixture's
  test assertion depends on something only the legacy parser
  produced. Surface as honest delta; orchestrator decides if test
  should retire.
- **Test using nested-quoted wat in a way the recipe can't
  mechanically translate.** Surface; STOP-and-report.
- **More than expected sites surface.** If sweep reveals more than
  81 fixtures (additional shapes the slice 3 deletion list missed),
  surface — could indicate arc 168 had more legacy shapes than
  acknowledged.
- **FM 5 trap.** If you find yourself wanting to rewrite a test's
  assertion or re-add a parser arm to make a test pass, STOP. The
  slice 2 follow-up commit `b220846` and slice 3's discipline
  caught this exact pattern.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial /
Mode C failed). Compare to predicted 60-120 min band.

Site count by file:
- `src/runtime.rs`: ___ / 70 expected
- `src/check.rs`: ___ / 11 expected

Honest deltas surfaced: ___ (count + brief).

Tool-call ratio (calibration for future similar-shape sweeps):
___ tool uses for ___ sites = ___ calls/site.

## What's next (orchestrator-side, post-slice-4)

When slice 4 ships green:
- Slice 5 closure paperwork:
  - SCORE-SLICE-1 (slice 1's ship was never SCORE-d; closure
    convention)
  - SCORE-SLICE-4 (this slice)
  - INSCRIPTION.md (arc 168 closure)
  - 058 changelog row (FOUNDATION-CHANGELOG.md in trading lab)
  - USER-GUIDE update (let flat-shape + multi-form body)
  - Atomic squash-merge to main as one squash commit

Plus arc 169 (number reserved post-slice-5) opens to investigate
the 5 pre-existing kernel/spawn/signal failures.

Plus future arc opens for struct-destructure form A
(`{outcome grace-residue} p`) settled in conversation 2026-05-08.

## SCORE artifact

Sonnet's report writes to chat; orchestrator commits SCORE-SLICE-4.md
to slice branch after scoring all rows + reviewing the diff +
re-running the inline pipeline locally for FM 9 verification.
