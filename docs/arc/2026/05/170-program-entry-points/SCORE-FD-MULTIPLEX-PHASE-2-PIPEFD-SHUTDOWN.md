# Arc 170 FD-multiplex Phase 2 SCORE — tier-2 PipeFd Receivers wake on shutdown

**Branch:** `arc-170-gap-j-v5-deadlock-state`
**Commit context:** post-Phase-1E (d609c1e), Phase 2 edits not yet committed per BRIEF.

## Scorecard

| Row | What | Status | Evidence |
|-----|------|--------|----------|
| A | `SHUTDOWN_BROADCAST_READ_FD: AtomicI32` global in src/runtime.rs | YES | `grep -n "SHUTDOWN_BROADCAST_READ_FD" src/runtime.rs` → line 200 (static def) + line 269 (init store). 2 hits. |
| B | `init_shutdown_signal_with_inputs` creates second pipe + stores broadcast_r_fd | YES | Lines 260–269: `libc::pipe(broadcast_fds)` + `SHUTDOWN_BROADCAST_READ_FD.store(broadcast_r_fd, ...)`. |
| C | Worker closure holds `broadcast_w_fd`; `libc::close(broadcast_w_fd)` AFTER `trigger_shutdown()` | YES | Line 310: `trigger_shutdown()`. Line 313: `unsafe { libc::close(broadcast_w_fd); }`. Order confirmed. |
| D | `WatReader` trait gains `as_raw_fd_for_poll() -> Option<i32>` with `None` default | YES | `grep -n "as_raw_fd_for_poll" src/io.rs` → line 60 (trait default returning None). |
| E | `PipeReader` overrides `as_raw_fd_for_poll` to return `Some(fd)` | YES | Line 582 in src/io.rs: `fn as_raw_fd_for_poll(&self) -> Option<i32> { Some(self.fd.as_raw_fd()) }`. |
| F | `typed_recv` PipeFd arm: poll(2) on (pipe_fd, broadcast_fd); POLLHUP on broadcast → Shutdown | YES | src/typed_channel.rs: loop over `libc::poll(fds, 2, -1)`; `fds[1].revents != 0` returns `RecvOutcome::Shutdown`; `fds[0].revents != 0` breaks to read_line. |
| G | `typed_try_recv` PipeFd arm: poll(timeout=0) on (broadcast_fd, pipe_fd); Shutdown if broadcast ready | YES | src/typed_channel.rs: `libc::poll(fds, 2, 0)`; broadcast_fd at fds[0]; `fds[0].revents != 0` returns `RecvOutcome::Shutdown`; n==0/n<0 returns `RecvOutcome::Disconnected`. Doc comment updated. |
| H | NEW probe `tests/probe_shutdown_cascade_pipefd.rs` exists; uses `ReceiverInner::PipeFd` + raises SIGTERM | YES | File exists. `PipeReader::from_owned_fd` + `ReceiverInner::PipeFd(reader)` + `libc::raise(libc::SIGTERM)`. |
| I | `cargo build --release --workspace --tests` clean | YES | `Finished release profile [optimized] target(s) in 54.70s` — zero errors, zero new warnings. |
| J | NEW probe PASSES + all 5 existing probes still pass | YES | All 6 probes pass in isolation: probe_shutdown_cascade_pipefd (1/1), probe_shutdown_cascade_crossbeam (1/1), probe_lifeline_pipe_proof (1/1), probe_lifeline_orphan_clean_via_substrate (1/1), probe_lifeline_orphan_clean_via_fork_program (1/1), probe_pdeathsig_kills_orphan_child (1/1). |

**Final score: 10/10 — Mode A (clean ship).**

## Honest deltas

**WatReader trait location.** BRIEF guessed `src/io.rs` — confirmed correct. No delta.

**PipeReader is the only FD-backed reader.** `RealStdin` wraps `std::io::Stdin` (stdlib-locked, no raw fd exposed) and correctly falls through to the default `None`. `StringIoReader` is in-memory, also `None`. No additional overrides needed.

**`typed_try_recv` param names.** The function signature uses `_types` and `_span` (underscore-prefixed unused params). The new PipeFd arm passes `_span` to `read_line` and `_types` to `read_edn` — using the actual values correctly. No issue.

**`FromRawFd` import in probe.** `OwnedFd::from_raw_fd` requires `use std::os::fd::FromRawFd` in scope. Added to probe imports.

**No `WatReader` re-export gap.** `wat::io::WatReader` is accessible from test files via `wat::io`. The probe imports `wat::io::{PipeReader, WatReader}` cleanly.

## Calibration

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 15–25 min | ~18 min |
| Scorecard rows | 10/10 PASS | 10/10 PASS |
| Honest deltas | 1–3 surfaces | 4 (all minor; no structural surprises) |
| Mode | A (clean) | A (clean) |

## Mechanism recap

The same kernel-FD-close-on-drop primitive used four times now:
- Phase 1B/1C: lifeline_w drop (parent death) → child POLLHUP → shutdown
- Slice B: crossbeam Sender drop → SHUTDOWN_RX disconnect → select! fires
- Phase 2 (this): broadcast_w_fd close (after trigger_shutdown) → all PipeFd recvs POLLHUP → RecvOutcome::Shutdown

After Phase 2: **no recv exposed from the substrate can outlive a shutdown event.** All three tiers observed.
