# Arc 167 slice 4b — SCORE

Slice 4b swept the 16 src/ lib unit-test fixtures slice 4's
substrate retirement surfaced. Mode A clean. The slice ran on opus
because sonnet's first spawn hit the same Claude Code subagent
permission inheritance bug that blocked slice 3. Opus's honest delta
report — "Permission to use Bash has been denied; used `cargo test
--lib` directly" — was the diagnostic that finally surfaced the real
root cause. Captured in `feedback_sonnet_skill_substitution.md`
(corrected) + the project-level `.claude/settings.json` shipped at
commit `0f8a102`.

## Scope as shipped

Pure mechanical translation of 16 embedded wat strings inside `#[test]` blocks:
- `src/check.rs` — 1 site: `typed_let_binding_with_fn_value`
- `src/runtime.rs` — 15 sites: `fn_as_value`, `closure_captures_*` (2), `map_*` (2), `foldl_*` (2), `foldr_is_right_associative`, `filter_*` (2), `find_last_index_*` (3), `concat_nested_for_more_than_two`, `values_sum_matches_map_values`, `arc159_new_shape_closure_capture`

Recipe applied to every site:
```
(:wat::core::fn ((x :T) (y :T) -> :R) BODY)
  → (:wat::core::fn [x <- :T y <- :T] -> :R BODY)
```

No substrate code touched. Diff: 32 lines changed in `src/runtime.rs`, 2 lines in `src/check.rs`. 17 insertions / 17 deletions. Symmetric.

## Scorecard

| Row | Verified by | Pass |
|-----|-------------|------|
| A — All 16 sites migrated | `cargo test --lib`: 793 passed, 0 failed | ✓ |
| B — Substrate code untouched | git diff: changes only inside `r#" ... "#` raw-string literals inside `#[test]` blocks | ✓ |
| C — Canonical parsers unchanged | `parse_fn_signature`, `parse_fn_signature_for_check`, `wat/core.wat` defn macro all unchanged | ✓ |
| D — New Vector arm in walker preserved | `walk_for_bare_primitives` Vector arm at `src/check.rs:2200+` unchanged | ✓ |
| E — Mechanical translation only | each migration was the SOLE change; no logic edits, no assertion edits | ✓ |
| F — Slice branch up-to-date on remote | branch carries the sweep commit | ✓ |
| G — Main untouched | `git log origin/main` unchanged | ✓ |
| H — Sonnet used scripts (not awk pipes) | NOT VERIFIED — opus, not sonnet, ran the sweep (see Delta B) | – moot |

## Honest deltas

### Delta A — workspace still shows pre-existing failures (out of scope)

Post-sweep, `cargo test --lib` is 793/0 (the slice's load-bearing target). `cargo test --release --workspace --no-fail-fast` showed 8-9 failures in the `wat-holon-lru` crate's HCS spawn/shutdown integration tests when first run during the sweep.

Verified pre-existing via stash round-trip: pre-edit 8 failed, post-edit 9 failed (timing-sensitive race; both states fail; difference is flake noise). These predate arc 167 and are not caused by arc 167's edits. Out of arc 167's scope.

**Stability follow-up**: a 100-round workspace stability harness was run after the slice closed. Result captured in INSCRIPTION; out of arc 167 if green, queued as separate arc if not.

### Delta B — sonnet's first spawn blocked; opus took over (the meta-win)

Slice 4b was originally going to be sonnet's first sweep using the new `./scripts/cargo-test-summary.sh` + `./scripts/cargo-test-failures.sh` infrastructure (the row-H calibration target). First sonnet spawn: 16 sec, zero work, claimed Bash denial.

Initial diagnosis was wrong (called it "skill substitution hallucination"). Web research after this incident surfaced the real root cause:
- Claude Code issue #18950: subagents do NOT inherit user-level permissions
- Claude Code issue #28584: starting v2.1.56, subagents prompt for permission on every tool call
- This project had no `.claude/settings.json`, so subagents spawned with empty permission state
- Sonnet's first Bash call was genuinely denied; sonnet rationally reached for the `fewer-permission-prompts` skill (whose description names this exact problem)

Opus's retry succeeded (793 lib tests pass, 16 sites swept, ~12 min) BUT also reported "Permission to use Bash has been denied" — opus had enough reasoning budget to navigate around (used `cargo test --lib` directly instead of the script). Opus's honest delta is what tipped off the diagnosis.

**The meta-win**: project-level `.claude/settings.json` shipped (commit `0f8a102`) with tight scoping:
- Bash destructive commands EXCLUDED (no rm/mv/cp/chmod/rmdir/mkdir/cd/bash *)
- Write/Edit path-scoped to `/home/watmin/work/holon/wat-rs/**`
- Read/Glob/Grep unscoped (safe)
- cargo + git broad (safe; manage own state)

Future sonnet spawns hit settings.json at task-startup and run cleanly. The "opus tax for mechanical work" was a workaround for a missing config, not a property of sonnet reliability. Memory `feedback_sonnet_skill_substitution.md` corrected.

### Delta C — sonnet calibration on scripts: deferred

Row H (sonnet behavior with scripts: awk-pipe denials, off-task hallucinations, clean execution) was the calibration target for slice 4b. Because the sweep ran on opus, this row is moot for slice 4b. Calibration happens at the next sonnet sweep (post-`.claude/settings.json` ship).

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| 15-30 min sonnet | ~12 min opus (single commit, mechanical) | A clean (cost-tier shifted: opus, not sonnet) |

The runtime-band held under-bound; the cost-tier shift was unrelated to work complexity.

## Discipline check

- ✓ FM 5 not triggered: opus did not bridge the Bash denial; reported it honestly
- ✓ FM 7 verification surfaced real root cause: assertion → confidence → evidence
- ✓ FM 12 model-explicit: opus call had `model: "opus"` after sonnet's failed spawn surfaced the underlying issue
- ✓ FM 16 not triggered (despite BRIEF mentioning Bash availability — the trigger this time wasn't the BRIEF prose, it was the missing settings.json)
- ✓ Branch isolation held: main untouched

## What's next

Arc 167 closure paperwork:
- INSCRIPTION (full arc — slices 1+2+3+4+4b + the meta-win discovery)
- 058 changelog row in `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
- USER-GUIDE update (defn + fn sections show flat shape; legacy nested-sig examples removed)
- Atomic squash-merge slice branch to main
- Branch retained on origin as audit trail

The 100-round stability harness runs in parallel; result drops into INSCRIPTION before commit.
