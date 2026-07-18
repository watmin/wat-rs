# DESIGN — the RST: best-effort peer-crash notification (wat's mini-TCP reset)

> **Builder, this session:** *"let's best effort notify all peers that we're crashing… we do not wait on their
> ack to continue… this is the 'RST' of our 'tcp'."* Stays in **278** (a tail, not a new arc). Honors the
> arc-294 ruling (*a crash reason is administrative — to the creator, never blind callers*): the **reason** goes
> to the **owner** (existing crash channel); a **reason-free reset** goes to connected **peers**.
>
> **Naming ruling (intueri cast):** the reader-facing variant is **`RecvError::PeerCrashed`**, NOT `Reset` —
> `Reset` borrows TCP lore a wat reader doesn't hold; `PeerCrashed` names the reality. *"It's RST in nature —
> it's communicating a peer that has crashed in reality."* The RST / FIN / mini-TCP framing below stays as an
> **implementer aside** (for whoever knows TCP) — it is never the variant name.

## The model (grounded in ZERO-MUTEX.md's mini-TCP)

Normal comms are **mini-TCP**: reliable, acked, bounded(1) backpressure. Two teardown shapes, named by what
actually happened rather than by TCP metaphor — **clean vs. abnormal, far-side vs. near-side**:

| shape | what it is | peer sees | reason? |
|---|---|---|---|
| clean far-side close (*aside: TCP calls this FIN* — graceful `/stop`) | the loop exits cleanly; senders dropped | `RecvError::Disconnected` (clean EOF — done by the transport twin) | n/a |
| abnormal far-side crash (*aside: TCP calls this RST* — handler panic → structured-exit) | the loop is dying | **NEW: `RecvError::PeerCrashed`** | NO — reason is the owner's (admin channel) |

The full `RecvError` family, by the same clean-vs-abnormal / far-side-vs-near-side axis:
- `Disconnected` — clean far-side close (peer closed the write-end with no error).
- `PeerCrashed` — abnormal far-side crash (the peer died from an unhandled panic; no reason — administrative,
  owner-only).
- `Shutdown` — the substrate shutdown cascade fired (neither side "closed"; the whole substrate is tearing
  down).
- `Failed(String)` — a near-side (our own receive-side) wire error: io failure, invalid UTF-8, undecodable
  frame.

The crash notification is legitimately **fire-and-forget by nature** — NOT the lazy fire-and-forget arc-119
retired: a dying process *cannot* wait for acks. Like a TCP RST (*aside*), it is a best-effort last word; if a
peer misses it, that is acceptable (best-effort, no retransmit, no ack).

## The contract (pinned)

- A crashing service **best-effort broadcasts a crash notification to every connected peer** (the serve
  loop's `clients` Vector — `service.wat`) **before it dies**, then lets-it-crash (the existing
  `eprintln`-terminal → `emit_structured_exit` path, `process/verbs.rs`). **No ack; no wait; no hang** — a
  send that fails (peer already gone) is skipped.
- **Peer side:** the crash surfaces through the peer's `recv'` as a distinct, catchable signal —
  `RecvError::PeerCrashed` — DISTINCT from `Disconnected` (clean far-side close), `Failed(reason)` (near-side
  wire error, the twin), `Shutdown`, `FrameTooLarge`. It carries **no reason** (a peer learns "the far side
  crashed," not why).
- **Owner side:** unchanged — the `/start` handle still gets the crash reason via the crash channel
  (`PeerRecvError::Crashed`). `PeerCrashed` does not touch that.
- **Graceful `/stop`** stays the clean far-side close (`Disconnected`) — honest as-is; a *graceful*
  peer-notification is a separate small add if wanted, NOT this stone (the builder's focus is the crash
  notification).

## The disconfirming probe (FIRST — the load-bearing unknown)

Can the serve loop's crash arm reach `clients` and best-effort-send a crash-notification control frame
**before** the `panic_any`/`emit_structured_exit` path exits? Two sub-questions:
1. Does a handler `panic_any` unwind to a point where the serve loop still holds `clients` and can broadcast,
   or does it go straight to structured-exit past the loop? (If past — the notification must hook where
   `clients` is reachable: thread the peer set to `emit_structured_exit`, or broadcast in the serve loop's
   catch before re-raising.)
2. Can a distinct control frame be sent on a peer tx and recognized by the receiver's frame path as the crash
   notification (→ `RecvError::PeerCrashed`), separably from a data frame and from EOF?

Write a 10–15 line probe that reproduces exactly this (a process service, a handler that genuinely panics, a
connected peer reading) and prove the current behavior (peer sees clean-EOF `Disconnected` — no crash signal)
before building. **STOP if the panic boundary structurally forecloses reaching `clients`** — surface the
exact mechanism (that reshapes where the notification hooks); do not guess.

**STOP-1 was hit, then reshaped by the builder into Option A** (below) — the FIRST pass found the panic
boundary forecloses reaching `clients` from every EXISTING `catch_unwind` site (`finish_forked_child`,
`spawn_thread_peer`): those sites only ever see the panic AFTER the whole `serve` recursion — and its
`clients` binding — has already unwound past them (confirmed: `apply_function` builds `clients`' `Environment`
binding *inside its own stack frame*; an outside-the-call `catch_unwind` never sees it). The builder's
redirect: a NEW `catch_unwind` can be inserted, not at an existing top-level site, but around the evaluation
of the serve loop's op-dispatch ITSELF — reachable via a native kernel primitive (below), which is where
`clients` genuinely is still in scope.

## Mechanism, as built — Option A: `serve-dispatch-op'`

