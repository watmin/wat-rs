# Arc 170 slice 3 Gap G EXPECTATIONS (sonnet scorecard)

**One spawn.** `:wat::test::deftest-hermetic` macro body Path E rewrite + isolation enforcement probes.

## Independent prediction

**Runtime band:** 45-90 min sonnet.

**Hard cap:** 180 min (2×).

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::deftest-hermetic` body emits Path E shape: `(define (~name -> RunResult) (run-hermetic (do ~@prelude ~body)))` | grep + read |
| B | `:wat::test::make-deftest-hermetic` factory follows Path E | grep + read |
| C | Documentation header explicitly names "strict isolation" contract | manual review |
| D | 4+ enforcement probes pass (parent symbol-table sealed; cross-test prelude isolation; child can't reach parent's runtime; test fn visible in parent but not prelude content) | cargo test |
| E | Workspace at baseline + new probes / 0 failed | full test |
| F | Existing deftest-hermetic users (service-template, ambient-stdio, roundtrip) work correctly under Path E | full test |

**6 rows.** All must PASS.

## Implementation approach

1. **Rewrite deftest-hermetic body** (5-10 min): one macro change in wat/test.wat
2. **Rewrite make-deftest-hermetic factory** (5 min): mirror
3. **Documentation header** (5-10 min): describe strict isolation contract
4. **Enforcement probes** (20-30 min): 4 probes proving strict isolation properties
5. **Verify** (15-25 min): workspace + regression check

## What sonnet produces

- `wat/test.wat` modified (deftest-hermetic body + make-deftest-hermetic factory + headers)
- `tests/probe_deftest_hermetic_isolation.rs` (or equivalent)
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-G-PATH-E-ISOLATION.md` with:
  - 6-row scorecard
  - Probe design rationale (what each probe demonstrates)
  - Final documentation header wording for review
  - Honest deltas (≥ 3)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Modify `:wat::test::deftest` macro body (Phase E V5)
- Modify run-hermetic / run-hermetic-driver substrate
- Modify ANY test call site outside the new probe file
- Touch `docs/arc/`
- Commit / push / git add / git restore
- Use deferral language in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Use --no-verify or skip hooks

## Verification commands

```bash
# New G probes
cargo test --release --test probe_deftest_hermetic_isolation 2>&1 | tail -5

# Regression: all prior substrate probes
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# Existing deftest-hermetic callers
cargo test --release --test deftest_hermetic_integration 2>&1 | tail -5 # or similar; sonnet locates
```

## Expected workspace delta

- Baseline (post-F-1+F-3+F-2): 2209 + N + N' + N'' / 0 failed
- Post-G: + 4 / 0 failed

## Honest delta categories (anticipated)

1. **Probe design** — what specifically demonstrates "parent's frozen world unchanged"; the assertion shapes
2. **Cross-test prelude collision probe** — does Path E's promise hold under empirical test?
3. **make-deftest-hermetic factory composition** — `~~default-prelude` double-unquote through the new outer shape
4. **Documentation header wording** — final for review
5. **Anything unexpected** — particularly any existing deftest-hermetic user revealing an assumption that Path E breaks
