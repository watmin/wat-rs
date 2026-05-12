# Arc 170 slice 3 phase E V2 — EXPECTATIONS (sonnet scorecard)

**Re-spawn after V1 stopped on the wrong splicer.** Use `:wat::core::do` (verified top-level splicer per arc 157 § Scope Q1 + `src/check.rs:6848`).

## Independent prediction

**Runtime band:** 45-90 min sonnet. Straight path (splicer settled); the bulk is workspace verification across 223 sites.

**Hard cap:** 240 min.

## Scorecard (8 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::deftest` body uses top-level `(:wat::core::do ~@prelude define)` shape | grep — no `run-sandboxed-ast` in deftest expansion |
| B | `:wat::test::deftest-hermetic` body rewritten (Layer 1 hermetic-by-default) | grep |
| C | Probe demonstrates top-level `do` splicing works for macro emissions | probe test passes |
| D | Workspace at 0 failed AFTER deftest rewrite | full cargo test |
| E | TestResult / RunResult reconciliation correct | grep + cargo test |
| F | `cargo check --release` green | clean |
| G | make-deftest factories cascade documented | SCORE |
| H | SCORE includes honest deltas + Phase F readiness check | manual review |

## Implementation approach

Same five phases as V1 BUT with corrected splicer:

- **Phase 1**: Probe with `:wat::core::do` (not `forms`). Expected to work per arc 157 doctrine.
- **Phase 2**: Rewrite deftest macro body using `(:wat::core::do ~@prelude (:wat::core::define ~name -> :RunResult (:wat::test::run-hermetic ~body)))`.
- **Phase 3**: Rewrite deftest-hermetic identically.
- **Phase 4**: Full workspace cargo test. Expected 2199/0.
- **Phase 5**: Check remaining `run-sandboxed-*` callers; document Phase F prerequisites.

## What sonnet should produce

1. **Code changes:**
   - `wat/test.wat` — deftest + deftest-hermetic macro bodies rewritten
   - Possibly a tiny probe wat-test file (commit OR delete after verifying)
2. **SCORE doc:** `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-V2-DEFTEST-REWRITE.md`
3. **Do NOT commit.** Orchestrator atomic-commits after verification.

## What sonnet should NOT do

- Do NOT modify any test call sites (no 223-site sweep)
- Do NOT modify Layer 1/2 macros / drivers
- Do NOT retire run-sandboxed-* substrate verbs (phase F)
- Do NOT touch BareLegacy* / spawn.rs / Process<I,O> struct fields
- Do NOT use deferral language in SCORE
- If top-level `do` splicing probe fails, STOP and report (substrate finding)
- If individual tests fail in scenario-specific ways, STOP and report

## Verification commands

```bash
# Baseline
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# After deftest rewrite — confirm no run-sandboxed-ast reference in deftest body
grep -A 20 "defmacro" wat/test.wat | grep "run-sandboxed"

# Workspace verify
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# Phase F readiness — remaining callers
grep -rn ":wat::kernel::run-sandboxed" wat/ wat-tests/ crates/ src/ 2>/dev/null | grep -v "defmacro" | head -20
```

## Expected workspace delta

- Baseline: 2199 passed / 0 failed
- Post Phase E V2: 2199 passed / 0 failed (no test count change; macro rewrite is internal)

## Honest delta categories (anticipated)

1. **Probe outcome** — top-level `do` splicing for macro emissions confirmed (or surprising findings)
2. **TestResult/RunResult typealias** — sonnet's V1 SCORE notes ≡ relationship; verify still holds
3. **Hermetic-by-default performance** — any observable cargo test slowdown
4. **make-deftest factory cascade** — transitive or manual handling
5. **Phase F readiness** — remaining `run-sandboxed-*` callers count + locations
