# Arc 170 Slice A EXPECTATIONS — shutdown-aware infra

**One slice.** Pure additive. No callers wired. Closes the substrate-level deferral on shutdown-cascade infrastructure.

## Runtime band

**60-90 min sonnet.** Hard cap 180 min (2×; bigger than expected for the additional surface — ThreadDiedError variant ripples through Display/EDN/scheme). Wakeup at T+3600s (runtime cap).

## Scorecard (10 rows) — see BRIEF for full row criteria

| Row | What | Pass |
|-----|------|------|
| A | RecvOutcome::Shutdown variant | grep + read |
| B | SHUTDOWN_RX/_TX_PTR/_WAKE_WRITE_FD statics | grep + read |
| C | init_shutdown_signal() + worker thread spawn | grep + read |
| D | trigger_shutdown() + ZERO-MUTEX | grep + read |
| E | ThreadDiedError::Shutdown + Display/EDN/scheme consistent | cargo test + grep |
| F | init called in bootstrap before trio | read order |
| G | cargo build --release --workspace passes | full build |
| H | cargo test 167/7 baseline UNCHANGED | independent re-run |
| I | No worker thread leaks (worker exits on trigger) | pgrep |
| J | ZERO-MUTEX compliance | grep verification |

**10 rows. All must PASS.**

## Discipline mirror (orchestrator-side)

- FM 9: independent re-verification of Rows G, H, I, J before commit
- FM 12: `model: "sonnet"` explicit
- FM 16: no Bash/tool-availability preamble in BRIEF
- FM 17: pre-action sweep — verify each ThreadDiedError variant site (Display, EDN encode, type-check scheme, to-failure accessor) consistent before commit
- Constraint pattern corrected per `feedback_brief_constraint_contradictions` — narrow surface; SCORE doc explicitly in scope, src/check.rs and src/spawn_process.rs explicitly out
- Atomic commit on success; ScheduleWakeup at T+3600s
- Reap orphans (`pkill -9 -f "target/release/deps/test-"`) before each cargo invocation

## Mode B trigger

- ZERO-MUTEX violation surfaces during ThreadDiedError variant ripple (Display/EDN/scheme)
- Cargo test baseline shifts from 167/7 — Slice A is supposed to be purely additive
- Worker thread spawn fails or creates leak — substrate is in unexpected state

## What this slice does NOT do

- Does NOT wire typed_recv to use SHUTDOWN_RX (Slice B)
- Does NOT change SIGTERM/SIGINT signal handlers (Slice B)
- Does NOT add PR_SET_PDEATHSIG (Slice C)
- Does NOT include the end-to-end probe (Slice D)
- Does NOT touch PipeFd-receiver shutdown awareness (Slice E)

Each is its own slice with its own BRIEF.
