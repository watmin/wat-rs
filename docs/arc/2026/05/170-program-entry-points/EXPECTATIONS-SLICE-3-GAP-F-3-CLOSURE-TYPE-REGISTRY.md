# Arc 170 slice 3 Gap F-3 EXPECTATIONS (sonnet scorecard)

**One spawn.** Self-contained substrate edit: `extract_closure` propagates parent's type registry to spawn-process child. Hermetic semantics preserved.

## Independent prediction

**Runtime band:** 30-60 min sonnet.

**Hard cap:** 120 min (2×).

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `extract_closure` includes parent's type registry in produced ClosurePackage | grep + read |
| B | ClosurePackage (or equiv) carries `Arc<TypeEnv>` (or equiv) | grep + read |
| C | Child startup uses inherited types before user code runs | grep + read |
| D | 3 new probes pass: struct / enum / parametric | cargo test |
| E | All 10+ existing Gap C V2 / D / E / F-1 probes still pass | cargo test |
| F | Hermetic isolation semantics preserved (existing fork-program/spawn-process tests unchanged) | full workspace test |

**6 rows.** All must PASS.

## Implementation approach

1. **Locate + audit** (5 min): grep `extract_closure` / `ClosurePackage`; identify current capture surface
2. **Probes** (15 min): 3 new probes confirming failure baseline
3. **Extend `extract_closure`** (10-15 min): add type-registry field; populate
4. **Update child startup** (5-10 min): use inherited types
5. **Verify** (10 min): probes + workspace + hermetic-suite regression check

## What sonnet produces

- `src/closure.rs` (or `src/spawn.rs`) modified
- ClosurePackage struct extended (one field)
- Child startup code updated to use inherited types
- 3 new probe tests (or 1 combined file)
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-F-3-CLOSURE-TYPE-REGISTRY.md` with:
  - 6-row scorecard
  - Inclusion strategy rationale (whole-registry vs filtered)
  - TypeEnv-compatibility verification
  - Hermetic regression check result
  - Honest deltas (≥ 3)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Modify TypeEnv's internal representation
- Add new type-system features
- Touch `docs/arc/`
- Commit / push / git add / git restore
- Use deferral language in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Extend to Gap F-1 / F-2 / G scope
- Use --no-verify or skip hooks
- Regress hermetic isolation

## Verification commands

```bash
# New F-3 probes
cargo test --release --test probe_spawn_process_parent_type 2>&1 | tail -5
# (Or whatever probe file structure sonnet chose)

# Regression: existing substrate probes + hermetic suite
cargo test --release --test probe_do_splice_def 2>&1 | tail -3
cargo test --release --test probe_let_splice_def 2>&1 | tail -3
cargo test --release --test probe_do_splice_define 2>&1 | tail -3
cargo test --release --test probe_let_splice_define 2>&1 | tail -3
# Plus F-1 probes (added by predecessor)
cargo test --release --test probe_do_splice_struct 2>&1 | tail -3 # or similar
# Plus fork-program / spawn-process integration tests
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline (post-F-1): 2209 + N passed / 0 failed
- Post-F-3: 2209 + N + 3 passed / 0 failed

## Honest delta categories (anticipated)

1. **Inclusion strategy** — whole-registry vs filtered. Surface choice + rationale.
2. **TypeEnv shape compatibility** — does it Arc-share cleanly?
3. **Hermetic regression** — any existing test depends on child NOT seeing parent types?
4. **Parametric type edge cases** — `:test::Wrapper<E>` and similar
5. **Anything unexpected** — closure-extraction edge cases
