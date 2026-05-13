# Arc 170 Slice B EXPECTATIONS — Crossbeam multiplex + SIGTERM wire-up

**One slice.** Wires Slice A's infrastructure into actual recv behavior. Closes the empirically-proven gap (`/tmp/shutdown_gap_proof.rs` 50-line Rust demo).

## Runtime band

**90-120 min sonnet.** Hard cap 240 min. ScheduleWakeup at T+3600s.

Bigger than Slice A because:
- crossbeam::select! introduction (new macro usage in substrate)
- Signal handler async-signal-safety verification (per signal-safety(7))
- New end-to-end probe with child-process isolation discipline

## Scorecard (10 rows) — see BRIEF for full criteria

| Row | What | Pass |
|-----|------|------|
| A | typed_recv Crossbeam multiplex via select! | read |
| B | typed_try_recv Crossbeam shutdown-first check | read |
| C | Shutdown arms map to Err(ThreadDiedError::Shutdown) at wat boundary | read |
| D | SIGTERM handler writes to wake pipe | read |
| E | SIGINT handler writes to wake pipe | read |
| F | Signal handlers async-signal-safe (signal-safety(7) compliance) | manual verify per manpage |
| G | cargo build --release --workspace passes | full build |
| H | cargo test 167/7 baseline (bimodal flake tolerable) | 3+ independent runs |
| I | probe_shutdown_cascade_crossbeam PASSES — child wakes on SIGTERM within 100ms | cargo test |
| J | ZERO-MUTEX compliance maintained | grep |

**10 rows. All must PASS.**

## Discipline mirror (orchestrator-side)

- FM 9: independent re-verification of Rows G, H, I, J before commit
- FM 12: `model: "sonnet"` explicit
- FM 16: no Bash/tool-availability preamble in BRIEF
- FM 17: pre-action sweep — verify async-signal-safety of every line added to SIGTERM/SIGINT handlers BEFORE commit; signal-safety(7) is the authoritative list
- Constraint pattern corrected per `feedback_brief_constraint_contradictions` — narrow surface; SCORE explicitly in scope, src/check.rs and src/spawn_process.rs explicitly out
- Atomic commit on success; ScheduleWakeup at T+3600s
- Reap orphans (`pkill -9 -f "target/release/deps/test-"`) before each cargo invocation

## Mode B trigger

- crossbeam::select! type-inference failure on cross-trait sender bounds — escalate
- Signal handler async-signal-unsafe operation surfaces during pre-commit sweep — STOP, do NOT commit, surface
- Probe Row I shows shutdown DOES NOT reach child within 100ms — deeper substrate gap; halt
- ZERO-MUTEX violation surfaces

## What this slice does NOT do

- Does NOT add PR_SET_PDEATHSIG (Slice C)
- Does NOT wire PipeFd recv to multiplex (Slice E)
- Does NOT include the stability-100 leak-count probe (Slice D)
- Does NOT modify spawn_process.rs (Slice C territory)

## What ships after this slice

After Slice B:
- Manually-sent SIGTERM/SIGINT cascade through crossbeam recvs cleanly
- The empirical gap from `/tmp/shutdown_gap_proof.rs` is structurally closed (at tier 1)
- Orphaned children still leak (PR_SET_PDEATHSIG is Slice C's job to deliver the signal)
- Tier-2 PipeFd recvs still block on shutdown (Slice E)

C+D+E remain mandatory to fully close the leak class.
