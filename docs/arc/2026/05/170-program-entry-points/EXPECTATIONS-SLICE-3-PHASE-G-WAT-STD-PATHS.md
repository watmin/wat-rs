# Arc 170 slice 3 Phase G-wat-std-paths EXPECTATIONS (sonnet scorecard)

**One spawn.** Last Phase 1 slice. ~38 wat/std/ hits + 3 fork-with-forms phantom + 1 README ASCII tree rewrite + COMPACTION-AMNESIA-RECOVERY.md careful triage.

## Independent prediction

**Runtime band:** 60-90 min sonnet.

**Hard cap:** 180 min (2×).

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | README.md ASCII tree (658-680) rewritten matching real disk truth | manual review; sonnet surfaces new tree in SCORE |
| B | All `fork-with-forms` hits in README replaced with `fork-program-ast` (3 hits at :98, :236, :238) | `grep -rn "fork-with-forms" .` returns zero outside docs/arc/ |
| C | `wat/std/` path lies transformed across 19 files; Bucket C historical preserved with rationale | per-file inventory in SCORE |
| D | `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 8 carefully updated; historical user-quote preserved | sonnet surfaces final wording for orchestrator review |
| E | `cargo check --release` green; workspace 2205 / 0 failed | full test run |
| F | Final grep returns ONLY Bucket C entries (historical context, each justified in SCORE) | grep |

**6 rows.** All must PASS.

## Implementation approach

1. **High-value rewrites first** (30-45 min):
   - README.md ASCII tree (658-680) — full rewrite matching real disk
   - README.md:501 "Every file under wat/std/" — reword
   - README.md fork-with-forms phantom (3 hits)
   - COMPACTION-AMNESIA-RECOVERY.md § FM 8 careful triage (4 hits)
2. **Path corrections sweep** (20-30 min): docs/USER-GUIDE, wat-tests/README, ZERO-MUTEX, docs/README, telemetry-sqlite/auto.rs, tests/*
3. **Substrate + wat-file Bucket triage** (15-20 min): src/check.rs (9) + src/types.rs (3) + src/stdlib.rs (3) + src/runtime.rs (2) + src/special_forms.rs (1) + src/freeze.rs (1) + src/sandbox.rs (1) + src/spawn.rs (1) + wat/test.wat (2) + wat/kernel/hermetic.wat (2) + wat/kernel/sandbox.wat (1)
4. **Verify** (5-10 min): workspace + grep

## What sonnet produces

- README.md heavily modified (ASCII tree + 501 + 98/236/238)
- docs/COMPACTION-AMNESIA-RECOVERY.md carefully modified
- ~17 other files modified (per-hit surgical)
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-G-WAT-STD-PATHS.md` with:
  - 6-row scorecard verification
  - Final README ASCII tree wording for orchestrator review
  - Final COMPACTION-AMNESIA-RECOVERY.md FM 8 wording for review
  - File-by-file Bucket classification (A/B/C with rationale)
  - Honest deltas (≥ 3)
  - Bonus catches beyond the audit's 13

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Touch anything under `docs/arc/` (FM 11 immutable)
- Commit / push / git add / git restore
- Use deferral language anywhere in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Erase the user-quote "remove wat/std/ast.wat — we are actively killing the std namespace..." historical context (FM 11 corollary; preserve verbatim)
- Touch `eval_kernel_wait_child` (deferred to Slice 4)
- Add walker-fires notes to fork-program docs (deferred to G-fork-program-walker-notes post-Slice-4)
- Add new walker / substrate features
- Modify arc 170 DESIGN docs or TIERS.md
- Run hooks bypass / `--no-verify`

## Verification commands

```bash
# 1. Workspace baseline
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# 2. wat/std/ final state
grep -rln "wat/std/" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: only Bucket C historical files; each justified in SCORE

# 3. fork-with-forms phantom verb
grep -rn "fork-with-forms" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: empty

# 4. wat-tests/std/ phantom directory
grep -rn "wat-tests/std/" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: empty

# 5. Workspace post-sweep
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2205 passed / 0 failed (unchanged)
```

## Expected workspace delta

- Baseline: 2205 passed / 0 failed
- Post-Phase-G-wat-std-paths: 2205 passed / 0 failed (pure text changes, no code modification)

## Honest delta categories (anticipated)

1. **README.md ASCII tree final wording** — highest stakes single edit; new tree read by every fresh developer. Surface for orchestrator review.
2. **COMPACTION-AMNESIA-RECOVERY.md § FM 8 rewording** — discipline doc; surface final phrasing for review. Balance: preserve FM 8 lesson + correct current reality + preserve historical user-quote.
3. **Substrate comment Bucket triage** — list each of the 21 substrate/wat-file hits with classification + rationale; surface judgment calls
4. **Self-referential wat-file comments** — files mentioning their own move; surface treatment per file
5. **Bonus catches beyond audit's 13** — already at 38; surface any additional discoveries
6. **Anything unexpected** — particularly any test breaking due to path-string changes (shouldn't happen but verify)
