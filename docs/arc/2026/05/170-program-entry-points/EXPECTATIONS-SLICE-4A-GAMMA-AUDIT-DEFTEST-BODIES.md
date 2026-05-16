# Arc 170 Slice 4a-γ-audit EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-4A-GAMMA-AUDIT-DEFTEST-BODIES.md`
**Task:** #317

## Independent prediction

**Runtime band:** 30–60 minutes Mode A.

Reasoning:
- ~224 deftest sites; per-site classification ~10–20 seconds (read body + apply three greps for R1/R2/R3)
- Per-pattern bulk classification likely possible (e.g., all of wat-tests/holon/ has consistent shape; can batch-classify by file)
- Audit doc writing: ~10 min for table construction
- Sanity check on deftest-hermetic sites: ~5 min

**Time-box:** ScheduleWakeup at 120 min (2× upper-bound).

## Predicted distribution

Educated guess based on the codebase's character:

| Class | Predicted count | Rationale |
|---|---|---|
| Safe for thread (no rules) | 150–180 (~70–80%) | Most tests panic via `assert-eq` only; no stdio capture, no stdio verbs, no `set-*!` |
| R1 only (stdio slot reads) | 5–15 | Tests of `assert-stdout-is` / `assert-stderr-matches` infrastructure (test.wat has several after 4a-β) |
| R2 only (stdio verbs) | 5–10 | Tests that drive println directly (ambient-stdio.wat-like) |
| R3 only (set-! family) | 10–20 | Capacity tests (wat_bundle_capacity), router tests, redef tests (arc157) |
| Multiple rules | 5–10 | Tests combining capacity-config + stdio capture |
| Total flagged | 25–55 (~10–25%) | |

These are predictions; the audit produces actuals.

## SCORE methodology (for the AUDIT doc)

The audit doc IS the SCORE for this slice — no separate scorecard. Quality is evaluated by:

- **Coverage:** total sites audited matches enumerable population (`grep -rEn ":wat::test::deftest\b" ... | grep -v "\.md:" | grep -vE "^[^:]+:[0-9]+:\s*;;" | wc -l` matches the audit's total).
- **Classification fidelity:** spot-check 5 random flagged sites + 5 random unflagged sites; verify the rules-fired column matches the actual body content.
- **Honest deltas surfaced:** any helper-obscured cases, ambiguous bodies, over-hermetic candidates flagged.

## Honest deltas to watch for

- **Helper-function indirection.** Some deftest bodies may call a helper that internally fires a rule (e.g., helper writes to stdout, body just calls helper). Lexically the deftest body doesn't fire R2, but semantically it does. Surface these; orchestrator decides whether to follow indirection in the worklist or treat lexically.

- **Tests-of-tests.** wat-tests/test.wat has several "test the testing harness" deftests that intentionally exercise stdio-capture / failure-path semantics. These should mostly land flagged correctly (they read RunResult slots); just confirm.

- **deftest-hermetic over-hermetic candidates.** If any `:wat::test::deftest-hermetic` site fires zero rules, it's using hermetic unnecessarily. Could downgrade after the flip. Flag without recommending downgrade in this slice — that's a separate cleanup.

- **Make-deftest variants.** `:wat::test::make-deftest` produces configured deftest macros (e.g., `:deftest-ambient` in ambient-stdio.wat). Those expansions go through deftest, so the underlying body audit applies. Note configured-variant call sites in the audit if they expand to deftest+three-rule-relevant code.

- **Multi-rule cases.** A body that fires R1 AND R3 (e.g., asserts on stdout AND calls set-capacity-mode!) is unambiguously hermetic-required. Not surprising; just confirm the audit captures these correctly.

- **Edge case: `:wat::test::run-thread` callers from 4a-β as helper.** A deftest whose body calls `(:wat::test::run-thread ...)` doesn't itself fire any rule lexically. The inner run-thread body has its own three-rule constraint, but the OUTER deftest reads only the run-thread's `RunResult.failure` slot (proven safe by 4a-β). These should land in the "safe for thread" set. Confirm.

## Workspace baseline (commit `e43c928`)

Working tree clean; tip is the hibernation commit. No build/test required for the audit (no code edits). Audit doc is the only artifact.

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 30–60 min | TBD |
| Total deftest sites audited | ~224 | TBD |
| Total flagged for decoration | 25–55 (~10–25%) | TBD |
| R1 (stdio reads) | 5–15 | TBD |
| R2 (stdio verbs) | 5–10 | TBD |
| R3 (set-! family) | 10–20 | TBD |
| Multi-rule | 5–10 | TBD |
| deftest-hermetic over-hermetic candidates | 0–5 | TBD |
| Helper-obscured cases | 0–10 | TBD |
| Mode | A (clean) | TBD |

## What happens after this slice

The audit doc's flagged list becomes the worklist for #318 (decorate). #318's BRIEF will:
- Take the flagged-site list as input
- Mechanically rename each site's `:wat::test::deftest` → `:wat::test::deftest-hermetic`
- Verify build clean + tests stay within baseline
- Land

Then #314 (4a-γ-flip) becomes safe: every hermetic-required deftest is already decorated; the deftest macro body flip from `run-hermetic` → `run-thread` affects only the safe-for-thread set.
