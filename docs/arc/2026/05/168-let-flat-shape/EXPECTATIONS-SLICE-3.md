# Arc 168 slice 3 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 30-60 minutes (opus agent).**

Reasoning: pure deletion work mirroring arc 167 slice 4 (which
ran ~30 min opus single commit). Arc 168 slice 3's substrate
deletion list is the same shape: walker variant + Display +
Diagnostic + walker body + freeze.rs registration + legacy
parser arms + vacuous tests. Nothing surprising should surface
because slice 2 closure already exercised the canonical path
end-to-end.

**Time-box (2× upper-bound): 120 minutes.** If opus still
iterating at 60 min, in-flight check; hard cap at 120.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — Walker variant + Display + Diagnostic deleted | `grep -rn "BareLegacyLetBindings" src/`: 0 hits | ✓ |
| B — Walker body + registration deleted | `grep -rn "validate_legacy_let_bindings\|walk_for_legacy_let_bindings" src/`: 0 hits | ✓ |
| C — Migration message text gone | `grep -rn "let bindings must be a vector" src/`: 0 hits | ✓ |
| D — `eval_let` legacy List arm deleted (if present) | `eval_let` produces clean `MalformedForm` if outer is non-Vector; no fall-through to legacy | ✓ |
| E — `parse_let_binding` typed-legacy `(name :T)` arm deleted | binder must be Symbol (single) or Vector-of-Symbols (destructure); `(name :T)` produces clean `MalformedForm` | ✓ |
| F — Check-side parallel retirement | `infer_let` + check-side parsers mirror the runtime retirements | ✓ |
| G — Vacuous tests retired (DELETED preferred) | walker-firing assertions in `tests/wat_arc168_let_flat_shape.rs` either deleted or replaced with `MalformedForm` assertions | ✓ |
| H — `cargo build --release --workspace` green | substrate compiles cleanly post-retirement | ✓ |
| I — Slice 1 substrate consumer paths preserved | `parse_let_binding` Symbol + Vector branches unchanged; `eval_let` Vector-outer + multi-form body unchanged | ✓ |
| J — Walker scoping infrastructure preserved | `walk_for_bare_primitives` Vector-arm fix from arc 167 slice 3 unchanged | ✓ |
| K — Inline pipeline verifies clean | `passed: 2077 failed: 5` (the 5 pre-existing kernel/signal unrelated) | ✓ |
| L — Slice branch on remote | branch carries the retirement commit(s); main untouched | ✓ |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Hidden dependency on legacy code path.** If some subsystem
  outside the deletion list referenced the legacy parser arm or
  the walker variant, surface as honest delta. Could indicate
  arc 168 slice 1 or slice 2 missed a site.
- **Test fails post-retirement that should pass.** If a test in
  `tests/wat_arc168_let_flat_shape.rs` (or elsewhere) fails
  unexpectedly post-retirement, STOP and report; do NOT modify
  the test to make it pass.
- **More than expected sites surface.** If the substrate has
  more legacy-parser-arm references than the deletion list
  enumerates, surface — could indicate slice 1's substrate work
  introduced more transitional code than acknowledged.
- **FM 5 trap.** If you find yourself wanting to keep a legacy
  arm "just in case some path needs it," STOP. The slice 2
  follow-up commit `b220846` caught this exact pattern. Vector-
  only is the discipline; bridging breaks it.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial /
Mode C failed). Compare to predicted 30-60 min band.

Sites deleted by file:
- `src/check.rs`: ___ sites
- `src/runtime.rs`: ___ sites
- `src/freeze.rs`: ___ sites
- `tests/wat_arc168_let_flat_shape.rs`: ___ tests retired

Honest deltas surfaced: ___ (count + brief).

## What's next (orchestrator-side, post-slice-3)

When slice 3 ships green:
- Slice 4 BRIEF + EXPECTATIONS for src/ lib unit-test fixture
  sweep (mirror of arc 167 slice 4b precedent). Predicted 15-30
  min sonnet.
- Then slice 5 closure paperwork: SCOREs 1-4 + INSCRIPTION + 058
  row + USER-GUIDE update + atomic squash-merge.

## SCORE artifact

Opus's report writes to chat; orchestrator commits SCORE-SLICE-3.md
to slice branch after scoring all rows + reviewing the diff.