`defservice`'s codegen (`wat/service.wat`'s `Message idx op` arm) wraps the op-dispatch match in a new native
primitive instead of evaluating it bare:

```wat
((:wat::spawn::ServiceEvent::Message idx op)
  (:wat::kernel::serve-dispatch-op' clients
    (:wat::core::match op -> :wat::core::nil
      ~@serve-op-arms)))
```

`:wat::kernel::serve-dispatch-op'` (`src/runtime.rs::eval_kernel_serve_dispatch_op_tail`, the tail-position
twin registered alongside `:wat::core::match` in `eval_tail`'s dispatch — REQUIRED to preserve `serve`'s
self-recursion trampoline through the wrapper) evaluates `clients` and `body` in its OWN Rust stack frame, then
wraps `body`'s evaluation in `std::panic::catch_unwind`:

- **`Ok(result)`** (no panic — the ordinary `Outcome::Reply`/`Outcome::Stop` path, INCLUDING a
  `EvalSignal::TailCall` for `serve`'s own recursion, which is a normal returned value, not a panic) passes
  through unchanged.
- **`Err(payload)`** (a genuine handler panic): best-effort broadcasts the `PeerCrashed` sentinel to every peer
  in `clients` (`kernel::peer::broadcast_peer_crashed_best_effort`), then
  `std::panic::resume_unwind(payload)`s the ORIGINAL, untouched panic — reaching the EXACT SAME
  `finish_forked_child`/`spawn_thread_peer` catch sites as before, byte-identical (same exit code, same owner
  crash reason via `PeerRecvError::Crashed`). This is why the owner-reason path needed ZERO changes.

**The frame convention** (question 2): there is no separate control channel at either transport tier — the
notification rides the peer's EXISTING data channel. A reserved sentinel keyword
(`:wat::kernel::__peer_crashed__`, `kernel::peer::PEER_CRASHED_SENTINEL`) is sent via a NEW, genuinely
non-blocking `CommSender::try_send` (crossbeam's native `try_send` for the thread tier; an `O_NONBLOCK`-toggled
write for the process tier — `EWOULDBLOCK`/a full pipe is treated as a skip, never a block) and recognized by
`kernel::peer::Peer::recv`/`recv_wire` BEFORE/instead of ordinary decode — an exact string compare against the
reserved wire text, never a decode attempt, so a malformed frame can never be confused with it. Reserved under
`:wat::kernel::` — never constructible from user wat source.

**Known limitation (best-effort, honestly so):** a bystander peer that has NEVER sent or received anything on
its connection, whose crash notification is written immediately before the crashing process's abrupt `_exit`,
can race a Unix-domain-socket-level `ECONNRESET` at the OS layer and see a raw `RecvError::Failed("io_uring
read failed")` instead of the clean `PeerCrashed` signal — the write succeeds, but the kernel can still discard
the unread bytes on an abrupt peer close. This is WITHIN the design's own "best-effort... if a peer misses it,
that is acceptable" contract (never a mute clean-`Disconnected` lie, never a hang) but is NOT a 100%-delivery
guarantee to every live peer. The peer whose OWN request triggered the panic (the primary, most common case —
it has necessarily already interacted with the connection) reliably sees `PeerCrashed`; a never-touched
bystander connection may not. Not chased further inside this stone (fixing it would mean either violating
"no wait" or deeper transport-layer work outside this stone's blast radius).

## RED gate (acceptance)

A process service whose handler **genuinely panics** → the peer whose own request triggered the panic sees
`recv'` surface `RecvError::PeerCrashed`, NOT a bare clean-EOF `Disconnected` (verified:
`tests/services/probe_arc278_rst_peer_notify_baseline.rs`); the service still dies; the **owner still
gets the crash reason** on the admin channel (unchanged — verified by construction: the resumed panic is
byte-identical to the pre-stone panic); **best-effort** — no ack, no hang (a peer that already left, or that
races the OS-level close as described above, is skipped, never blocked on). No-regression: a genuine clean
close still surfaces `Disconnected` (structurally unchanged — `serve-dispatch-op'` wraps ONLY the `Message`
arm, never `Admin::Stop`/`Closed`/`Lost`/`Connection`/`Malformed`); `dead_child_speaks`,
`probe_arc272_rs2_*`, `probe_arc259_thread_crash_reason`, the transport-twin probes all stay green (245/245 in
the scoped `comms + channel + process + services` nextest gate); full suite green by the orchestrator's own
re-run.

## Blast radius (as built)

`src/comms/mod.rs` (`RecvError::PeerCrashed` variant + Display; `CommSender::try_send`), `src/comms/thread.rs`
+ `src/comms/process.rs` (`try_send` impls — crossbeam native / `O_NONBLOCK`-toggle), `src/kernel/peer.rs`
(the sentinel, `Peer::recv`/`recv_wire` interception, `notify_peer_crashed_best_effort`,
`broadcast_peer_crashed_best_effort`), `src/runtime.rs` (`serve-dispatch-op'`'s tail + non-tail eval arms,
`eval_peer_recv_prime`'s `PeerCrashed` mapping), `src/check.rs` (`infer_serve_dispatch_op` — do-style
passthrough type-check), `src/channel/transfer.rs` (exhaustive-match cascade — `PeerCrashed` is unreachable on
a bare `:wat::kernel::Sender<T>`/`Receiver<T>` channel; surfaced loudly rather than silently folded into
`Disconnected`, per arc 278 no-hidden-failures), `wat/service.wat` (the `Message idx op` arm's codegen). NOT
touched: `emit_structured_exit`/`finish_forked_child`/`spawn_thread_peer` (the owner-reason path) — the whole
point of the `serve-dispatch-op'` hook is that it never needed to be.
