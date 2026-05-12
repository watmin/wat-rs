# Arc 170 slice 3 Phase G-lambda-docstrings EXPECTATIONS (sonnet scorecard)

**One spawn.** Fix 2 substrate docstring lies + sweep ~20-30 doc hits. Walker stays armed; user-facing residue gone; future implementers no longer misled.

## Independent prediction

**Runtime band:** 30-50 min sonnet.

**Hard cap:** 100 min (2×).

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `src/runtime.rs` eval_fn docstring rewritten (no false "lambda routes here" claim) | grep + read; new wording surfaced for review |
| B | `src/check.rs` infer_fn docstring rewritten (no parallel false claim) | grep + read; new wording surfaced |
| C | Doc prose sweep complete across 6-9 files; ~20-30 hits | grep |
| D | `docs/USER-GUIDE.md:2716` fn rendering claim corrected to actual `<fn@{}>` per src/runtime.rs:14532 | manual review |
| E | `cargo check --release` green; workspace 2205 / 0 failed | full test run |
| F | Final grep returns ONLY Bucket C (historical context) + Bucket D scaffolding (variant + Display + walker + tests) | grep |

**6 rows.** All must PASS.

## Implementation approach

1. **Substrate docstrings** (10-15 min). Two surgical edits. Surface final wording in SCORE for review.
2. **Doc prose sweep** (20-30 min). File-by-file, hit-by-hit, judgment-driven. Mix of Bucket A (code → transform), Bucket B (concept rename), Bucket C (historical KEEP).
3. **Verification** (5-10 min). Workspace + probe + grep.

## What sonnet produces

- `src/runtime.rs` + `src/check.rs` modified (docstrings only — no code changes)
- ~6-9 doc files modified
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-G-LAMBDA-DOCSTRINGS.md` with:
  - 6-row scorecard verification
  - Final eval_fn + infer_fn docstring wording for orchestrator review
  - USER-GUIDE.md:2716 corrected sentence for review
  - USER-GUIDE.md:3236 reference table judgment (remove row vs. replace with fn vs. retired section)
  - File-by-file hits transformed
  - Bucket C inventory (historical entries kept)
  - Honest deltas (≥ 3)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Delete `BareLegacyLambda` variant / Display / Diagnostic / walker firing
- Delete `tests/wat_arc144_special_forms.rs`
- Touch anything under `docs/arc/`
- Commit / push / git add / git restore
- Use deferral language anywhere in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Add new walker / substrate features
- Run hooks bypass / `--no-verify`
- Modify any actual code in eval_fn / infer_fn (docstrings ONLY)
- Erase historical context comments (Bucket C: USER-GUIDE:803, :809, etc.)

## Verification commands

```bash
# 1. Workspace baseline
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# 2. Walker probe
echo '(:wat::core::lambda [x] x)' > /tmp/probe-lambda.wat
./target/release/wat /tmp/probe-lambda.wat 2>&1 | head -5
# Expected: BareLegacyLambda with :wat::core::fn canonical

# 3. Final grep
grep -rln ":wat::core::lambda\|lambda@\|wat__core__lambda\|eval_lambda\|infer_lambda" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: only Bucket C/D files

# 4. fn rendering truth check
grep -n "fn@" src/runtime.rs | head -3
# Confirms `<fn@{}>` is the actual format

# 5. Workspace post-sweep
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline: 2205 passed / 0 failed
- Post-Phase-G-lambda-docstrings: 2205 passed / 0 failed (pure docstring + doc text changes)

## Honest delta categories (anticipated)

1. **Substrate docstring final wording** — surface eval_fn + infer_fn rewrites for orchestrator review
2. **USER-GUIDE.md:3236 reference table judgment** — remove row / replace with fn / move to retired section — surface choice
3. **USER-GUIDE.md:2716 fn rendering correction** — final sentence wording matching actual debug format
4. **Bucket C inventory** — list each historical-context hit kept and why
5. **`docs/COMPACTION-AMNESIA-RECOVERY.md` triage** — careful per-hit classification (our own discipline doc)
6. **Bonus catches** — additional files / lines beyond the audit's 9 (expected per pattern; surface)
7. **Anything unexpected** — particularly any pre-existing source-level `:wat::core::lambda` in workspace
