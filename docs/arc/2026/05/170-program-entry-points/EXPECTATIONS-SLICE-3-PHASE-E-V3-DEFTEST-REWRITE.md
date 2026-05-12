# Arc 170 slice 3 Phase E V3 EXPECTATIONS (sonnet scorecard)

**One spawn.** Wat-side macro rewrite consuming the now-shipped substrate `do`-splice (Gap C V2 + Gap D). Two macro bodies rewritten; workspace stays at 2205/0.

## Independent prediction

**Runtime band:** 30-50 min sonnet.

**Hard cap:** 100 min (2×).

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::deftest` body uses `(:wat::core::do ~@prelude (:wat::core::define ...))` shape with `(:wat::test::run-hermetic ~body)` | grep — no `run-sandboxed-ast` or `:wat::core::forms` in deftest expansion |
| B | `:wat::test::deftest-hermetic` body rewritten (Path A collapse OR Path B keep-as-alias; surface choice) | grep — no `run-sandboxed-hermetic-ast` |
| C | `make-deftest` + `make-deftest-hermetic` factories still compose | workspace test passes |
| D | `cargo test --release --workspace --no-fail-fast`: 2205 passed / 0 failed | full test run |
| E | Documentation headers at wat/test.wat:260+ updated to reflect new expansion | manual review |
| F | Phase F readiness inventory: remaining run-sandboxed-* callers documented | SCORE inventory |

**6 rows.** All must PASS.

## Implementation approach

1. **Verify substrate readiness** (5 min): `cargo test --release --test probe_do_splice_def` (3 expected pass)
2. **Rewrite deftest** (10-15 min): wat/test.wat:305-318
3. **Rewrite deftest-hermetic** (5-10 min): wat/test.wat ~338+
4. **Verify factories** (5 min): make-deftest + make-deftest-hermetic at ~380+
5. **Workspace verify** (15-20 min): full cargo test
6. **SCORE produce** (5-10 min): including Phase F readiness

## What sonnet produces

- `wat/test.wat` modified (deftest macro + deftest-hermetic macro + headers)
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-V3-DEFTEST-REWRITE.md` with:
  - 6-row scorecard
  - deftest-hermetic Path A vs Path B choice with rationale
  - Prelude semantic shift impact (any tests affected)
  - Updated documentation header wording
  - Phase F readiness — exact callers remaining
  - Honest deltas (≥ 3)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Modify `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` substrate (Phase F)
- Modify `run-ast` / `run-hermetic-ast` wrappers (Phase F)
- Modify `wat/kernel/hermetic.wat` (Phase F)
- Modify ANY test call site (macro signature unchanged)
- Touch anything under `docs/arc/`
- Commit / push / git add / git restore
- Use deferral language anywhere in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Add new substrate features
- Run hooks bypass / `--no-verify`
- Mass-modify call sites to fix failures (the macro is supposed to be call-site compatible)
- Auto-fix prelude semantic mismatches (surface them; user decides)

## Verification commands

```bash
# 1. Baseline
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# 2. Substrate probe (proves do-splice substrate fix is in place)
cargo test --release --test probe_do_splice_def 2>&1 | tail -5

# 3. deftest expansion shape check
grep -A 12 "^(:wat::core::defmacro" wat/test.wat | grep -A 12 ":wat::test::deftest" | head -15

# 4. No run-sandboxed-ast in deftest expansion
grep -A 8 ":wat::test::deftest$\|:wat::test::deftest " wat/test.wat | grep "run-sandboxed-ast"
# Expected: empty

# 5. Workspace post-rewrite
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2205 passed / 0 failed
```

## Expected workspace delta

- Baseline: 2205 passed / 0 failed
- Post-Phase-E-V3: 2205 passed / 0 failed (macro signature unchanged from caller view; substrate path swap should be transparent)

## Honest delta categories (anticipated)

1. **deftest-hermetic Path A vs Path B** — surface choice + rationale; user can adjust if they prefer the other
2. **Prelude semantic shift impact** — any specific tests where top-level-expansion vs sandbox-internal makes a difference
3. **Factory composition** — verify `~~default-prelude` double-unquote still works through the new outer shape
4. **Documentation header wording** — surface new explanation of expansion
5. **Phase F readiness** — list each remaining run-sandboxed-* caller in wat/test.wat + wat/kernel/hermetic.wat with disposition
6. **TestResult typealias verification** — confirm `:wat::test::TestResult = :wat::kernel::RunResult` and the macro return type is consistent
7. **Anything unexpected** — particularly any test that fails (don't fix; report)
