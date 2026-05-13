# Arc 170 slice 3 Gap K EXPECTATIONS — fix run-hermetic-driver

**One spawn.** Restructure `wat/test.wat:506-551` `:wat::test::run-hermetic-driver` so the lockstep nesting from SERVICE-PROGRAMS.md § Step 3 is satisfied: inner let owns + drains Receivers; outer let joins.

## Runtime band

**30-60 min.** Hard cap 120 min. Wakeup at T+7200s.

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `run-hermetic-driver` body restructured: `Process/join-result` in OUTER let, Receivers from `Process/stdout`/`Process/stderr` in INNER let | grep + read |
| B | `ProcessJoinBeforeOutputDrain` does NOT fire on `wat/test.wat` after fix | grep cargo output |
| C | New positive probe `tests/probe_run_hermetic_drains_before_join.rs` PASSES | cargo test |
| D | No wall-clock timeouts introduced anywhere (no `sleep`, no `set_*_timeout`, no arbitrary numbers) | grep + read |
| E | Workspace completes within `timeout -k 5 90 cargo test ...`; no orphans | full test run + ps |
| F | Other failures (V5 retry's Pattern A typealias / Pattern C exit-3) — if they still appear — fail FAST with clean diagnostics. The deadlock category is gone. | full test run |

**6 rows. All must PASS.**

## Discipline mirror (orchestrator-side)

- FM 9: baseline detection-fires-30-times verified pre-spawn
- FM 12: `model: "sonnet"` explicit
- FM 16: no Bash/tool-availability preamble in BRIEF
- Atomic-commit after scoring on branch `arc-170-gap-j-v5-deadlock-state`

## Hard constraints (mirror BRIEF)

- DO NOT modify `src/check.rs` — detection is committed verifier
- DO NOT add wall-clock timeouts anywhere
- DO NOT touch deftest, substrate primitives, docs/arc/
- DO NOT commit / push / git add
- DO USE `timeout -k 5 N` (N=30 probe / N=90 workspace)
- DO USE `pkill -9 -f "target/release/deps/test-"` if orphans appear; report in SCORE

## Mode B trigger

If after the restructure ProcessJoinBeforeOutputDrain STILL fires from `wat/test.wat`, STOP and report. Either the fix shape is wrong OR there's a SECOND offender we missed. Surface the diagnostic; don't ship until clean.
