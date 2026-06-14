# EXPECTATIONS — Stone C0b.3a-ii (written before the strike)

Independent scorecard. The Inquisitor verifies each row by its own re-run before any
commit. The deadlock-surface stone — the gate probe both proves the capability AND the
termination-on-owner-drop (a hang = the deadlock, not a pass).

| # | What | Command | Expected |
|---|------|---------|----------|
| 1 | The process service loop works + terminates | `cargo test --release -p wat --test probe_arc209_c0b3aii_process_service_loop -- --test-threads=1` | `1 passed` (returns 105 AND does not hang — GREEN ⟹ serve + clean termination) |
| 2 | Thread `poll'` UNCHANGED | `cargo test --release -p wat --test nursery probe_arc209_c0b1b -- --test-threads=1` | `1 passed` |
| 3 | 1-arg `select'` unchanged | `cargo test --release -p wat --test nursery connection_primitive -- --test-threads=1` | `1 passed` |
| 4 | cross-process connect-by-name intact | `cargo test --release -p wat --test probe_arc209_c0b2d_named_cross_process` | `1 passed` |
| 5 | `CommListener::reactor_class` is the NAMED enum | read `src/kernel/listener.rs` | `reactor_class() -> ReactorClass` (`InMemory`/`Fd`); NO `Option<RawFd>`/`listen_fd` |
| 6 | termination is structural | read `eval_poll_prime` process path | self-peer = Select index 0; `Recv{0}` → `ServiceEvent::Shutdown` (result NOT inspected); NO cooperative Stop / shutdown channel / sleep |
| 7 | Nursery baseline holds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (4 known: arc-255 reflection ×2 + undefined-builtin ×2 — ZERO new) |
| 8 | Full surface compiles | `cargo test --release --workspace --no-run` | clean |

## Runtime prediction

15–30 min. `reactor_class` on `CommListener` (small) + the `eval_poll_prime` process branch
(the meat — mirror the thread template over `process::Select`) + a recompile cascade. The
gate probe forks a process; runs in seconds when correct.

## Trap-doors named (the deadlock surface)

- **STOP-1 — the gate probe HANGS:** the loop didn't terminate on owner-drop. This is THE
  deadlock. Capture forensics (gdb bt + `/proc/<pid>/task/*/{stack,wchan}`); the fix is
  structural (self-peer `Recv{0}` EOF = the wake), NEVER a cooperative Stop (races the drop).
  A hang is a FAIL, not a slow pass — do not wait it out.
- **Index mapping:** process path is self-peer=0, clients=1..N (recv), listener=accept-arm —
  so `Message.idx = index-1` (NOT `index-2` like the thread path where the listener is recv
  index 1). Off-by-one here = wrong client gets the reply.
- **`as_any` None:** if a peer/self-peer receiver doesn't downcast to `process::Receiver<Value>`
  in the Fd branch, the reactor_class grouping is wrong — investigate, don't paper over.
- **Scope creep:** any `Address'`/`SocketAddress'` change (= iii), SO_PEERCRED (= C0b.3b),
  `comms` trait change, or thread-path edit is OUT — `git diff --stat` confined to
  listener.rs + runtime.rs (+ the probe).

## Honest-delta slots (filled at SCORE time)

- Did the process path mirror the thread template cleanly, or did `process::Select`
  (recv + listener-arm in one ring) force structural changes? —
- Did termination-on-drop work first try, or did STOP-1 fire? —
- Any baseline drift in rows 2–7? Diff stat? —
