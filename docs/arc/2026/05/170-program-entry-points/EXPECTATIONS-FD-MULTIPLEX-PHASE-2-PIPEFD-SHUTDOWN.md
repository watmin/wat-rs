# Arc 170 FD-multiplex Phase 2 EXPECTATIONS

**BRIEF:** `BRIEF-FD-MULTIPLEX-PHASE-2-PIPEFD-SHUTDOWN.md`

## Independent prediction

**Runtime band:** 15–25 minutes Mode A.

Reasoning:
- 4 substrate edits (runtime.rs global + init pipe + worker close; io.rs trait method + PipeReader override; typed_channel.rs typed_recv arm; typed_channel.rs typed_try_recv arm).
- 1 new probe ~150 lines (mirror probe_shutdown_cascade_crossbeam shape, swap crossbeam for from-pipe Receiver).
- WatReader trait location is the main unknown — sonnet greps to find it.

**Time-box:** ScheduleWakeup at 50 minutes (2× upper-bound).

## SCORE methodology

Each row YES/NO with evidence per `feedback_four_questions_yes_no`:

- **Row A**: `grep -n "SHUTDOWN_BROADCAST_READ_FD" src/runtime.rs` — at least 2 hits (def + store).
- **Row B**: grep for `broadcast_fds` or `broadcast_w_fd` in init_shutdown_signal_with_inputs — shows the pipe creation + store call.
- **Row C**: read the worker closure; `libc::close(broadcast_w_fd)` appears AFTER `trigger_shutdown()`.
- **Row D**: `grep -rn "as_raw_fd_for_poll" src/` — at least 2 hits (trait def + impl).
- **Row E**: PipeReader's `as_raw_fd_for_poll` returns `Some(...)`.
- **Row F**: `awk` extraction of the PipeFd arm in `typed_recv`; visual confirm of: poll(2) call, broadcast_fd revents check returning Shutdown, fallback to read_line.
- **Row G**: same for `typed_try_recv`. Poll timeout=0; Shutdown short-circuit; Disconnected on empty.
- **Row H**: `ls tests/probe_shutdown_cascade_pipefd.rs`; grep shows `Receiver/from-pipe` or `receiver_from_pipe` + SIGTERM raise.
- **Row I**: `cargo build --release --workspace --tests 2>&1 | tail -3` — Finished, zero errors.
- **Row J**: each probe invoked in isolation; all PASS.

## Honest deltas to watch for

- **WatReader trait location.** May live in `src/io.rs`, `src/thread_io.rs`, or a re-exported module. Adjust the BRIEF's "src/io.rs" reference if grep shows otherwise.
- **PipeReader vs other FD-backed readers.** If multiple FD-backed reader types exist (PipeReader, FileReader, etc.), override `as_raw_fd_for_poll` on ALL of them.
- **StringReader / InMemoryReader behavior under shutdown.** With default None they fall back to bare `read_line` — no shutdown observability. This is acceptable (these are in-memory/test-only; users don't reach them on shutdown). If a test happens to exercise such a reader during shutdown and hangs, surface as honest delta.
- **`run_in_fork` for the new probe.** The existing `probe_shutdown_cascade_crossbeam` uses a child-isolation pattern that may have its own quirks; mirror it precisely.
- **Pipe FD leak in test.** The probe creates a pipe pair; both ends must be properly owned (OwnedFd) so they drop cleanly. The receiver inherits the read-end; the write-end is held in test scope. If shutdown fires before recv started, the test pattern still needs to work — verify the order.

## Workspace baseline (post-Phase-1E, commit d609c1e)

- `cargo build --release --workspace --tests`: clean
- All 5 probes PASS in isolation:
  - probe_shutdown_cascade_crossbeam (Slice B)
  - probe_lifeline_pipe_proof (mechanism, 100/100)
  - probe_lifeline_orphan_clean_via_substrate (Phase 1D)
  - probe_lifeline_orphan_clean_via_fork_program (Phase 1E)
  - probe_pdeathsig_kills_orphan_child (historical marker)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 15–25 min | TBD |
| Scorecard rows | 10/10 PASS | TBD |
| Honest deltas | 1–3 surfaces | TBD |
| Mode | A (clean) | TBD |
