# Arc 170 FD-multiplex Phase 3 BRIEF AMENDMENT — probe rendezvous via pidfd_open

**Predecessor:** Phase 3 substrate work shipped uncommitted in working tree. Sonnet's SCORE doc identified 4 probe regressions from the canonical close-sweep — probes relied on FD-inheritance leak that Phase 3 closed. Phase 3 substrate edits are correct (substrate-as-teacher: probes were leveraging an implementation-detail leak; honest substrate refuses).

**Goal:** Update the 4 affected probes to use `pidfd_open(2) + poll(2)` as the process-death rendezvous instead of `done_pipe` FD-inheritance. Atomic commit with the Phase 3 substrate edits.

## Why pidfd_open

The 4 probes need to detect "grandchild wat-vm process exited" from a parent (test) that is NOT the immediate parent of the grandchild. The chain is `test → supervisor → grandchild`. Today these probes used `done_pipe` (test-held read-end; grandchild-inherited write-end) — kernel closes write-end on grandchild exit → POLLHUP on read-end. Phase 3's `close_inherited_fds_above_stdio` closes the grandchild's inherited write-end at startup, defeating the rendezvous.

The substrate-correct replacement: `pidfd_open(grandchild_pid, 0)` returns a fd that becomes readable when the process exits. Kernel-guaranteed. Linux 5.3+. Per `feedback_no_windows` we're Linux-only and unapologetic.

Test reads grandchild_pid from supervisor (existing pid_pipe pattern), then `pidfd_open(pid, 0)` in the test, then `poll(pidfd, POLLIN, 1000ms)`. POLLIN → grandchild exited. Lock-step via kernel-managed FD; not inheritance-dependent; survives close-sweep because the test process owns the pidfd directly.

## Edits

### 1. `tests/probe_lifeline_orphan_clean_via_substrate.rs` — pidfd rendezvous

Replace `done_pipe` setup + supervisor's done_w + grandchild's done_w + test's `poll(done_r, POLLHUP, 1000ms)` with:

```rust
// After test reads grandchild_pid from pid_pipe:
let pidfd = unsafe {
    libc::syscall(libc::SYS_pidfd_open, grandchild_pid as libc::c_long, 0i32 as libc::c_long)
} as libc::c_int;
assert!(pidfd >= 0, "pidfd_open(grandchild_pid={}) failed: {}", grandchild_pid, std::io::Error::last_os_error());

let mut pollfd = libc::pollfd { fd: pidfd, events: libc::POLLIN, revents: 0 };
let t0 = Instant::now();
let poll_ret = unsafe { libc::poll(&mut pollfd as *mut _, 1, 1000) };
let elapsed = t0.elapsed();
unsafe { libc::close(pidfd) };

assert!(
    poll_ret > 0,
    "pidfd_open POLLIN did not fire within 1s — lifeline cascade broken (poll_ret={}, elapsed={:?})",
    poll_ret, elapsed
);
```

Remove all `done_pipe`, `done_r`, `done_w` plumbing. Supervisor no longer needs to drop done_w. Test no longer creates done_pipe.

Keep the `/proc/<pid>/stat` zombie-state verification as a final assertion — that's a separate check that the process is actually gone (not just signalled).

### 2. `tests/probe_pdeathsig_kills_orphan_child.rs` — same migration

Same pattern as #1. This is the historical-marker probe; per `feedback_inscription_immutable` we don't edit it lightly, but the rendezvous-mechanism update is required for the probe to keep PASSING under the new substrate semantics. The OBSERVABLE CONTRACT (grandchild dies via cascade) is unchanged; only the OBSERVATION MECHANISM updates.

Add a header comment noting:
```
//! Rendezvous mechanism updated 2026-05-13: done_pipe FD-inheritance
//! rendezvous replaced with pidfd_open + poll. Phase 3 of arc 170
//! FD-multiplex adds canonical close_inherited_fds_above_stdio to
//! spawn-process, which closes inherited test-pipes; pidfd_open is the
//! substrate-correct wire that survives the close-sweep. Observable
//! contract unchanged.
```

### 3. `tests/probe_pdeathsig_diagnostic.rs` — same migration

Same pattern. Header comment update similar to #2.

### 4. `tests/probe_row_g_sweep.rs` — no change needed

This is a subprocess harness that runs `probe_pdeathsig_diagnostic` 50 times. Once #3 is fixed, this passes automatically.

## Scorecard (6 rows)

| Row | What | Evidence |
|-----|------|----------|
| A | All 3 probes (substrate, pdeathsig, diagnostic) call `pidfd_open` instead of `done_pipe` | `grep -nE "done_pipe\|done_w\|done_r\|pidfd_open" tests/probe_lifeline_orphan_clean_via_substrate.rs tests/probe_pdeathsig_kills_orphan_child.rs tests/probe_pdeathsig_diagnostic.rs` — done_pipe references gone; pidfd_open present in each |
| B | Each probe PASSES 1/1 in isolation | `cargo test --release --test <probe_name>` shows `1 passed; 0 failed` for each |
| C | `probe_row_g_sweep` PASSES (50/50 sub-trials) | `cargo test --release --test probe_row_g_sweep` shows `1 passed; 0 failed` |
| D | `cargo build --release --workspace --tests` clean | build output |
| E | Workspace failure set: 9 or fewer (pre-existing svc-test + tmp + wat-cli + lifeline flake; NO new probe regressions) | `cargo test --release --workspace --no-fail-fast` failure count |
| F | Phase 3's primary symptom (stream + lru pressure failures) STILL resolved | absence in workspace failure list |

## Constraints

- ONLY edit the 3 probe files (probe_row_g_sweep doesn't need changes).
- DO NOT modify substrate code (Phase 3 substrate edits already in working tree).
- Use `pidfd_open` via `libc::syscall(libc::SYS_pidfd_open, ...)` — no third-party crate.
- Maintain the `/proc/<pid>/stat` zombie verification as the final post-rendezvous assertion.

## STOP-at-first-red

- `pidfd_open` returns -1 with ENOSYS → kernel doesn't support it (pre-5.3). Unlikely on user's System76 + recent Linux. STOP and report — would need to fall back to /proc polling.
- Any probe fails in isolation post-edit → STOP. The rendezvous shift should be behavior-preserving for the observable contract.

## On completion

Append a SCORE-AMENDMENT section to `SCORE-FD-MULTIPLEX-PHASE-3-CANONICAL-CHILD-INIT.md`. Note the 6-row scorecard outcome.

Do NOT commit. Orchestrator atomic-commits Phase 3 substrate edits + this probe migration together.
