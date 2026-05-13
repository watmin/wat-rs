# Arc 170 slice 3 Gap K EXPECTATIONS — fix run-hermetic-driver

**One spawn.** Restructure `wat/test.wat:506-551` `:wat::test::run-hermetic-driver` so the lockstep nesting from SERVICE-PROGRAMS.md § Step 3 is satisfied: inner let owns + drains Receivers; outer let joins.

## Runtime band

**30-60 min.** Hard cap 120 min. Wakeup at T+7200s.

## Scorecard (8 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `run-hermetic-driver` body restructured: `Process/join-result` in OUTER let, Receivers from `Process/stdout`/`Process/stderr` in INNER let | grep + read |
| B | `ProcessJoinBeforeOutputDrain` does NOT fire on `wat/test.wat` after fix | grep cargo output |
| C1 | spawn-process lockstep probe (`tests/probe_run_hermetic_no_deadlock.rs`) PASSES — empty body + panic body on run-hermetic; verifies "lockstep restructure prevents the deadlock category on the spawn-process path" | cargo test |
| C2 | fork-program-ast stdout-capture probe (`tests/probe_run_hermetic_ast_stdout_capture.rs`) PASSES — child writes stdout; parent captures; probe NAME + BRIEF claim openly identify the fork-program-ast path | cargo test |
| C3 | **stdout-capture-on-spawn-process is OUT OF SCOPE** — spawn-process child does NOT install ThreadIO / ambient stdio (gap surfaced 2026-05-15; depends on slice 1F services landing on spawn-process). NO probe attempts to verify stdout-capture on the spawn-process path. SCORE explicitly states this scope cut. | grep + read SCORE |
| D | No wall-clock timeouts introduced anywhere (no `sleep`, no `set_*_timeout`, no arbitrary numbers) | grep + read |
| E | Workspace completes within `timeout -k 5 90 cargo test ...`; no orphans | full test run + ps |
| F | Other failures (V5 retry's Pattern A typealias / Pattern C exit-3) — if they still appear — fail FAST with clean diagnostics. The deadlock category is gone. | full test run |
| G | **Path-honesty audit** — every probe body exercises the SAME surface its file NAME + BRIEF CLAIM identify. No probe silently switches to an adjacent working path. | manual review + read each probe body against its filename |

**9 rows. All must PASS.**

### Row G rationale — the new discipline

The prior Gap K attempt (`66641d8`, reverted at `63cb747`) had a single `tests/probe_run_hermetic_drains_before_join.rs` file whose probe 3 silently switched to `run-hermetic-ast` (the fork-program-ast path with working stdio) to verify stdout capture. The file name + BRIEF claimed verification of the `run-hermetic` (spawn-process) path; the test body actually exercised a different surface. The detection went to 0; the probe passed; the bandaid shipped.

**That's a Honest failure (FM 9 applied to the load-bearing claim).** Row G is the explicit discipline: each probe body exercises the SAME path its name claims. No path-switching. If a property can't be verified on the named path (e.g., stdout-capture-on-spawn-process today), it goes in Row C3's out-of-scope list, NOT a probe with a misleading name.

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
