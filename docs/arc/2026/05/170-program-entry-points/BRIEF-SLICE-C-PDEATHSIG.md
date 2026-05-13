# Arc 170 Slice C BRIEF — PR_SET_PDEATHSIG in child fork branches

**Slice:** C of SHUTDOWN-AWARE-CHANNELS-BACKLOG (5 slices A-E, all mandatory).
**Predecessor:** Slice B shipped at `d3810d3`. Slice A at `6d5d85c`.
**Goal:** Add `libc::prctl(PR_SET_PDEATHSIG, SIGTERM, ...)` immediately after `setpgid(0, 0)` in BOTH child fork branches. This makes the kernel deliver SIGTERM to a child process when its parent dies for ANY reason (clean exit, panic, OOM-kill, segfault). The SIGTERM then triggers Slice B's cascade in the child.

## Context (read before starting)

1. `docs/arc/2026/05/170-program-entry-points/BRIEF-SLICE-C-PDEATHSIG.md` (this file)
2. `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-B-CROSSBEAM-MULTIPLEX.md` — predecessor; verify the cascade Slice C feeds is working
3. `docs/arc/2026/05/170-program-entry-points/DESIGN-SHUTDOWN-AWARE-CHANNELS.md` — full design (PR_SET_PDEATHSIG section + edges)
4. `docs/arc/2026/05/170-program-entry-points/SHUTDOWN-AWARE-CHANNELS-BACKLOG.md` — 5-slice plan
5. `man 2 prctl` — kernel manpage; verify PR_SET_PDEATHSIG semantics (resets across fork+exec; per-thread but child threads inherit the parent process's state at fork)
6. `src/spawn_process.rs::spawn_process_child_branch` — existing setpgid site (around line 321)
7. `src/fork.rs` — existing setpgid site for fork-program (around line 1032)

## Substrate edits

### 1. `src/spawn_process.rs` — set PR_SET_PDEATHSIG in spawn-process child

In `spawn_process_child_branch`, IMMEDIATELY AFTER the existing `setpgid(0, 0)` call (around line 321):

```rust
// Arc 170 Slice C — PR_SET_PDEATHSIG.
// Tells the kernel: when my parent process dies (for ANY reason —
// clean exit, panic, segfault, OOM-kill), deliver SIGTERM to me.
// This is the substrate's way to ensure orphaned children don't
// outlive their parents indefinitely. The SIGTERM triggers the
// Slice B cascade (signal handler → wake pipe → worker → drop
// SHUTDOWN_TX → all blocked recvs wake with Shutdown).
//
// MUST be called after setpgid + after we are sure we're in the
// child (post-fork). Lives across the rest of the child's life
// until exec (we don't exec) or until reset.
if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM as libc::c_ulong, 0, 0, 0) } < 0 {
    let err = std::io::Error::last_os_error();
    // Child branch — emit structured-stderr per arc 170 slice 1i and exit.
    // prctl failure is rare but must surface honestly.
    eprintln!(
        "#wat.kernel/ProcessPanics {{\"message\":\"prctl(PR_SET_PDEATHSIG) failed: {}\"}}",
        err
    );
    unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
}
```

Use the SAME `EXIT_STARTUP_ERROR` exit code already used for the existing dup2 failures in this function (preserves the exit-code contract).

### 2. `src/fork.rs` — set PR_SET_PDEATHSIG in fork-program-ast child

In the child branch around line 1032, IMMEDIATELY AFTER the existing `setpgid(0, 0)` call (mirror the spawn_process.rs edit; same shape):

```rust
if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM as libc::c_ulong, 0, 0, 0) } < 0 {
    let err = std::io::Error::last_os_error();
    eprintln!(
        "#wat.kernel/ProcessPanics {{\"message\":\"prctl(PR_SET_PDEATHSIG) failed: {}\"}}",
        err
    );
    // Use whatever exit code fork.rs uses for setpgid failure (same class)
    unsafe { libc::_exit(/* match existing setpgid-failure exit code */) };
}
```

CHECK: find the existing setpgid failure exit code in fork.rs and reuse it. Don't invent a new exit code.

### 3. New probe `tests/probe_pdeathsig_cascade.rs`

The probe demonstrates:
1. Parent test process forks a child via `spawn-process` (which now sets PDEATHSIG).
2. Parent (test process worker thread or test main) IMMEDIATELY exits or releases the fork.
3. Kernel delivers SIGTERM to the orphan.
4. Orphan's Slice B cascade wakes its blocked recv → child exits cleanly within 1s wall-clock.
5. Probe verifies the child process is reaped within bounded time.

**CRITICAL constraint:** the probe MUST run in an isolated test-binary context. The PARENT for the orphan-test is NOT the cargo test binary itself (we can't kill the test binary). The shape: spawn a `:wat::kernel::spawn-process` grandchild from inside a sandboxed wat-vm; the wat-vm IS the parent; let the wat-vm exit; verify the grandchild dies via PDEATHSIG.

Simpler shape: use `run-hermetic-with-io` which already does the spawn-process dance. After the hermetic block, verify no grandchild processes survive (pgrep + grep on a unique test identifier the grandchild prints to stderr before being killed).

OR even simpler: Rust-level probe that:
- Forks twice via libc::fork
- Outer fork = "parent we'll kill"
- Inner fork = "grandchild we want PDEATHSIG to kill"
- Outer fork calls `setpgid + prctl(PR_SET_PDEATHSIG, SIGTERM)` then exec's nothing — just blocks
- Inner fork inherits NOTHING — must set its own PDEATHSIG (per kernel doc, PDEATHSIG resets across fork)
- Parent test process kills outer fork → kernel delivers SIGTERM to outer fork
- Outer fork's signal handler triggers Slice B cascade → outer fork exits
- HOWEVER — what about the inner fork? It must set its OWN PDEATHSIG to be killed when outer fork dies. That's the substrate's job for each spawn-process site.

**The probe verifies one specific claim:** spawn-process children set PDEATHSIG correctly. Test by:
1. Spawning a child via the substrate's spawn-process (which sets PDEATHSIG after the edit)
2. Killing the PARENT (the test wat-vm) via SIGTERM or just letting it exit
3. Verifying the child also dies within 1s

Implementation note: it's hard to "exit the test wat-vm" mid-test without exiting cargo test. The probe should fork a "supervisor" Rust process that:
- Forks a child via `spawn-process` substrate primitive
- IMMEDIATELY exits (calls libc::_exit)
- The CHILD now has PPID=1 (orphaned)
- The CHILD should receive SIGTERM via PDEATHSIG
- The TEST harness verifies the child died within 1s by waitpid'ing or pgrep'ing

This requires the Rust test to fork its own supervisor. Doable with `std::process::Command::new(current_exe).args(["--test", "..."])` re-entry pattern, or simpler: fork directly in the probe and have the child do the wat work.

**Recommended shape:**
```rust
#[test]
fn probe_pdeathsig_kills_orphan_child() {
    // 1. Fork a supervisor child
    let supervisor_pid = unsafe { libc::fork() };
    if supervisor_pid == 0 {
        // Supervisor: spawn a wat-process grandchild via substrate, then exit
        // (orphaning the grandchild). Grandchild should receive SIGTERM via
        // PR_SET_PDEATHSIG and exit cleanly within 1s.
        let world = freeze_ok(...);
        let call = WatAST::List(/* spawn-process some-fn */, ...);
        let _grandchild = eval(&call, ...);  // grandchild PID is in the Process struct
        // Write grandchild PID to a known file or pipe for the test to observe
        // Then exit, orphaning the grandchild
        unsafe { libc::_exit(0) };
    }
    // 2. Test (parent of supervisor): wait for supervisor to exit
    // 3. Read grandchild PID from the file/pipe
    // 4. Within 1s, verify grandchild process is gone (waitpid with WNOHANG loop;
    //    NO wall-clock sleep — use bounded retry with a rendezvous pipe)
    // 5. Assert: grandchild exited within 1s (proves PDEATHSIG cascade worked)
}
```

Use SAME rendezvous discipline as `probe_shutdown_cascade_crossbeam` — no wall-clock sleeps; use pipe-based ack with bounded waitpid retries.

## Scorecard (10 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `libc::prctl(PR_SET_PDEATHSIG, SIGTERM, ...)` in `spawn_process_child_branch` after `setpgid` | grep + read order |
| B | `libc::prctl(PR_SET_PDEATHSIG, SIGTERM, ...)` in `fork.rs` child branch after `setpgid` | grep + read order |
| C | Both sites emit structured-stderr `ProcessPanics` EDN on prctl failure (consistent with existing dup2 failure handling) | read |
| D | Both sites call `libc::_exit` with the existing `EXIT_STARTUP_ERROR` (or fork.rs equivalent) on failure | read |
| E | No exit-code drift: existing exit-code contracts for the two sites preserved | read |
| F | `cargo build --release --workspace` passes | full build |
| G | `cargo test --release -p wat --test test` shows 167/7 baseline (bimodal flake tolerable; same characterization as Slice A/B) | 3+ independent runs |
| H | New probe `probe_pdeathsig_kills_orphan_child` PASSES — orphan grandchild dies within 1s after parent exit | cargo test |
| I | No new orphan processes accumulate after running the new probe (pgrep check) | pgrep after probe |
| J | NO new Mutex/RwLock/CondVar introduced | grep verification |

**All 10 rows must PASS.**

## Constraints (NARROW per feedback_brief_constraint_contradictions)

- **In scope:** `src/spawn_process.rs`, `src/fork.rs`, new probe file. SCORE doc creation.
- **Out of scope:** typed_recv changes (Slice B's territory). PipeFd multiplex (Slice E). End-to-end leak-zero stability run (Slice D).
- **Must not touch:** `src/check.rs` (committed verifier).
- **Must not introduce:** wall-clock sleeps in the probe. Use rendezvous + waitpid bounded loop.

## Discipline

- Pre-action sweep per FM 17: confirm prctl(2) constants match Linux ABI (PR_SET_PDEATHSIG = 1, SIGTERM = 15). The libc crate provides these constants; use those, don't hardcode magic numbers.
- Verify PDEATHSIG semantics from `man 2 prctl`: resets across fork (so each child must set its own — good, the substrate IS setting it in each fork site). Resets across exec (we don't exec — fine). Per-thread, but the syscall-set value applies to the process at process death (the kernel tracks parent process death, not parent thread death).
- Atomic commit on green. SCORE doc:
  `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-C-PDEATHSIG.md`
- Reap orphans (`pkill -9 -f "target/release/deps/test-"`) before each cargo invocation

## Mode B trigger

- prctl(2) returns -1 unexpectedly during probe (kernel might not support PR_SET_PDEATHSIG on the test env) → escalate, investigate
- New probe shows grandchild does NOT die within 1s → cascade is broken; halt and surface (could be Slice B incomplete, could be PDEATHSIG semantics misunderstanding)
- ZERO-MUTEX violation surfaces

## Runtime band

**60-90 min sonnet.** Hard cap 180 min. ScheduleWakeup at T+3600s.

Smaller than Slice B because the edit shape is well-bounded (~5 lines per fork site + one probe). The probe design (fork-supervisor + rendezvous + waitpid loop) is the main complexity.
