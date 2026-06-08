# SCORE — Stone 6.4: THE REBIRTH GATE — the v5 deadlock is DEAD

**Mode A.** Sonnet ~17 min. The kill of the bug that named the branch
(`arc-170-gap-j-v5-deadlock-state`) — root-caused 6.3, killed here.

## The kill, verified (orchestrator's own enveloped runs)

| # | Row | Result |
|---|-----|--------|
| 1 | THE GATE — comms in-suite, BOTH detectors green, **5 consecutive rounds** | ✓ 52/0/6 ×5 (0.23–0.26s each); the run-order coin has no tails left |
| 2 | The bug's own path sealed: `run_in_fork` child body calls `rebirth_substrate_after_fork()` FIRST, before `body()` | ✓ read child.rs:163 |
| 3 | The honest delta sound: `child_post_fork_init_preserving` routes rebirth through step 4's pid-aware init (WITH the lifeline fd) — not an explicit pre-call that would no-op lifeline registration | ✓ lifeline_orphan_clean_via_substrate ok + lifeline_pipe_zero_orphans_across_100_trials ok |
| 4 | lib 943/0/1 · nursery 865/4/4 (4 = parked-255) · check 0 · clippy no-new | ✓ own runs |
| 5 | Enveloped: channel_pipes 23/0 · gamma 5/0 · hermetic 2/0 | ✓ own runs (sonnet) + spot |

## The fix (the smallest possible root for the largest scar)

THE ENTIRE v5 DEADLOCK WAS A GLOBAL. `SHUTDOWN_RX: OnceLock` whose worker
thread does not survive `fork`: clone3 copies the state, not the thread; the
idempotence guard (`runtime.rs:233`) told the child "already initialized" and
no-op'd the rebuild; SIGTERM's wake byte landed in a pipe whose reader was a
ghost; the recv never woke. Run-order-dependent (parent inits first → child
inherits a corpse → hang; child inits first → real → pass) — which is why it
read as a flaky "old-stack" hang for weeks instead of a deterministic bug.

Two layers killed it:
1. **The guard can no longer lie** — `SHUTDOWN_RX` OnceLock → `AtomicPtr` +
   `SHUTDOWN_INIT_PID`; the guard rebuilds when `pid != getpid()` (fresh
   channel + wake-pipe + broadcast-pipe + worker; the inherited COW boxes
   LEAK BY DESIGN — freeing a parent copy in the child is the heresy).
2. **The rebirth gate** — `rebirth_substrate_after_fork()` (the doc-contract:
   attendant-bearing globals rebirth here; pre-gate region async-signal-safe;
   inventory = the shutdown infra; the trio rebirths via bootstrap; fork+exec
   is the banked top rung). Called first from `run_in_fork`; routed through
   step 4 of the canonical preserving sequence.

STOP-1/2/3 all NOT triggered: the comms wiring reads `shutdown_rx()` fresh
per call (getter-swap serves child-created channels); no enveloped timeout;
no OnceLock consumer depended on set-once.

## The standing

The deadlock that OPENED arc 214 — `v5` in the branch name, weeks of
containment, the envelope ritual's whole reason for being — is **dead,
verified, deterministic.** Both detectors that timed out forever now pass
in-suite, five rounds running. The branch name's `v5` is now false.

This unblocks the stability-100 soak (#207) — the soak would have hung on
round one until this landed; now it can run to prove the *whole* suite
race-free. NEXT: 6.w (ward channel/ + process/ + the value//comms
touch-audits) → Slice 7 → the soak → Slice 9 (the triple INSCRIPTION).
