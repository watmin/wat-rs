# Arc 170 slice 3 Phase G-console EXPECTATIONS (sonnet scorecard)

**One spawn.** Mint `BareLegacyConsolePath` walker + sweep ~20 doc hits. Walker fires friendly migration diagnostic; users no longer cliff into cold UnknownFunction.

## Independent prediction

**Runtime band:** 60-90 min sonnet.

**Hard cap:** 180 min (2×).

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `BareLegacyConsolePath` variant + Display + Diagnostic in src/check.rs | grep + read confirms variant exists, Display teaches migration |
| B | Walker firing detects `:wat::console::*` source-level tokens (prefix-match) | probe `:wat::console::spawn` + `:wat::console::Console/out` both fire |
| C | Doc sweep complete — 12 Bucket A hits transformed across 6 files (USER-GUIDE: 11, CONVENTIONS: 3, CIRCUIT: 1, ZERO-MUTEX: 2, CLOJURE-ROSETTA: 2, WAT-CHEATSHEET: 1) | grep returns zero user-facing hits |
| D | `cargo check --release` green; workspace 0 failed | full test run |
| E | Probe diagnostic teaches the ambient kernel trio | manual probe + diagnostic text review |
| F | Final grep returns ZERO `:wat::console::` hits outside Bucket C scaffolding (variant + walker + Display) | grep |

**6 rows.** All must PASS.

## Implementation approach

1. **Walker mint** (15-20 min). Template clone from BareLegacyLambda; prefix-match `:wat::console::` instead of exact `:wat::core::lambda`.
2. **Doc sweep** (45-70 min). File-by-file, hit-by-hit, judgment-driven. Don't sed; rewrite per pedagogical intent. Use `examples/console-demo/wat/main.wat` as the canonical new-shape reference.
3. **Verification** (10 min). Workspace + probe + final grep.

## What sonnet produces

- `src/check.rs` modified (variant + Display + Diagnostic + walker firing)
- ~6 doc files modified (rewritten Console references)
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-G-CONSOLE.md` with:
  - 6-row scorecard verification
  - File-by-file inventory of hits transformed
  - Final diagnostic message wording for orchestrator review
  - Honest deltas (≥ 3)
  - Bucket C inventory (deliberately kept references)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Delete or modify any of the other 14 `BareLegacy*` variants
- Touch anything under `docs/arc/`
- Commit / push / git add / git restore
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Use deferral language anywhere in SCORE
- Add registry entry to special_forms.rs for `:wat::console::*` (lambda precedent says walker fires fatal; registry entry would let `(help :wat::console::*)` return something — wrong shape; let users see "unknown form" via help)
- Run hooks bypass / `--no-verify`
- Rewrite TIERS.md or arc 170 DESIGN docs (locked architecture)

## Verification commands

```bash
# 1. Workspace baseline
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# 2. Walker probe — verb form
echo '(:wat::console::spawn fn)' > /tmp/probe-console-1.wat
./target/release/wat /tmp/probe-console-1.wat 2>&1 | head -10

# 3. Walker probe — Console/out method form (catches the old service-method shape)
echo '(:wat::console::Console/out c "x")' > /tmp/probe-console-2.wat
./target/release/wat /tmp/probe-console-2.wat 2>&1 | head -10

# 4. Doc sweep verification
grep -rln "wat::console" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: only src/check.rs (variant + walker + Display)

# 5. Workspace post-sweep
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2205 passed / 0 failed (unchanged)
```

## Expected workspace delta

- Baseline: 2205 passed / 0 failed
- Post-Phase-G-console: 2205 passed / 0 failed (substrate addition + doc rewrites; no test-count change unless the walker firing surfaces existing source we didn't know about, in which case STOP and report)

## Honest delta categories (anticipated)

1. **Walker positioning + prefix-match shape** — design choices for the variant + walker; surface for review
2. **Diagnostic-message final wording** — the migration teaching text; surface for review before commit
3. **Doc rewrite judgment calls** — examples where the old Console-service-shape's teaching intent doesn't trivially map to ambient kernel verbs; surface specific cases
4. **Tier-list / namespace-list treatment** — does `:wat::console::*` get removed entirely or replaced with a pointer to ambient kernel I/O? Surface the choice
5. **Bucket C identifications** — what stays (substrate scaffolding teaching the legacy name) and what's gone (everything user-facing)
6. **Anything unexpected** — particularly any pre-existing source-level `:wat::console::*` use that the new walker surfaces (would mean slice 1f-η's sweep missed something)
