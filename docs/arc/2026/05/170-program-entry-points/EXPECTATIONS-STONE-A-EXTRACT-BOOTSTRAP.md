# Arc 170 RUNTIME-BOOTSTRAP-BACKLOG Stone A EXPECTATIONS

**One slice.** Extract `bootstrap_wat_vm_process` from `invoke_user_main_orchestrated`. No behavior change.

## Runtime band

**60-90 min sonnet.** Hard cap 180 min (clamped to 3600s by ScheduleWakeup runtime).

## Scorecard (8 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `pub(crate)` (or `pub`) `bootstrap_wat_vm_process(BootstrapArgs) -> Result<ProcessRuntime, RuntimeError>` exists | grep + read |
| B | `BootstrapArgs` + `ProcessRuntime` types defined; `ProcessRuntime` has `.symbols()` + `Drop` impl | grep + read |
| C | `invoke_user_main_orchestrated` refactored to delegate; inline bootstrap (lines 757-810) + cleanup (827-849) gone from this function | grep + read |
| D | Drop order in `ProcessRuntime` matches existing cleanup exactly (deregister → uninstall → drop sym → drop services → join stdin → join stdout → join stderr) | read |
| E | Baseline workspace: 167 pass / 7 fail / ~1s; detection 0 | cargo test |
| F | All 7 existing probes PASS (no regression) | cargo test |
| G | New `probe_bootstrap_wat_vm_process` PASSES — verifies helper callable, services accessible, ThreadIO installed, cleanup runs on Drop | cargo test |
| H | No edits outside the documented surface (no spawn_process.rs / fork.rs / check.rs / wat-side files / docs/arc/ / ~/.claude/) | git diff |

**8 rows. All must PASS.**

## Discipline mirror (orchestrator-side)

- FM 9: independent re-run of baseline + each probe before scoring
- FM 12: `model: "sonnet"` explicit
- FM 16: no Bash/tool-availability preamble in BRIEF (none added)
- FM 17: pre-action sweep before commit — verify drop order matches (Row D); workspace baseline holds (Row E)
- Atomic commit on success
- ScheduleWakeup at T+3600s (runtime cap; predicted upper-bound = 90 min so wake fires near upper-bound; another reschedule if needed)

## Mode B trigger

- If refactor cannot preserve behavior exactly — STOP and report. Don't try to fix incidental issues; surface them.
- If Drop order can't match current cleanup precisely (some edge requires reordering) — STOP and report. Drop order is load-bearing.
- If RuntimeError doesn't have a clean variant for bootstrap failures and existing variants don't fit — surface in honest delta; mint a new variant only if absolutely needed (otherwise reuse).

## Hard constraints (mirror BRIEF)

- DO NOT modify `src/check.rs` / `src/spawn_process.rs` / `src/fork.rs`
- DO NOT add wall-clock timeouts
- DO NOT touch `docs/arc/` or `~/.claude/`
- DO NOT use `cd <subdir> && ...` — use absolute paths or `git -C`
- DO NOT commit / push / git add
- DO use `timeout -k 5 N` (N=30 probe, N=90 workspace)
- DO use `pkill -9 -f "target/release/deps/test-"` for orphans; report in SCORE
