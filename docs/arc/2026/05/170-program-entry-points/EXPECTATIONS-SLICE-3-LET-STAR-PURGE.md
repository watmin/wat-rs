# Arc 170 slice 3 — let* purge EXPECTATIONS (sonnet scorecard)

**One spawn.** Substrate housekeeping + ~170-hit textual sweep across ~10 files. Walker stays armed; user-facing residue gone.

## Independent prediction

**Runtime band:** 50-80 min sonnet.

**Hard cap:** 160 min.

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | Substrate housekeeping: registry entry removed (`special_forms.rs:147`); stale "fall-through" comments updated (`check.rs:1636-1665` + 5 `runtime.rs` sites) | grep + read |
| B | Documentation sweep: 8 files (`docs/USER-GUIDE.md`, `docs/SERVICE-PROGRAMS.md`, `docs/WAT-CHEATSHEET.md`, `docs/CIRCUIT.md`, `docs/CONVENTIONS.md`, `docs/CLOJURE-ROSETTA.md`, `docs/INTENTIONS.md`, `README.md`); ~89 hits transformed | grep |
| C | Wat source sweep: 3 files (`wat/kernel/services/{stdin,stdout,stderr}.wat`); 6 hits transformed | grep |
| D | Spell sweep: 2 files (`.claude/skills/complectens/SKILL.md`, `.claude/skills/vocare/SKILL.md`); ~19 hits transformed | grep |
| E | Test file judgment-call review: `tests/wat_arc136_do_form.rs` + `tests/wat_arc155_fn_rename.rs` — comments transformed, fixtures preserved | manual |
| F | Verification: cargo test --workspace --no-fail-fast at 2205 passed / 0 failed; final grep returns ZERO let* hits outside Bucket C (`docs/arc/`, `tests/wat_arc154_kill_let_star.rs`, historical retirement context comments) | full verification |

**6 rows.** All must PASS.

## Implementation approach

1. **Substrate first** (5-10 min). Three small mechanical edits in src/. Verify workspace stays green after.
2. **Bulk doc sweep** (30-45 min). Read each file; transform `let*` → `let` per Bucket B. Check each prose context still reads correctly.
3. **Wat source + spell sweep** (10 min). Both are simple comment text.
4. **Test file judgment** (5-10 min). Review the two test files; preserve fixtures.
5. **Verification** (10 min). Full workspace test + grep + probe.

## What sonnet produces

- Modified files across substrate + docs + wat + skills + tests
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-LET-STAR-PURGE.md` with:
  - Scorecard verification
  - File-by-file change inventory (number of hits transformed, any judgment calls)
  - Honest deltas (≥ 3)
  - Final state of substrate cleanup + Bucket C inventory (what was deliberately kept and why)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Delete `BareLegacyLetStar` variant / Display / Diagnostic / walker firing
- Delete `tests/wat_arc154_kill_let_star.rs`
- Touch anything under `docs/arc/`
- Commit / push / git add / git restore
- Run hooks bypass / `--no-verify` / etc.
- Operate in any directory other than `/home/watmin/work/holon/wat-rs/`
- Use deferral language anywhere in SCORE
- Ship Path B or Path C from the prior discussion — Path A only (lambda precedent symmetry)

## Verification commands

```bash
# 1. Workspace baseline (before sweep)
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# 2. Final state: grep returns ONLY Bucket C
grep -rln "let\*" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/" | grep -v "tests/wat_arc154_kill_let_star.rs"
# Expected: empty, OR only files listed with all matches confirmed as historical context comments (Bucket C) — list these in SCORE

# 3. Probe: let* still fatal at check
echo '(:wat::core::let* [x 1] x)' > /tmp/probe-let-star.wat
target/release/wat /tmp/probe-let-star.wat 2>&1 | head -5
# Expected: BareLegacyLetStar fires with friendly diagnostic

# 4. Workspace post-sweep
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2205 passed / 0 failed (unchanged)
```

## Expected workspace delta

- Baseline: 2205 passed / 0 failed
- Post-sweep: 2205 passed / 0 failed (no test-count change; pure textual sweep + small substrate housekeeping)

## Honest delta categories (anticipated)

1. **Judgment calls per hit** — any prose where `let*` text isn't mechanically replaceable; surface specific cases
2. **Bucket C identifications** — text intentionally kept (historical retirement record); list every file + line if not in the standard exclusion set
3. **Substrate comment rewriting** — the new wording for the previously-stale "fall-through" comments; surface for review
4. **Test fixture review outcomes** — what stayed, what transformed, in the two test files
5. **Workspace impact** — any unexpected test interaction (shouldn't happen with pure textual changes)
6. **Anything unexpected** during sweep
