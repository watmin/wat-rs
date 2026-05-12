# Arc 170 slice 3 Phase E V4 EXPECTATIONS (sonnet scorecard)

**One spawn.** V3 re-attempt with substrate ready (Gap E `3d65b82`). Identical target shape; same constraints; updated baseline.

## Independent prediction

**Runtime band:** 30-50 min sonnet.

**Hard cap:** 100 min (2×).

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::deftest` body uses `(:wat::core::do ~@prelude (:wat::core::define ...))` + `(:wat::test::run-hermetic ~body)` | grep — no `run-sandboxed-ast` or `:wat::core::forms` in deftest expansion |
| B | `:wat::test::deftest-hermetic` body rewritten | grep — no `run-sandboxed-hermetic-ast` |
| C | `make-deftest` + `make-deftest-hermetic` factories still compose | workspace test passes |
| D | `cargo test --release --workspace --no-fail-fast`: **2209 passed / 0 failed** (Gap E baseline) | full test run |
| E | Documentation header at wat/test.wat:260+ updated | manual review |
| F | Phase F readiness inventory in SCORE | SCORE inventory |

**6 rows.** All must PASS.

## Implementation approach

1. **Verify substrate readiness** (2 min): `cargo test --release --test probe_do_splice_define && cargo test --release --test probe_let_splice_define` — 4 expected pass
2. **Rewrite deftest** (5-10 min): wat/test.wat ~305
3. **Rewrite deftest-hermetic** (5 min): wat/test.wat ~338; Path A collapse (V3 precedent)
4. **Verify factories** (3 min): make-deftest at ~380
5. **Workspace verify** (15-25 min): full cargo test
6. **SCORE produce** (5-10 min): Phase F readiness inventory

## What sonnet produces

- `wat/test.wat` modified (deftest + deftest-hermetic macro bodies + headers)
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md` with:
  - 6-row scorecard
  - Path A vs Path B choice for deftest-hermetic (likely Path A per V3 precedent)
  - Prelude semantic shift impact (any specific tests affected)
  - Updated documentation header wording for orchestrator review
  - Phase F readiness inventory (remaining run-sandboxed-* callers)
  - Honest deltas (≥ 3)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

Identical to V3 — abbreviated:
- Modify run-sandboxed-* substrate / run-ast wrappers / wat/kernel/hermetic.wat
- Modify ANY test call site
- Touch docs/arc/
- Commit / push / git add / git restore
- Use deferral language in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Add new substrate features
- Run hooks bypass / `--no-verify`
- Mass-modify call sites to fix failures (macro is supposed to be call-site compatible)

## Verification commands

```bash
# 1. Substrate ready (Gap E probes)
cargo test --release --test probe_do_splice_define 2>&1 | tail -3
cargo test --release --test probe_let_splice_define 2>&1 | tail -3

# 2. Baseline
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# 3. No run-sandboxed-ast in deftest expansion
grep -A 12 ":wat::test::deftest$" wat/test.wat | grep "run-sandboxed-ast"
# Expected: empty

# 4. Workspace post-rewrite
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2209 passed / 0 failed
```

## Expected workspace delta

- Baseline: 2209 passed / 0 failed (Gap E baseline)
- Post-V4: 2209 passed / 0 failed (macro signature unchanged; substrate path swap should be transparent)

## Honest delta categories (anticipated)

1. **Path A vs Path B for deftest-hermetic** — V3 sonnet chose Path A; validate or revisit with rationale
2. **Prelude semantic shift impact** — top-level expansion vs sandbox-internal; any specific tests where preludes need adjustment
3. **Factory composition** — `~~default-prelude` double-unquote still works through new outer shape
4. **TestResult typealias** — confirm equivalence to :wat::kernel::RunResult; return type consistency
5. **Phase F readiness** — list each remaining run-sandboxed-* caller with disposition
6. **Anything unexpected** — V3 surfaced Gap E; V4 ideally finds nothing new but be honest if a layer-deeper gap exists
