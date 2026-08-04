# DESIGN-STONE — `send` is `recv` in the opposite direction

> **Status: RULED 2026-08-03, unbuilt.** Builder: *"i see no reason why send shouldn't be a full
> mirror of recv?.... they are the same in opposite directions."*
>
> **This stone consolidates three findings into one type change.** They were filed separately
> before the ruling; they are not three stones. They are one missing enum and the two mechanisms
> that fill it.

## How we got here

The session opened on the arc-170 signal races. Fixing them meant spawning real children and
signalling them — which put us back on the IPC teardown path. Every time we looked at that path,
**send was poorer than recv**. Three times, in three different ways, over one afternoon.

The reason is structural, not accidental: **arc 170's IPC work was done recv-first.** Recv got the
rich error enum, the shutdown poll, the frame cap, and the cause threading. Send got a newtype and
a `libc::write` loop. And nobody noticed, because **nobody reads a send failure** — all six
`SendOutcome::Lost` consumers in the stdlib bind `_c` and discard it.

## The two types today

```rust
pub enum RecvError { Disconnected, Shutdown, FrameTooLarge, Failed(String), PeerCrashed }
pub struct SendError<T>(pub T);
```

`RecvError` documents the law in its own doc comment:

> *"NEVER produced for a raw transport failure … those carry a reason via `RecvError::Failed`
> instead, per the arc 278 no-hidden-failures law. **Mute-collapsing a real error into
> `Disconnected` is exactly the mislabeling this variant's contract forbids.**"*

`SendError` is that mislabeling, structurally. It has one shape, so every failure becomes it.

**And the rationale for keeping it thin is on the record, at `comms/mod.rs:916`:**

> *"unlike `SendError` (which is used only by the blocking send, where the distinction is moot — a
> blocking send never returns 'full', it just waits)"*

Correct about *full*, and read as covering everything. A blocking send has no "full" case — but
`Shutdown`, a real io error, and an over-cap frame are three distinctions it needs. **One axis was
checked and the conclusion was drawn for all of them.**

## ★ THE MIRROR, APPLIED AS A TEST

The ruling is not only a fix — it is an **instrument**. Enumerate one side's states; demand a
meaning for each on the other. Every variant without a twin is either a real difference you can
*name*, or a hole.

| `RecvError` | the send meaning | today |
|---|---|---|
| `Disconnected` | EPIPE — the reader's end is gone | ✅ the only thing send has |
| `Shutdown` | the broadcast fired mid-write | ❌ **hole** — no poll arm; see § the poll |
| `Failed(String)` | an io error, carrying its reason | ❌ **hole** — `Err(_)` discards it |
| `FrameTooLarge` | the **outgoing** frame exceeds the cap | ❌ **hole** — read-checked, write-unchecked |
| `PeerCrashed` | the peer died rather than closed | **?** — see below; likely a real difference |

**Three of five were holes.** Two hours of examining the send path through a wake-shaped lens found
one of them. The mirror found the other two in minutes — including `FrameTooLarge`, which fails
*silently*, and which no amount of thinking about shutdown would have surfaced.

`PeerCrashed` is the honest maybe. On recv it comes off the crash channel. A sender sees a dead peer
as EPIPE and has no crash-channel access at the `comms::Sender` layer, so it plausibly collapses
into `Disconnected`. **Name that reason in the code or mirror it — do not simply omit it.** That is
the whole point of the gate below.

## The strike — one type, then the two mechanisms that fill it

**1. `SendError<T>` becomes an enum mirroring `RecvError`**, carrying the undelivered `T` on every
arm so the existing recover-or-resend contract survives:

```rust
pub enum SendError<T> {
    Disconnected(T),
    Shutdown(T),
    FrameTooLarge(T),
    Failed(T, String),
    // PeerCrashed — mirror it, or record why it collapses into Disconnected.
}
```

Two `CommSender` impls must produce it: `comms/thread.rs:120` and `comms/process.rs:522`.

**2. The poll arm** — `comms::process::Sender::send`'s write loop gains
`[fd → POLLOUT, SHUTDOWN_BROADCAST_READ_FD → POLLIN|POLLHUP]`, exactly as `PipeWriter::write`
already has it (`src/io.rs`, arc 170 closure #5), returning `SendError::Shutdown(value)`.

> ⛔ **THE BLOCKING IS NOT THE BUG.** The reactor is lockstep, blocking, size-1 channels with real
> backpressure — a write that stalls until the reader drains *is* the mechanism. And the
> 512KiB-frame-over-64KiB-pipe mismatch is **deliberate**: the builder chose layer-7 frames larger
> than layer-2 frames so the plumbing absorbs fragmentation and subsystems never learn the pipe
> size. **Do not "fix" either.** The defect is that a blocked write is *un-cancellable* — it cannot
> observe a stop. `PipeWriter::write` still blocks; it just blocks **wakeably**. That is the port.
>
> *(Two earlier framings of mine, withdrawn and kept visible: that the block was the bug, and that
> the substrate should `F_SETPIPE_SZ` the pipe up to match the budget — which is precisely the
> coupling the design avoids.)*

**3. The cap check** — `Sender::send` tests the framed length before the write loop and returns the
send-side `FrameTooLarge`. Every `DEFAULT_MAX_FRAME_BYTES` reference in `src/comms/` today is on
the **receiver** side; `Sender::send`'s body checks nothing.

**4. `send_outcome_lost` maps the enum through** to the `LociDiedError` carrier already widened:
`Shutdown → Stopped`, `Disconnected → Disconnected`, `Failed → the real reason`.

## ★ How the class stops recurring — a gate, not a habit

"Remember to check pairs" is rung one and it rots. The in-tree precedent is
`tests/lint/unused_span_justified.rs`, built after a hand-audit mis-classified three times.

**The gate:** enumerate `RecvError`'s variants; assert each has a `SendError` counterpart **or** a
co-located written reason it does not mirror. Add a variant to either side and the build goes red
until you mirror it or justify it. That is what stops the next person extending recv and silently
leaving send behind — which is exactly what happened here.

## Scale

35 `SendError` mentions across 7 files (`comms/{mod,thread,process}.rs`, `kernel/peer.rs`,
`channel/transfer.rs`, `runtime.rs`, `types.rs`); 2 `CommSender` impls. **One strike.** Change the
type and let the compiler produce the worklist — do not carry a grep count into the brief.

## STOPs

- **⛔ Do not make the send stop blocking.** Backpressure is the design. Make it wakeable.
- **⛔ Do not size the pipe to the frame budget.** The mismatch is a deliberate decoupling.
- **⛔ Do not omit `PeerCrashed`.** Mirror it or record the reason; silence is what got us here.
- **⛔ Do not trust a grep for the site list.** Change the type; read the compiler.
- **⛔ Do not fold in the service-layer budget.** That is the in-flight per-op `:max-request-bytes`
  work; this is the transport floor beneath it. See the open question.

## Open — the builder's

**Is the transport-level frame check wanted, given the service-layer per-op budget is already in
flight?** They are not duplicates — the service layer gives a caller a typed refusal for a declared
op; the transport floor stops the wire emitting a frame its own reader is contractually obliged to
reject. Defence in depth, or redundant? **Unmeasured either way:** whether any live path can reach
`Sender::send` with an over-cap frame has not been proven, and it should be probed before this arm
is built.
