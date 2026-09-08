# DESIGN — the sender grows a ring

> Added to arc 170 **after its INSCRIPTION**, at the builder's ruling. Arc 170 owns the FD-multiplex
> shutdown machinery (`DESIGN-FD-MULTIPLEX-SHUTDOWN.md`); this completes its write side. Named so it
> does not read as one of 170's original slices.

## The defect, visible in two struct definitions

```rust
pub struct Sender<T>   { write_fd: OwnedFd, _phantom }              // no ring
pub struct Receiver<T> { source, accumulator, max_frame_bytes,
                         ring: RefCell<IoUring>,                    // capacity 4
                         _phantom }
```

★★★ **The `Sender` has no ring *because* its write blocked.** A blocking call cannot be multiplexed,
so nobody ever gave it a reactor — and that blocking call was the bug this arc's descendant found.
The missing ring is not a second defect beside the first; it is **the same defect, written into the
type.** `comms/process.rs` line 10 says it in prose — *"newline-framed bytes → `libc::write` →
io_uring Read"* — and the structs say it in code.

Stone 1 made the **wait** honest: `O_NONBLOCK`, and the poll owns it. This makes the **mechanism**
symmetric. After it, `Sender` and `Receiver` are the same kind of object.

## The mechanism

Per write attempt, on the Sender's own ring:

```
push Write(framed[written..], len − written)   user_data = WRITE
push PollAdd(broadcast, POLLIN|POLLHUP)        user_data = BROADCAST
submit_and_wait(1)                             EINTR → retry (mirrors process.rs:1315-1320)

Write completes   n > 0  → written += n; loop
                  n < 0  → errno mapped exactly as today (EPIPE → Disconnected, …)
Broadcast first          → the Write is parked. AsyncCancel(WRITE), drain, Err(Shutdown(value))
```

## Why this is safe — the probes proved the LOOP, not just the call

Four measurement stones stand behind this, each re-run and graded independently:

| regime | measured behaviour |
|---|---|
| room `== 0` | the Write **parks having delivered nothing**; `AsyncCancel` → `ECANCELED`, 0 delivered |
| room `> 0` | the Write **completes immediately with the short count** — at every payload from 8192 to 131072 (2× pipe capacity) |

★★ **Therefore there is no state in which this loop cancels a partially-delivered write.** Either the
write made progress and reported it, or it made none and can be cancelled cleanly. The byte count is
never lost, so `written += n` remains correct. That is the whole reason this stone is drawable.

## ★ The tie-break stops being a comment and becomes a property

Today, `comms/process.rs:435-448`:

```rust
if fds[0].revents != 0 { break; }                       // "Writable wins ties …
if fds[1].revents != 0 { return Err(SendError::Shutdown(value)); }   //  a dying process must
                                                        //  still be able to utter its last words."
```

That ordering is a **hand-written branch guarding a hand-written comment** — exactly the shape that
hid the original bug, where two files wrote down their own safety in prose.

With `Write` and `PollAdd` on one ring, the ordering is no longer written anywhere: **if the write
can make progress the kernel completes it; only when it would block does the broadcast win.** The
documented tie-break becomes an emergent property of the multiplexer. Convention → check → **a shape
the mistake cannot be written down in.**

## ⛔ The trap that would silently invalidate every measurement

**All four probes measured a BLOCKING fd.** `fill_until_eagain` sets `O_NONBLOCK` and then **restores
the original flags** before the ring submission.

If the `Sender` keeps stone 1's `NonblockGuard`, an io_uring `Write` on a full pipe completes with
**`-EAGAIN` instead of parking** — a different mechanism, and none of the measured behaviour above
transfers. **The guard must go when the write joins the ring**, and its departure is part of this
stone, not a follow-up.

## The one contract decision

**`SendError`'s variants and their meanings do not change.** `Shutdown(value)` still returns the
unsent value; `Disconnected`, `Failed` keep their errno mapping and their strings. A caller cannot
tell this happened except that the process now stops when told to.

## Out of scope — REJECTED, not deferred

- **`src/io.rs`.** Four questions: *Obvious?* NO — it owns no ring, and `PipeWriter`/`PipeReader` are
  `:ephemeral` wat resources with no evident ring owner. *Simple?* NO — two changes wearing one hat.
  It is correct today under stone 1. Its own stone, once ring-ownership has an answer.
- **`try_send`** (`:534-576`). Non-blocking by contract; there is no wait to multiplex.
- **`signalfd`.** The next stone, `a signal is an fd`. It retires the handler and the wake pipe —
  **not** the broadcast, since parent-death (`LIFELINE_FD`) and in-process `trigger_shutdown()` are
  not signals.
- **`Sender: Clone`.** It deliberately does not exist (single-writer; `PIPE_BUF` atomicity). Adding a
  ring must not create one.

## Files

`src/comms/process.rs` only: the `Sender` struct, its four construction sites (`:614`, `:2055`,
`:2108`, `:2132`), and `send`. Plus the header at line 10, which stops being true.
