# Arc 170 FD-multiplex Phase 2 BRIEF — tier-2 PipeFd Receivers wake on shutdown

**Phase:** 2 of DESIGN-FD-MULTIPLEX-SHUTDOWN.md.
**Predecessors:** Phases 1A–1E shipped at `61217c7..d609c1e`. INPUT side of substrate shutdown is fully plumbed (SIGTERM/SIGINT + parent-death → trigger_shutdown). OUTPUT side covered for crossbeam recvs via Slice B's `typed_recv` select! on SHUTDOWN_RX. This Phase covers the LAST tier of consumers: `Receiver/from-pipe` (tier-2, OS-pipe-backed) recvs today block on `libc::read(pipe_fd)` and cannot observe shutdown.
**Goal:** Substrate adds a shutdown-broadcast pipe alongside the wake-pipe. Worker drops the broadcast write-end after `trigger_shutdown()`. `typed_recv`'s PipeFd arm multiplexes on (pipe_fd, broadcast_r_fd) via `libc::poll(2)`; POLLHUP on broadcast_r → `RecvOutcome::Shutdown`.

## What gets minted

A SECOND pipe at `init_shutdown_signal_with_inputs` time:
- `broadcast_r` — read-end, raw fd stored in a public atomic (e.g., `SHUTDOWN_BROADCAST_READ_FD: AtomicI32`); ALL consumers poll this fd
- `broadcast_w` — write-end, captured into the worker thread's closure; dropped after `trigger_shutdown()` completes; this drop EOFs all readers

The mechanism is the same one the lifeline uses (parent-death → FD close → reader EOF) and the same crossbeam Sender::Drop relies on (writer drop → channel disconnect → readers wake). Phase 2 is the third application of this primitive at the third tier.

After Phase 2: **no recv exposed from the substrate can outlive a shutdown event.** All three tiers — crossbeam (Slice B), pipe-fd (this Phase), future thread-bridge (out of scope) — observe shutdown structurally.

## Substrate edits

### 1. `src/runtime.rs` — add SHUTDOWN_BROADCAST_READ_FD; init creates the second pipe

Add near the existing globals (around line 192):

```rust
/// Arc 170 Phase 2 — substrate-owned shutdown broadcast read-fd.
/// Worker holds the write-end; drops it after trigger_shutdown.
/// All `Receiver/from-pipe` recvs poll this fd; POLLHUP → Shutdown.
/// Value -1 until init_shutdown_signal_with_inputs runs; valid fd
/// after. Once set, never re-set (idempotent init).
pub static SHUTDOWN_BROADCAST_READ_FD: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(-1);
```

In `init_shutdown_signal_with_inputs` (around line 215 — right after the wake-pipe creation), add a second pipe:

```rust
// Phase 2 — broadcast pipe for tier-2 PipeFd recvs.
let mut broadcast_fds = [0_i32; 2];
let broadcast_result = unsafe { libc::pipe(broadcast_fds.as_mut_ptr()) };
if broadcast_result != 0 {
    let msg = b"substrate: pipe(2) failed during broadcast init\n";
    unsafe { libc::write(2, msg.as_ptr() as *const _, msg.len()) };
    std::process::exit(1);
}
let broadcast_r_fd = broadcast_fds[0];
let broadcast_w_fd = broadcast_fds[1];
SHUTDOWN_BROADCAST_READ_FD.store(broadcast_r_fd, Ordering::SeqCst);
```

Update the worker closure to hold `broadcast_w_fd` and close it AFTER `trigger_shutdown()`:

```rust
std::thread::Builder::new()
    .name("wat-shutdown-worker".to_string())
    .spawn(move || {
        let mut pollfds: Vec<libc::pollfd> = input_fds.iter().map(|&fd| libc::pollfd {
            fd,
            events: libc::POLLIN | libc::POLLHUP,
            revents: 0,
        }).collect();
        loop {
            let n = unsafe { libc::poll(pollfds.as_mut_ptr(), pollfds.len() as _, -1) };
            if n > 0 { break; }
        }
        trigger_shutdown();
        // Phase 2 — propagate shutdown to tier-2 PipeFd consumers by
        // closing the broadcast write-end. All readers POLLHUP.
        unsafe { libc::close(broadcast_w_fd); }
    })
    .expect("wat-shutdown-worker thread spawn failed");
```

Note: keep using raw `libc::close` for `broadcast_w_fd` (not OwnedFd) to match the existing wake_pipe pattern in this fn.

### 2. `src/io.rs` (or wherever `WatReader` trait lives) — expose pollable fd

Find the `WatReader` trait definition. Add a default-None method:

