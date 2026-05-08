# Arc 165 slice 1 — SCORE

Spawned 2026-05-08; sonnet completed in ~18 minutes (predicted 30-45
min band; actual under lower bound). Mode A clean.

Commit: `e1f366b` — arc 165 slice 1: tuple → Tuple Pascal-case rename.

## Scorecard

| Row | Verified by | Pass |
|-----|-------------|------|
| A — `Value::Tuple(_) => "wat::core::Tuple"` at runtime.rs:480 | git diff confirms exact line | ✓ |
| B — Eval arm key flipped at runtime.rs:3081 | sonnet caught duplicate; line removed instead of renamed (line 3082 already had PascalCase arm) | ✓ honest correction |
| C — Type-comparison literal flipped at runtime.rs:3640 | git diff confirms | ✓ |
| D — runtime.rs:5607 head field flipped | git diff confirms | ✓ |
| E — check.rs:8959 head field flipped | git diff confirms | ✓ |
| F — Pattern 2 poison unchanged + one-line comment added | git diff confirms shape preserved + comment addition | ✓ |
| G — Test fixture at check.rs:14463 flipped | git diff confirms | ✓ |
| H — New test file with 4 cases | tests/wat_arc165_tuple_pascal.rs created; 4 tests all PASS | ✓ |
| I — `cargo test --release --workspace --no-fail-fast` clean | orchestrator re-ran post-commit; 0 failed across all test binaries | ✓ |
| J — Pre-existing test count unchanged | no regressions; the new tests are additions | ✓ |
| K — Comment text updated per BRIEF | runtime.rs + check.rs comment rewrites confirmed in diff | ✓ |

## Honest deltas (recorded for calibration)

1. **Duplicate eval-arm at runtime.rs:3081/3082** — pre-arc-165 BOTH
   the lowercase and PascalCase arms existed; renaming would have
   created an unreachable second arm. Sonnet's correction: remove
   the lowercase arm. BRIEF's "13 expected sites" accordingly
   resolves to 12 string-literal flips + 1 arm removal. Calibration
   note: **future BRIEFs for canonical-form renames should grep BOTH
   sides** (legacy AND canonical) before listing expected sites; the
   substrate may be mid-migration with both arms registered.

2. **Missed `expected: "tuple"` at runtime.rs:3964** — sonnet
   reported the orphan; orchestrator closed it in the same commit
   per FM 11 (no known defect left unfixed). Calibration note:
   greps for `wat::core::X` miss bare `"X"` strings used as
   error-message prose; future audits should also grep the bare
   identifier.

3. **Pre-existing latent defect at TypeExpr::Tuple comparison** —
   confirmed unreachable as true-positive pre-arc-165 (`type_name`
   returned `"tuple"` while comparison expected `"wat::core::tuple"`).
   Post-arc-165 both sides aligned at `"wat::core::Tuple"`. No
   pre-existing tests failed from the alignment, confirming the path
   was unreachable in practice (consistent with EXPECTATIONS row C
   prediction).

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| 30-45 min upper-band, 90-min hard cap | ~18 min | A clean (under lower bound) |

The predicted band assumed the BRIEF would itemize 13 sites
mechanically; sonnet's discovery of the duplicate arm + the
return-type syntax mismatch added one cargo-test cycle but did not
push runtime out of band. Calibration trend: small mechanical
sweeps continue to ship under predicted lower bounds (arc 163 slice
3f: ~25 min; arc 165 slice 1: ~18 min). Future single-slice
mechanical-rename predictions can lean toward 15-30 min.
