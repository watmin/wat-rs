# Arc 170 Slice C EXPECTATIONS — PR_SET_PDEATHSIG

**One slice.** Delivers the SIGTERM signal to orphaned children, completing the Slice B cascade for the orphan-child case. After C: forked children die when their parent dies, via the kernel-delivered SIGTERM → Slice B cascade chain.

## Runtime band

**60-90 min sonnet.** Hard cap 180 min. ScheduleWakeup at T+3600s.

Smaller than Slice B — two ~5-line edits + one probe. The probe design (fork-supervisor + rendezvous + waitpid loop, NO wall-clock sleeps) is the main attention site.

## Scorecard (10 rows) — see BRIEF for full criteria

| Row | What | Pass |
|-----|------|------|
| A | prctl(PR_SET_PDEATHSIG, SIGTERM) in spawn_process_child_branch | read |
| B | prctl(PR_SET_PDEATHSIG, SIGTERM) in fork.rs child branch | read |
| C | Both sites emit ProcessPanics EDN on prctl failure | read |
| D | Both sites use existing exit-codes for failure | read |
| E | No exit-code drift | read |
| F | cargo build --release --workspace passes | full build |
| G | cargo test 167/7 baseline (bimodal flake tolerable) | 3+ independent runs |
| H | probe_pdeathsig_kills_orphan_child PASSES — orphan dies within 1s | cargo test |
| I | No new orphan accumulation after probe | pgrep check |
| J | ZERO-MUTEX compliance maintained | grep |

**10 rows. All must PASS.**

## Discipline mirror (orchestrator-side)

- FM 9: independent re-verification of Rows G, H, I, J before commit
- FM 12: `model: "sonnet"` explicit
- FM 16: no Bash/tool-availability preamble in BRIEF
- FM 17: pre-action sweep — verify libc::PR_SET_PDEATHSIG + libc::SIGTERM constants are used (not magic numbers); verify prctl(2) semantics via `man 2 prctl`
- Constraint pattern corrected per `feedback_brief_constraint_contradictions` — narrow surface; SCORE explicitly in scope, src/check.rs explicitly out, typed_recv explicitly out (Slice B's job)
- Atomic commit on success; ScheduleWakeup at T+3600s
- Reap orphans (`pkill -9 -f "target/release/deps/test-"`) before each cargo invocation

## Mode B trigger

- prctl(2) returns -1 in probe context — kernel might not support PDEATHSIG; investigate, don't ship broken
- Grandchild does NOT die within 1s — cascade is broken somewhere; halt and surface
- ZERO-MUTEX violation surfaces (no expected source for this in C, but vigilant)
- Wall-clock sleep introduced in probe — rendezvous required per lock-step doctrine

## What this slice does NOT do

- Does NOT modify typed_recv (Slice B already did)
- Does NOT add PipeFd multiplex (Slice E)
- Does NOT run the stability-100 leak-zero verification (Slice D's job — but C unblocks it)
- Does NOT modify src/check.rs

## What ships after this slice

After Slice C:
- Slice B's cascade fires for orphaned children too (parent dies → kernel delivers SIGTERM → cascade)
- The dominant leak class (cargo test orphans) is structurally closed at tier 1
- Slice D's stability run should show zero NEW leaks accumulating
- Tier-2 PipeFd recvs still block on shutdown (Slice E) — bare from-pipe recvs without typed_recv multiplex still hang on EOF