```rust
/// Arc 170 Phase 2 — return the raw FD this reader is backed by,
/// for OS-level multiplex via poll(2). Default `None` for
/// non-FD-backed readers (StringReader, InMemoryReader). Override
/// in PipeReader to return `Some(self.fd.as_raw_fd())`.
fn as_raw_fd_for_poll(&self) -> Option<i32> {
    None
}
```

In `PipeReader` (or whatever the FD-backed impl is called) — override:

```rust
fn as_raw_fd_for_poll(&self) -> Option<i32> {
    Some(self.fd.as_raw_fd())
}
```

If you find the WatReader trait is in a different file than expected, grep `pub trait WatReader` and follow the path.

### 3. `src/typed_channel.rs::typed_recv` PipeFd arm — multiplex on shutdown

Today (line ~324):

```rust
ReceiverInner::PipeFd(reader) => match reader.read_line(span) {
    Ok(Some(line)) => { ... }
    Ok(None) => RecvOutcome::Disconnected,
    Err(_) => RecvOutcome::Disconnected,
},
```

Becomes:

```rust
ReceiverInner::PipeFd(reader) => {
    // Phase 2 — multiplex on shutdown via OS-level poll.
    // If reader exposes a pollable FD AND the substrate's shutdown
    // broadcast is initialized, poll both; otherwise fall back to
    // bare read_line (non-FD-backed reader, or pre-init bootstrap).
    let pipe_fd_opt = reader.as_raw_fd_for_poll();
    let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD.load(
        std::sync::atomic::Ordering::SeqCst,
    );
    if let (Some(pfd), true) = (pipe_fd_opt, broadcast_fd >= 0) {
        loop {
            let mut fds = [
                libc::pollfd {
                    fd: pfd,
                    events: libc::POLLIN | libc::POLLHUP,
                    revents: 0,
                },
                libc::pollfd {
                    fd: broadcast_fd,
                    events: libc::POLLHUP,
                    revents: 0,
                },
            ];
            let n = unsafe { libc::poll(fds.as_mut_ptr(), 2, -1) };
            if n < 0 {
                // EINTR retries; other errors fall through to read_line.
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                break;
            }
            if n == 0 {
                // timeout -1 should never produce n=0; defensively retry.
                continue;
            }
            // Shutdown wins ties per Slice B discipline — process is
            // going down; honest reporting.
            if fds[1].revents != 0 {
                return RecvOutcome::Shutdown;
            }
            if fds[0].revents != 0 {
                break;
            }
        }
    }
    // Pipe is ready (or no multiplex possible). Read.
    match reader.read_line(span) {
        Ok(Some(line)) => {
            let trimmed = line.trim_end_matches('\n');
            match crate::edn_shim::read_edn(trimmed, types) {
                Ok(v) => RecvOutcome::Value(v),
                Err(e) => RecvOutcome::DecodeError(format!("{}", e)),
            }
        }
        Ok(None) => RecvOutcome::Disconnected,
        Err(_) => RecvOutcome::Disconnected,
    }
}
```

### 4. `src/typed_channel.rs::typed_try_recv` PipeFd arm — non-blocking poll

