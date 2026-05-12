# Arc 170 slice 3 Phase G-stream EXPECTATIONS (sonnet scorecard)

**One spawn.** Pure Bucket B doc sweep — no substrate work. Walker already fires; this slice drains the textual residue teaching the old namespace.

## Independent prediction

**Runtime band:** 30-50 min sonnet.

**Hard cap:** 100 min (2×).

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `docs/USER-GUIDE.md` sweep complete — 20 hits (tier-4 list ~586, streaming section ~2050-2150, reference table ~3487-3495) | grep |
| B | `docs/CONVENTIONS.md` typealias table corrected — 3 rows; namespace + Stream<T> inner type + file path all fixed | manual review |
| C | `wat-scripts/README.md` + `README.md` sweep complete (1 hit each) | grep |
| D | `cargo check --release` green; workspace 2205 / 0 failed | full test run |
| E | Probe: `(:wat::std::stream::map x y)` fires BareLegacyStreamPath with `:wat::stream::*` canonical | manual probe |
| F | Final grep returns ONLY src/check.rs (Bucket D scaffolding) + docs/SUBSTRATE-AS-TEACHER.md (Bucket C historical) | grep |

**6 rows.** All must PASS.

## Implementation approach

1. **Namespace sweep** (15-20 min). Mechanical 1:1 `:wat::std::stream::*` → `:wat::stream::*` across 4 user-facing files. Per-hit judgment: is this teaching current usage (transform) or recording the migration (keep — Bucket C)?
2. **CONVENTIONS.md table** (10-15 min). Three rows, three corrections each. Verify against `wat/stream.wat:40-90` typealias source.
3. **USER-GUIDE.md tier-4 restructure** (5-10 min). Reflect stream's graduation out of `:wat::std::*`.
4. **Verification** (5-10 min). Workspace + probe + grep.

## What sonnet produces

- ~4 doc files modified (sweep)
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-G-STREAM.md` with:
  - 6-row scorecard verification
  - Final CONVENTIONS.md typealias table wording for orchestrator review
  - Final USER-GUIDE.md tier-4 wording for review
  - File-by-file hits transformed
  - Honest deltas (≥ 3)
  - Bucket C inventory (what stays and why)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Touch `src/check.rs` BareLegacyStreamPath scaffolding (Bucket D — keep)
- Touch `docs/SUBSTRATE-AS-TEACHER.md:225` (Bucket C — keep)
- Touch anything under `docs/arc/`
- Commit / push / git add / git restore
- Use deferral language anywhere in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Broaden scope to other `wat/std/` paths beyond the typealias-table 3 rows
- Add new walker / substrate features
- Add registry entry to special_forms.rs for `:wat::std::stream::*`
- Run hooks bypass / `--no-verify`
- Touch ~/.claude/ memory system

## Verification commands

```bash
# 1. Workspace baseline
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# 2. Walker probe
echo '(:wat::std::stream::map x y)' > /tmp/probe-stream.wat
./target/release/wat /tmp/probe-stream.wat 2>&1 | head -10
# Expected: BareLegacyStreamPath with :wat::stream::* canonical teaching

# 3. Final grep
grep -rln "wat::std::stream" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: only src/check.rs (Bucket D) + docs/SUBSTRATE-AS-TEACHER.md (Bucket C)

# 4. Workspace post-sweep
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2205 passed / 0 failed (unchanged)
```

## Expected workspace delta

- Baseline: 2205 passed / 0 failed
- Post-Phase-G-stream: 2205 passed / 0 failed (pure textual sweep; no test-count change)

## Honest delta categories (anticipated)

1. **CONVENTIONS.md typealias rewording** — surface final 3 rows for review (especially Stream<T> inner type: verify `Thread<wat::core::nil, wat::core::nil>` vs alternative canonical forms against `wat/stream.wat:49`)
2. **USER-GUIDE.md tier-list restructure** — propose new tier-4 wording; most judgment-heavy edit. Reflects that stream graduated out of `:wat::std::*` to its own top-level tier
3. **Bucket C identifications** — any prose where transformation would erase legitimate historical context (paralleling `docs/SUBSTRATE-AS-TEACHER.md:225` pattern)
4. **Bonus catches** — if sweeping surfaces any related hits beyond the 25 audited (e.g., `wat-tests/`, additional `wat-scripts/`), surface them
5. **Workspace impact** — should be zero; report anything unexpected
6. **Anything pre-existing source-level `:wat::std::stream::*`** in workspace — would mean slice 9d's sweep missed something; STOP and report
