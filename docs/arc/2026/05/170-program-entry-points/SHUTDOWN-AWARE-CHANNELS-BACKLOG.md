# Arc 170 — Shutdown-aware channels backlog (sequenced slices)

**Created:** 2026-05-13. Discipline per `feedback_iterative_complexity`: prove each stepping stone, no one-shotting.
**Design:** `DESIGN-SHUTDOWN-AWARE-CHANNELS.md` (committed `ef01463`).
**Empirical proof of gap:** standalone Rust 50 lines; SIGTERM does NOT wake blocked crossbeam recv.

## Why a backlog (not one BRIEF)

The shutdown-aware-channels architecture is 4-5 substrate concerns stitched together. One-shotting risks:
- Half-baked atomic+OnceLock+heap-Box semantics
- Cross-file edit conflicts
- Tests that pass for the wrong reason
- Hidden coupling we don't surface until the whole thing's in

Each slice is independently verifiable. Each gates on the prior. Sonnet ships one; orchestrator verifies; atomic commit; next slice briefs against fresh substrate state.

## The five slices

| # | Slice | What ships | Independently verifiable? | Size | Blocks |
|---|---|---|---|---|---|
| **A** | **Infrastructure — globals, worker, variant.** Add `SHUTDOWN_TX` (`AtomicPtr<Box<crossbeam::Sender<()>>>`) + `SHUTDOWN_RX` (`OnceLock<crossbeam::Receiver<()>>`) + `SHUTDOWN_WAKE_FD` (`AtomicI32`). Add `init_shutdown_signal()` called at bootstrap. Add shutdown-worker thread that blocks on wake-pipe + calls `trigger_shutdown()` (atomic swap to null + Box::drop). Add `RecvOutcome::Shutdown` variant (no callers yet). Add `ThreadDiedError::Shutdown` wat-level variant. | YES — pure additive. cargo build, cargo test must show 167/7 baseline. No behavior change. | S | B, C |
| **B** | **Crossbeam multiplex.** Modify `typed_recv` Crossbeam arm to use `crossbeam::select!` between data_rx and SHUTDOWN_RX. On shutdown fire → return `RecvOutcome::Shutdown`. Wire `eval_kernel_recv` to map `Shutdown` → `Err(ThreadDiedError::Shutdown)` at wat boundary. Wire SIGTERM/SIGINT handler in `fork.rs` to write to wake-pipe (async-signal-safe). | YES — probe: raise SIGTERM mid-blocked-recv; assert recv returns Err Shutdown within 100ms. | M | C |
| **C** | **PR_SET_PDEATHSIG in child fork branches.** Add `libc::prctl(PR_SET_PDEATHSIG, SIGTERM, ...)` in `src/spawn_process.rs::spawn_process_child_branch` (after `setpgid`) and `src/fork.rs` (after `setpgid`). | YES — probe: parent-forks-child; parent immediately exits; verify child receives SIGTERM and exits within 1s (no leak). | S | D |
| **D** | **End-to-end probe + stability verification.** Test deftest that demonstrates the full cascade: spawn child via spawn-process → orphan it (parent dies) → child detects parent death → cascade fires → child exits cleanly. Run `scripts/stability-100.sh 20`; verify zero leaked processes accumulate. | YES — empirical leak count goes from "12 per run" to 0. | M | (terminal for the leak class) |
| **E** | **PipeFd multiplex.** Tier-2 from-pipe Receivers also multiplex on shutdown. Implementation: OS-level select via `epoll`/`poll(2)` on (pipe_fd, shutdown_eventfd). The substrate gains a shutdown eventfd alongside the wake-pipe; PipeFd recv selects on both. Closes the tier-2 leak path so the architecture is uniform across tiers (per TIERS.md uniformity claim). | YES — probe: from-pipe wrapped Process/stdout recv wakes on shutdown within 100ms. | M-L | (independent of A-D, parallelizable with C/D) |

**Critical path:** A → B → C → D → E. Total: 5 ship cycles.

Slice E is mandatory, not deferred. Per `feedback_no_known_defect_left_unfixed`: known defect surfaceable now → ship now. Per `feedback_pivot_not_defer`: deferral bias is a STOP signal that the thing actually needs doing. PipeFd shutdown awareness is required for TIERS.md's uniformity claim to hold honestly at tier 2.

## What each slice protects against (failure-engineering)

- **Slice A** alone: cannot regress — purely additive, no callers wired.
- **Slice B** alone: closes crossbeam-blocked-recv leak class. Even without C, manually-sent SIGTERM/SIGINT can shut services down cleanly.
- **Slice C** alone (without B): less useful — kernel delivers SIGTERM but blocked recv still doesn't wake unless wat program polls `(stopped?)`.
- **Slices B+C together:** closes the orphan-child leak (cargo test runs no longer accumulate leaks).
- **Slice D:** demonstrates the discipline empirically + adds the structural probe for regression.
- **Slice E:** only needed if there are pipe-fd-blocked recvs (uncommon; from-pipe wrappers typically wrap user-level pipes that have explicit close discipline).

## Discipline mirror per stone

Each slice gets its own BRIEF + EXPECTATIONS + SCORE. Per `feedback_background_mechanical_agents`: orchestrator briefs sonnet, sonnet implements, atomic commit on green. Per `feedback_pre_existing_verification`: independent re-verification of load-bearing rows before commit.

Per `feedback_brief_constraint_contradictions`: BRIEFs must NOT have constraints contradicting deliverables. Narrow surface; SCORE doc explicitly in scope.

## Honest open edges

- **Async-signal-safety of `trigger_shutdown`:** the worker thread (which calls `trigger_shutdown` after reading from wake-pipe) runs in normal context. The signal handler ONLY writes a byte to the wake-pipe (POSIX-guaranteed async-signal-safe). The actual sender-drop happens in normal-context worker. Verified by reading `signal-safety(7)`.
- **Process group + setsid:** existing `setpgid(0,0)` in child branches makes children pgrp leaders. This is unchanged — PR_SET_PDEATHSIG works independently of pgrp membership.
- **PR_SET_PDEATHSIG inheritance:** the flag is RESET across `fork()` and across `execve()`. Each forked child must set it again. spawn-process forks but does NOT exec — we set it once in the child branch. Children of children (recursive spawn-process) each set it on their fork.

## Cross-references

- `DESIGN-SHUTDOWN-AWARE-CHANNELS.md` — full design + four-questions verdict + empirical proof
- `feedback_iterative_complexity` — why this is a backlog, not a BRIEF
- `feedback_background_mechanical_agents` — orchestrator-side discipline for sonnet BRIEFs
- `RUNTIME-BOOTSTRAP-BACKLOG.md` — Stone A's `bootstrap_wat_vm_process` is where `init_shutdown_signal()` gets called (Slice A integration point)
- `feedback_brief_constraint_contradictions` — BRIEF authoring discipline
- `INTERSTITIAL-REALIZATIONS.md` — context for why this surfaced from Stone C audit

## Status

Backlog drafted. Slice A ready to BRIEF. Awaiting orchestrator green light to draft BRIEF + spawn sonnet.