Today (line ~383): `ReceiverInner::PipeFd(_) => RecvOutcome::Disconnected` (the BRIEF's "NOT YET IMPLEMENTED" placeholder).

Update to a `poll(..., timeout=0)` check:

```rust
ReceiverInner::PipeFd(reader) => {
    let pipe_fd_opt = reader.as_raw_fd_for_poll();
    let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD.load(
        std::sync::atomic::Ordering::SeqCst,
    );
    if let (Some(pfd), true) = (pipe_fd_opt, broadcast_fd >= 0) {
        let mut fds = [
            libc::pollfd {
                fd: broadcast_fd,
                events: libc::POLLHUP,
                revents: 0,
            },
            libc::pollfd {
                fd: pfd,
                events: libc::POLLIN | libc::POLLHUP,
                revents: 0,
            },
        ];
        let n = unsafe { libc::poll(fds.as_mut_ptr(), 2, 0) };
        if n > 0 {
            // Shutdown wins ties.
            if fds[0].revents != 0 {
                return RecvOutcome::Shutdown;
            }
            // Pipe ready — fall through to read_line.
        } else {
            // n == 0: timeout (no data, no shutdown). Empty.
            // n < 0: error — return Disconnected as the substrate's
            //   honest "I have no data" signal.
            return RecvOutcome::Disconnected;
        }
    } else {
        // No multiplex possible — preserve old behavior.
        return RecvOutcome::Disconnected;
    }
    // pipe ready — try one read.
    match reader.read_line(span) {
        Ok(Some(line)) => {
            let trimmed = line.trim_end_matches('\n');
            match crate::edn_shim::read_edn(trimmed, types) {
                Ok(v) => RecvOutcome::Value(v),
                Err(e) => RecvOutcome::DecodeError(format!("{}", e)),
            }
        }
        Ok(None) => RecvOutcome::Disconnected,
        Err(_) => RecvOutcome::Disconnected,
    }
}
```

Also update the doc comment at line ~352-358 (`"NOT YET IMPLEMENTED — pipe fds are blocking..."`) to reflect Phase 2's implementation. Note that `try-recv` on PipeFd is now correctly non-blocking via poll(timeout=0); the broadcast fd is checked alongside.

### 5. NEW probe: `tests/probe_shutdown_cascade_pipefd.rs`

Mirror `tests/probe_shutdown_cascade_crossbeam.rs` shape. Child-isolated fork; child creates a `Receiver/from-pipe` wrapping a NEVER-WRITTEN pipe; child blocks on `typed_recv`; after a known delay, main raises SIGTERM; assert child's recv wakes with `RecvOutcome::Shutdown` within 100ms.

The pipe-wrapping setup:
- Parent process (the test) creates a pipe pair via `libc::pipe`.
- Wrap the read-end in a `PipeReader` and then a `Receiver/from-pipe` Value.
- Spawn a child process via `run_in_fork` (or whatever the existing pattern in `probe_shutdown_cascade_crossbeam` uses).
- Child blocks on `typed_recv(receiver, ...)`.
- Parent (test) raises SIGTERM (e.g., `kill(child_pid, SIGTERM)`).
- Assert child's recv returns `RecvOutcome::Shutdown` within 100ms via the child's `done_pipe` POLLHUP rendezvous.

Test name: `probe_shutdown_cascade_pipefd_wakes_pipe_recv` (or whatever fits the existing naming pattern).

## Scorecard (10 rows)

| Row | What | Evidence |
|-----|------|----------|
| A | `SHUTDOWN_BROADCAST_READ_FD: AtomicI32` global exists in src/runtime.rs | `grep -n "SHUTDOWN_BROADCAST_READ_FD" src/runtime.rs` shows the static + init store |
| B | `init_shutdown_signal_with_inputs` creates a second pipe + stores broadcast_r_fd | grep shows the pipe creation + atomic store |
| C | Worker closure holds broadcast_w_fd; closes it AFTER trigger_shutdown() | grep + read worker block; close call appears after trigger_shutdown |
| D | `WatReader` trait gains `as_raw_fd_for_poll() -> Option<i32>` with `None` default | `grep -n "as_raw_fd_for_poll" src/io.rs` (or wherever trait lives) |
| E | `PipeReader` overrides `as_raw_fd_for_poll` to return `Some(fd)` | grep shows the override |
| F | `typed_recv` PipeFd arm: poll(2) on (pipe_fd, broadcast_fd); POLLHUP on broadcast_fd → Shutdown | `awk '/ReceiverInner::PipeFd\(reader\) =>/,/^        \},$/' src/typed_channel.rs` shows the multiplex + Shutdown return |
| G | `typed_try_recv` PipeFd arm: poll(timeout=0) on (broadcast_fd, pipe_fd); Shutdown if broadcast ready | similar grep for typed_try_recv |
| H | NEW probe `tests/probe_shutdown_cascade_pipefd.rs` exists; routes through `Receiver/from-pipe` + raises SIGTERM | `ls` + grep |
| I | `cargo build --release --workspace --tests` clean | build output |
| J | NEW probe PASSES 1/1 + all 5 existing probes still pass | invoke each in isolation |

## Constraints

- NO Mutex / RwLock / CondVar.
- NO new wall-clock timers; use libc::poll(2) — timeout=-1 for blocking, timeout=0 for non-blocking try-recv.
- NO changes outside the named files.
- DO NOT touch tests/probe_pdeathsig_kills_orphan_child.rs (historical artifact).
- DO NOT touch tests/probe_lifeline_orphan_clean_via_*.rs (Phase 1 probes).
- DO NOT modify Slice B's `typed_recv` crossbeam arm or `probe_shutdown_cascade_crossbeam.rs`.

## STOP-at-first-red

- `cargo build` fails → STOP, report. Likely cause: WatReader trait method signature mismatch with an existing impl; surface and we'll add the default.
- New probe panics → STOP. Use /proc snapshot of any stuck procs to root-cause.
- Phase 1 probes regress → STOP. The broadcast pipe should be additive; if it breaks lifeline detection that's a substrate ordering bug.

## On completion

Write `SCORE-FD-MULTIPLEX-PHASE-2-PIPEFD-SHUTDOWN.md`. 10 rows. Note honest deltas — especially: WatReader trait location, PipeReader file location, any non-FD-backed readers that need the default None to behave correctly.

Do NOT commit. Orchestrator commits atomically after independent verification.
