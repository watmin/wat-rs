# DESIGN — the RST: best-effort peer-crash notification (wat's mini-TCP reset)

> **Builder, this session:** *"let's best effort notify all peers that we're crashing… we do not wait on their
> ack to continue… this is the 'RST' of our 'tcp'."* Stays in **278** (a tail, not a new arc). Honors the
> arc-294 ruling (*a crash reason is administrative — to the creator, never blind callers*): the **reason** goes
> to the **owner** (existing crash channel); a **reason-free reset** goes to connected **peers**.

## The model (grounded in ZERO-MUTEX.md's mini-TCP)

Normal comms are **mini-TCP**: reliable, acked, bounded(1) backpressure. Two teardown shapes:

| shape | what it is | peer sees | reason? |
|---|---|---|---|
| **FIN** (clean close / graceful `/stop`) | the loop exits cleanly; senders dropped | `RecvError::Disconnected` (clean EOF — done by the transport twin) | n/a |
| **RST** (crash) | the loop is dying (handler panic → structured-exit) | **NEW: a distinct RESET signal** | NO — reason is the owner's (admin channel) |

The RST is legitimately **fire-and-forget by nature** — NOT the lazy fire-and-forget arc-119 retired: a dying
process *cannot* wait for acks. Like a TCP RST, it is a best-effort last word; if a peer misses it, that is
acceptable (best-effort, no retransmit, no ack).

## The contract (pinned)

- A crashing service **best-effort broadcasts an RST to every connected peer** (the serve loop's `clients`
  Vector — `service.wat`) **before it dies**, then lets-it-crash (the existing `eprintln`-terminal →
  `emit_structured_exit` path, `process/verbs.rs`). **No ack; no wait; no hang** — a send that fails (peer
  already gone) is skipped.
- **Peer side:** the RST surfaces through the peer's `recv'` as a distinct, catchable signal — proposed
  `RecvError::Reset` — DISTINCT from `Disconnected` (clean FIN), `Failed(reason)` (raw wire error, the twin),
  `Shutdown`, `FrameTooLarge`. It carries **no reason** (a peer learns "the server reset/crashed," not why).
- **Owner side:** unchanged — the `/start` handle still gets the crash reason via the crash channel
  (`PeerRecvError::Crashed`). The RST does not touch that.
- **Graceful `/stop`** stays the FIN (clean-EOF) — honest as-is; a *graceful* peer-notification is a separate
  small add if wanted, NOT this stone (the builder's focus is the crash/RST).

## The disconfirming probe (FIRST — the load-bearing unknown)

Can the serve loop's crash arm reach `clients` and best-effort-send an RST control frame **before** the
`panic_any`/`emit_structured_exit` path exits? Two sub-questions:
1. Does a handler `panic_any` unwind to a point where the serve loop still holds `clients` and can broadcast,
   or does it go straight to structured-exit past the loop? (If past — the RST must hook where `clients` is
   reachable: thread the peer set to `emit_structured_exit`, or broadcast in the serve loop's catch before
   re-raising.)
2. Can a distinct control frame be sent on a peer tx and recognized by the receiver's frame path as an RST
   (→ `RecvError::Reset`), separably from a data frame and from EOF?

Write a 10–15 line probe that reproduces exactly this (a process service, a handler that genuinely panics, a
connected peer reading) and prove the current behavior (peer sees clean-EOF `Disconnected` — no RST) before
building. **STOP if the panic boundary structurally forecloses reaching `clients`** — surface the exact
mechanism (that reshapes where the RST hooks); do not guess.

## RED gate (acceptance)

A process service whose handler **genuinely panics** → every connected peer's `recv'` surfaces
`RecvError::Reset` (the RST), NOT a bare clean-EOF `Disconnected`; the service still dies; the **owner still
gets the crash reason** on the admin channel; **best-effort** — no ack, no hang (a peer that already left is
skipped). No-regression: a genuine clean close still surfaces `Disconnected` (FIN); `dead_child_speaks`,
`probe_arc272_rs2_*`, `probe_arc259_thread_crash_reason`, the transport-twin probes all stay green; full suite
green by the orchestrator's own re-run.

## Blast radius

`src/comms/mod.rs` (`RecvError::Reset` variant + Display), the serve loop's crash arm in `wat/service.wat`
(best-effort RST broadcast to `clients`), the frame/transport path (send + recognize the RST control frame),
`runtime.rs` recv' (surface `Reset`), possibly `emit_structured_exit` (`process/verbs.rs`) if the peer set
must thread there. + the new probe. The compiler cascade drives the `RecvError` exhaustive-match sweep.
