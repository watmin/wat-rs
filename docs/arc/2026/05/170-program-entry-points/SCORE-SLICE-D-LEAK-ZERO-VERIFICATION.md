# Arc 170 Slice D SCORE — Empirical leak-rate measurement; PDEATHSIG race confirmed

**Branch:** arc-170-gap-j-v5-deadlock-state
**Date:** 2026-05-13
**Status:** SLICE-D DIAGNOSTIC COMPLETE — surfaced substrate-design defect, not flake

## Scorecard

| Row | What | Evidence | Pass |
|-----|------|----------|------|
| A | Slice C's probe accumulates orphans empirically | 13 `probe_pdeathsig_kills_orphan_child-*` procs alive 04:31–19:49 (15h window). All `ppid=1`, all 8 threads parked: 1 `futex_do_wait` (main), 1 `anon_pipe_read` (`wat-shutdown-wo`), 3 `futex_do_wait` (wat-thread workers), 3 `futex_do_wait` (stdio bridges) | PASS |
| B | The shutdown cascade is correct WHEN signal is delivered | Manual `kill -TERM` of two orphans → instant clean exit (cascade fires, all threads unparked, process gone). Slices A/B/C wiring intact | PASS |
| C | SIGTERM is in `SigCgt`, not `SigBlk`, not `SigIgn` | `grep -E "^Sig" /proc/<pid>/status` → `SigCgt: 0000000100004e43` (bit 14 set), `SigBlk: 0`, `SigIgn: 0x1000` (SIGPIPE only). Handler installed; not blocked; not ignored. The kernel simply did not deliver SIGTERM | PASS |
| D | Diagnostic clone of probe, parameterized by supervisor pre-exit delay | `tests/probe_pdeathsig_diagnostic.rs` reads `WAT_PROBE_SUPERVISOR_DELAY_MS`. Default 0 (current behaviour); >0 inserts `nanosleep` between supervisor pid write and `_exit(0)`. No substrate edits | PASS |
| E | A/B test: 50 trials at delay=0, 50 at delay=10ms | delay=0: 45/50 pass, **5 orphans accumulated (10% race rate)**. delay=10ms: **50/50 pass, 0 orphans**. A 10ms supervisor pause eliminates the race | PASS |
| F | Race window matches mechanism analysis | `spawn_process_child_branch` child path: dup2×3 → drop fds → `install_silent_panic_hook` → setpgid → **prctl** (`spawn_process.rs:343`). ~10 syscalls + heap-allocating `set_hook`. ~50–200µs to prctl. Supervisor exit ~5–20µs after `eval` returns. Same order of magnitude → ~10% race rate matches empirical 5/50 | PASS |
| G | Lifeline pipe is a structurally race-free alternative — 100/100 trials | `tests/probe_lifeline_pipe_proof.rs` — pure-libc demo: supervisor holds lifeline write-end; grandchild reads. Parent exits → kernel closes write-end → grandchild's `read()` returns 0 (EOF) → exit. 100 trials in **28ms** (~280µs/trial). **0 orphans across 100 trials.** No prctl, no signal, no timer, no race | PASS |
| H | Mechanism is the substrate's existing pattern, not a new invention | INTERSTITIAL § "How the shadow channel fans out": substrate piggybacks on crossbeam Sender::Drop disconnect-broadcast. Lifeline piggybacks on the symmetric kernel invariant: process-death closes its FDs. Both = "documented invariant; we don't write the fanout." Slice C's PDEATHSIG is the deviation: signal-handler intermediary that the rest of the architecture had already rejected (see "Wat disciplines its own designers" session-catch #3) | PASS |
| I | NO new Mutex/RwLock/CondVar | `grep -nE "Mutex\|RwLock\|CondVar" tests/probe_pdeathsig_diagnostic.rs tests/probe_lifeline_pipe_proof.rs` → zero hits | PASS |
| J | The lifeline FD subsumes Slice E's PipeFd-multiplex family | Slice E was scoped for tier-2 from-pipe Receivers to wake on shutdown via `epoll`/`poll(2)` over (pipe_fd, shutdown_eventfd). Lifeline adds an FD INPUT to the same multiplex; Slice E adds FD-aware recv OUTPUTS. Same kernel primitive (epoll/poll over pipe FDs). Same family. Forward work: one unified slice, not two | PASS |

## Files changed

- `tests/probe_pdeathsig_diagnostic.rs` — diagnostic clone of `probe_pdeathsig_kills_orphan_child`; supervisor `nanosleep` parameterized by `WAT_PROBE_SUPERVISOR_DELAY_MS` env var
- `tests/probe_lifeline_pipe_proof.rs` — 100-trial proof of the lifeline mechanism via pure libc fork chain (test → supervisor → grandchild); demonstrates parent-death detection via FD-close-on-exit kernel invariant
- `scripts/stability-100.sh` — earlier-committed; captures `---- TESTNAME stdout ----` failure bodies in workspace stability runs (separate commit)
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-D-LEAK-ZERO-VERIFICATION.md` — this file

No substrate edits. Slice D's deliverable was the empirical record. The forward fix lands in the unified slice that retires Slice C's mechanism.

## Honest deltas

### Slice D was BRIEF'd as "leak-zero verification" — it became "leak-class characterization"

The original Slice D scope: run `stability-100.sh` and verify zero PDEATHSIG-orphan accumulation. The verification ran and showed the OPPOSITE — 13 cumulative orphans across a 15h window of stability-100 invocations. Slice D as written would have shipped a FAIL.

The honest framing: Slice D's job was to surface the truth. It did. The mechanism Slice C shipped is race-prone at a measurable rate. Slice D's "PASS" rows above are about the DIAGNOSTIC succeeding, not about the substrate being clean.

### Slice C's INSCRIPTION (commit `fb9522d`) stays unchanged

Per `feedback_inscription_immutable`: *"what is inscribed is inscribed - all we can do is make forward progress - we do not hide our faults - we learn from them."* Slice C shipped a real mechanism that turns out to be the wrong shape for this substrate. The new slice closes that defect via a structurally-different mechanism; Slice C's INSCRIPTION is the historical record of the deviation and its discovery.

### The 10% rate matches expectation; the 28ms × 100 trials matches expectation

50 trials per arm is enough to make 5/50 vs 0/50 unambiguous at this magnitude. The lifeline proof runs end-to-end in ~280µs/trial — consistent with the cost of two fork/exit cycles plus blocking-pipe-read EOF detection. Both numbers reinforce the mechanism analysis (Row F).

### What this slice does NOT close

- The substrate edit. The lifeline pipe needs to be wired into `spawn_process_child_branch` and `child_branch_from_source`; the shutdown worker's `select!` needs to grow to multiplex (wake-pipe, lifeline-pipe). That lands in the new unified slice (DESIGN to follow).
- The `test-*` procs we also saw stuck (7-thread shape, no `wat-shutdown-wo` thread). Different bootstrap path missing `init_shutdown_signal()`. Tracked separately; not Slice C/D/E concern.
