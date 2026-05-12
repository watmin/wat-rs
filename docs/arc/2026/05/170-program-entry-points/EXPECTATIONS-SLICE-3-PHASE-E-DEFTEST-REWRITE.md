# Arc 170 slice 3 phase E — EXPECTATIONS (sonnet scorecard)

**One spawn.** Rewrite `:wat::test::deftest` + `:wat::test::deftest-hermetic` macro bodies to use `run-hermetic` (Layer 1). Workspace must stay at 0 failed across 223 deftest call sites with NO call-site changes.

## Independent prediction

**Runtime band:** 60-150 min sonnet. Macro rewrite is small; the 223-site verification is the bulk. Some tests may surface real failures needing investigation.

**Hard cap:** 300 min.

## Scorecard (8 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::deftest` macro body uses `run-hermetic` (no `run-sandboxed-ast` reference) | grep |
| B | `:wat::test::deftest-hermetic` body uses `run-hermetic` (no `run-sandboxed-hermetic-ast`) | grep |
| C | Mechanism A verified (forms-splices-at-top) OR Mechanism B chosen with documented rationale | SCORE |
| D | Workspace at 0 failed AFTER deftest rewrite | full cargo test |
| E | TestResult vs RunResult reconciled | grep + cargo test |
| F | `cargo check --release` green | clean |
| G | make-deftest factories disposition documented (cascade or follow-up) | SCORE |
| H | SCORE documents honest deltas (≥ 3) including hermetic-by-default note | manual review |

**8 rows.**

## Approach

**Phase E1 — Verify the splicing mechanism FIRST before touching deftest.**

Write a tiny defmacro experiment:
```
(:wat::core::defmacro
  (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
  `(:wat::core::forms
     (:wat::core::define (:my::helper -> :wat::core::i64) 42)
     ~body))

(:my::probe (:wat::core::define (:my::main -> :wat::core::i64) (:my::helper)))
```

Run; verify both defines land at file top level. If yes, Mechanism A works. If no, surface in SCORE; pivot to Mechanism B.

**Phase E2 — Rewrite deftest macro body.**

Replace `run-sandboxed-ast` expansion with the run-hermetic-based expansion. Keep the macro's 3-arg signature.

**Phase E3 — Rewrite deftest-hermetic identically.**

Since Layer 1 IS hermetic-by-default, deftest-hermetic and deftest now do the same thing. Either keep them as duplicates or alias deftest-hermetic → deftest.

**Phase E4 — Run full workspace.**

Expected: 2199 passed / 0 failed (nothing changes — same tests, same workings).

If ANY test fails:
- Investigate root cause per test
- If root cause is the macro rewrite (e.g., prelude not visible in scope), it's a real bug to fix at the macro layer
- If root cause is a TEST-LEVEL scenario incompatibility (test depended on in-process semantics that don't exist under fork), STOP and report — that's a separate slice's work, not a workaround target

**Phase E5 — Document Phase F prerequisites.**

After E ships, grep for any remaining callers of `run-sandboxed-ast` / `run-sandboxed-hermetic-ast`. If zero (Phase F path is open), document in SCORE. If non-zero (other callers exist in the codebase), document them as Phase F's targets.

## What sonnet should produce

1. **Code changes:**
   - `wat/test.wat` — deftest + deftest-hermetic macro bodies rewritten (Layer 1 expansion)
   - Possibly the test.wat header comments updated
   - NO test source files modified (call sites stay verbatim — that's the whole point)
2. **SCORE doc:** `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-DEFTEST-REWRITE.md` mirroring prior SCORE structure:
   - Scorecard verification
   - Mechanism A vs B outcome
   - TestResult/RunResult reconciliation
   - make-deftest factory disposition
   - Honest deltas (≥ 3 categories)
   - Files modified
   - What's next (Phase F)
3. **Do NOT commit.** Orchestrator atomic-commits after scoring verification.

## What sonnet should NOT do

- Do NOT modify any test call sites (the whole point of this slice is no migration needed)
- Do NOT modify Layer 1 / Layer 2 macros or drivers
- Do NOT retire run-sandboxed-* substrate verbs (phase F)
- Do NOT touch BareLegacy* walker / spawn.rs / Process<I,O> struct fields
- Do NOT use deferral language in SCORE
- If Mechanism A fails AND Mechanism B requires test call-site modifications, STOP and report — do not start a 223-site sweep without orchestrator direction
- If individual tests fail in scenario-specific ways requiring per-test rewrites, STOP and report

## Tools required

- Read / Edit / Bash (cargo, git, grep)
- Write for SCORE doc + maybe small probe wat-test file
- No Agent invocations

## Verification commands

```bash
# Baseline
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# Mechanism A probe — write tiny test file with macro that uses forms-splice
# (sonnet authors the probe; runs it; checks both defines registered)

# After deftest rewrite — verify no run-sandboxed-ast in deftest macro body
grep -n "run-sandboxed" wat/test.wat

# Workspace verify
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# Phase F readiness — any remaining callers of run-sandboxed-*?
grep -rn ":wat::kernel::run-sandboxed" wat/ wat-tests/ crates/ src/ 2>/dev/null | grep -v ":wat::kernel::run-sandboxed.*defines\|defmacro" | head -20
```

## Expected workspace delta

- Baseline: 2199 passed / 0 failed
- Post Phase E: 2199 passed / 0 failed (NO test count change — same tests, same passing; the macro rewrite is internal)

## Honest delta categories (anticipated)

1. **Mechanism A vs B outcome** — which compiled + why
2. **TestResult vs RunResult** — same struct? Different? Conversion needed?
3. **Hermetic-by-default performance** — observe any cargo test slowdown; surface if meaningful
4. **make-deftest factory cascade** — did rewrites flow through factories or do they need separate work
5. **Anything unexpected** — surfaced during 223-site verification
