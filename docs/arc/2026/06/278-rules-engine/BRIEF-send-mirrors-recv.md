# BRIEF — `send` is `recv` in the opposite direction

**Stone:** `DESIGN-STONE-send-mirrors-recv.md` — fully ruled, nothing open.
**Floor at brief time:** `4343 tests run: 4343 passed, 262 skipped`, established by the
orchestrator's own `--release` re-run.

## The work, in one paragraph

`RecvError` is a five-variant enum; `SendError<T>` is a newtype. Recv got the rich error, the
shutdown poll, and the frame cap; send got none of them — because arc 170's IPC work was done
recv-first and nobody ever reads a send failure (all six `SendOutcome::Lost` consumers in the
stdlib bind `_c` and discard it). **Make send the mirror**: `SendError` becomes an enum, the
process sender gains the frame budget it is currently constructed without, and its write loop gains
the shutdown poll `PipeWriter::write` already has.

## Read in order

1. **`src/comms/mod.rs:940`** — `RecvError`. **This is your template**; you are giving send the same
   shape. Read its `Disconnected` doc comment in particular — it states the law this work restores.
2. **`src/comms/mod.rs:914`** — `pub struct SendError<T>(pub T);`, the thing being replaced. And
   **`:916`** — the recorded rationale for keeping it thin (*"a blocking send never returns 'full',
   it just waits"*). Correct about *full*, wrong as a general conclusion. Leave that comment
   corrected, not deleted.
3. **`src/comms/process.rs:311`** — `pub struct Sender<T> { write_fd, _phantom }`. **No budget
   field**; it needs one.
4. **`src/comms/process.rs:1915`** — `pair_with_budget(max_frame_bytes)`. The number is **already in
   scope**: it is handed to `Receiver { …, max_frame_bytes, … }` and the `Sender` is constructed
   three lines later without it. Same for **`:1977`** `sender_receiver_from_fd_with_budget` and
   **`:2006`** `sender_receiver_from_split_fds`.
5. **`src/comms/process.rs:333`** — `Sender::send`. The write loop that needs both the cap check and
   the poll arm.
6. **`src/io.rs:634`** — `PipeWriter::write`. **Copy this poll.** Its own comment at `:645` names the
   shape: `[fd → POLLOUT, SHUTDOWN_BROADCAST_READ_FD → POLLIN|POLLHUP]`.
7. **`src/channel/transfer.rs:173-199`** — the read-side twin, showing how `SHUTDOWN_BROADCAST_READ_FD`
   is loaded and why the broadcast arm watches **both** `POLLIN` (a written byte = WAKE) and
   `POLLHUP` (the drop = SEVER).
8. **`src/comms/thread.rs:120`** and **`src/comms/process.rs:522`** — the two `CommSender` impls that
   must produce the new enum.
9. **`src/runtime.rs`** — `send_outcome_lost` / `try_send_outcome_lost` and their call sites. These
   already take a `LociDiedError` cause (landed at `43673225`); they now map the new variants
   through instead of always passing `Disconnected`.

## The mirror — three holes, one named non-mirror

| `RecvError` | send meaning | do |
|---|---|---|
| `Disconnected` | EPIPE, the reader is gone | already have |
| `Shutdown` | the broadcast fired mid-write | **build** — the poll arm |
| `Failed(String)` | an io error, with its reason | **build** — stop discarding it |
| `FrameTooLarge` | the outgoing frame exceeds the cap | **build** — budget field + pre-write check |
| `PeerCrashed` | the peer died rather than closed | **do not mirror** — record the reason at the type |

## Sketch

```rust
pub enum SendError<T> {
    Disconnected(T),
    Shutdown(T),
    FrameTooLarge(T),
    Failed(T, String),
    // PeerCrashed: not mirrored — a sender has no crash-channel access at this
    // layer and sees a dead peer as EPIPE, which is honestly Disconnected.
}
```

`Sender::send`, in order: build the frame → **if over budget, return `FrameTooLarge(value)` before
touching the fd** → then the write loop, each iteration polling `[fd → POLLOUT, broadcast]` and
returning `Shutdown(value)` when the broadcast arm fires.

## ⛔ STOPs — ship nothing, surface the gap

- **⛔ STOP-1 — do NOT make the send stop blocking.** The reactor is lockstep, blocking, size-1
  channels with real backpressure; a write that stalls until the reader drains **is** the mechanism.
  `PipeWriter::write` still blocks — it blocks *wakeably*. If your change makes a send return early
  where it used to wait, you have broken backpressure. STOP.
- **⛔ STOP-2 — do NOT size the pipe to the frame budget.** The 512 KiB-frame-over-64 KiB-pipe
  mismatch is a deliberate layer-7/layer-2 decoupling so subsystems never learn the pipe size.
  `F_SETPIPE_SZ` is out of scope and forbidden.
- **⛔ STOP-3 — do NOT delete the receiver's `FrameTooLarge` dismissal.** Belt AND braces: the write
  check keeps a well-behaved client inside the limit; the receiver's teardown still defends against
  anything that bypasses our patterns.
- **⛔ STOP-4 — do NOT omit `PeerCrashed` silently.** Mirror it or write the reason at the type.
  Silence is exactly what produced the three holes you are closing.
- **⛔ STOP-5 — do NOT trust a grep for the site list.** Change the type and let the compiler
  enumerate. Report the number **the compiler** gave you and say that is where it came from.
- **⛔ STOP-6 — if a `CommSender` impl cannot honestly produce a variant, STOP and name it.** Do not
  fabricate a mapping to make the enum look complete. (A prior rider correctly refused to invent a
  `Stopped` mapping on this same path; that refusal is why this stone exists.)

## ★ THE DELIBERATE BREAK — two rows, and they are different

Both must be run, both restored byte-exact, both reported with real output.

1. **The poll arm.** Remove the broadcast arm from `Sender::send`'s poll (leave `POLLOUT`), rebuild,
   and run whatever test proves a blocked send wakes. Confirm RED. If nothing goes red, **the test
   does not depend on the mechanism** — say so plainly; that is a finding about the gate, not a
   pass.
2. **The cap check.** Raise the pre-write budget test to `usize::MAX` so it can never fire, rebuild,
   and confirm the over-cap case goes RED. Same rule: if nothing reddens, report it.

## Done means

- `SendError` is an enum; both `CommSender` impls produce it; `PeerCrashed`'s absence is documented
  at the type.
- `Sender` carries `max_frame_bytes`, threaded from all three construction sites.
- An over-budget send returns `FrameTooLarge` **without writing to the fd**.
- A blocked send wakes on the shutdown broadcast and returns `Shutdown`, and **still blocks**
  otherwise.
- `send_outcome_lost` maps `Shutdown → LociDiedError::Stopped` and carries `Failed`'s reason.
- Both breaks went RED and both restores went green — reported with actual output.
- `cargo nextest run --release` **Summary line verbatim** against `4343/4343/0/262`, count
  arithmetic explained.
- `cargo clippy --release --all-targets` clean.
- Every STOP hit, named. If none, say so explicitly.

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you and no
notification is coming. Run every verification in the FOREGROUND and block on it: your turn ends
when the numbers are in your hands, not when a command is launched. Do not commit, do not push, do
not stash, do not revert anything you did not write. If a verification is incomplete when you finish
writing, say so plainly rather than yielding on it.
