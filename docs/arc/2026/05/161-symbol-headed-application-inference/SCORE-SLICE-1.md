# Arc 161 — Slice 1 SCORE

**Scored 2026-05-07 by orchestrator.** Sonnet shipped Mode A in
9m11s (predicted 10-15 min Mode A; under-band).

## Scorecard

| Row | Status | Notes |
|---|---|---|
| R1 | ✓ | Workspace pre-fix: 1 failed (`deftest_wat_telemetry_test_svc_tel_null_translator`); confirmed by sonnet's pre-flight run |
| R2 | ✓ | Workspace post-fix: 0 failed; orchestrator verified 2047 passed / 0 failed |
| R3 | ✓ | Telemetry null-translator test passes (in 2047 count) |
| R4 | ✓ | Single file edited: `src/check.rs` (verified via `git diff --stat`: 1 file changed, 56 insertions(+), 7 deletions(-)) |
| R5 | ✓ | Working tree dirty pre-orchestrator-commit (no sonnet commits) |
| R6 | ✓ | Report explicitly names `reduce`, `apply_subst`, `unify`; confirms `infer_spawn` 7556-7589 mirror |
| R7 | ✓ | Honest delta surfaced (the regression-and-correction below) |

## Honest delta — the mid-flight correction

Sonnet's first draft followed the BRIEF template exactly (`other`
arm emits `TypeMismatch` for non-Fn head). This regressed
`multiple_lambda_sites_post_retirement_silently_alias`.

**Root cause:** retired `(:wat::core::lambda ...)` forms have no
checker arm in the keyword-match block; the substrate's
schema-lookup fall-through (line 4432-4435) recurses into args.
The lambda's params-list `(() -> :wat::core::i64)` is itself a
non-keyword list whose head `()` (empty list) infers to
`Some(TypeExpr::Tuple([]))`. The first-draft value-head branch
saw this Tuple as a non-Fn head and emitted a false-positive
`TypeMismatch`.

**Sonnet's correction:** changed the `_ =>` arm to silent
`return None` + arg recursion. Justification: the value-head
branch is an INFERENCE OPPORTUNITY at a fall-through site, not
a strict call-site checker. Emitting an error here creates
false positives wherever a non-keyword list is recursed-into by
upstream paths. The `infer_spawn` reference pattern is also
silent on non-Fn heads (returns a `ProgramHandle<?>` placeholder
type, no error). The corrected arc 161 branch matches that
precedent: additive only — `None → correct type when head IS Fn`;
never `None → false-positive error`.

This is exemplary substrate discipline: sonnet caught its own
regression mid-sitting, applied the principled fix consistent
with prior precedent, and surfaced the chain of reasoning in
the report. No orchestrator-side rework needed.

## Other deltas

- **Inline-expression heads** (`((make-fn) arg1)`) work via the
  unified `infer(head)` path. No separate handling.
- **`reduce` is the right helper** for this site; no separate
  `apply_subst` pre-step needed (`reduce` covers Var-walk + alias
  expansion in one normalization).
- **`unify` works cleanly** for arg-vs-param at this layer; same
  shape as the keyword-headed branch.

## Calibration

- **Predicted band:** 10-15 min Mode A
- **Actual:** 9m11s, Mode A
- **Pattern:** under-band (faster than predicted by ~10%). Trend
  consistent with arc 158a/160 sonnet sweeps. Tighten future Mode-A
  predictions on single-site substrate fixes to 8-12 min.

## Commit chain

- `f6ab13f` arc 161 slice 1: Symbol-headed application inference
  (this commit)

## Next

Slice 2 — closure paperwork (orchestrator-side). Then close arcs
in dependency order: 161 → 160 → 159. FOUNDATION-CHANGELOG rows
for all three batched at the end.
