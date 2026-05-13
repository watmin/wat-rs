# Arc 197 — Signal-events bus

**Status:** STUB. Captured 2026-05-13 from user direction. Not yet designed.

## Origin

Mid-arc-170 Slice C spawn, after empirically demonstrating the shutdown-aware channels cascade (Slice B). User asked:

> *"we could totally shim in other signals and the shutdown pipe could grow to deliver more signals?... any signal can issue a select wake up with a form bearing unit of description 'SIGUSR1 delivered' or whatever?"*

The recognition: arc 170's wake-pipe (one async-signal-safe byte channel between the signal handler and a normal-context worker thread) is THE substrate primitive for kernel→userland event delivery. Currently it carries one signal class (terminal: TERM/INT → drop SHUTDOWN_TX). It generalizes naturally to carry ALL observable signal events.

## Goal

Replace the existing atomic-polling layer for non-terminal signals (SIGUSR1/SIGUSR2/SIGHUP — see `src/runtime.rs:KERNEL_SIGUSR1/2/HUP`) with the substrate's lock-step recv discipline. Any wat thread can observe ANY signal by calling recv on the corresponding signal-event channel, multiplexed in the same `select!` as data channels.

## Architecture sketch

The wake-pipe becomes a tagged event bus:

```
Signal handler (async-signal-safe):
  writes ONE byte to wake-pipe; byte encodes which signal fired

Shutdown-worker thread (normal context):
  reads bytes from wake-pipe, dispatches by tag:
    TERM/INT  → drop SHUTDOWN_TX        (terminal, arc 170 cascade)
    USR1      → send () on SIGUSR1_TX    (observation, fire-once-per-signal)
    USR2      → send () on SIGUSR2_TX    (observation)
    HUP       → send () on SIGHUP_TX     (observation)
    (future)  → ...

Wat code sees per-signal Receivers exposed via substrate:
    :wat::runtime::sigusr1-rx
    :wat::runtime::sigusr2-rx
    :wat::runtime::sighup-rx
```

Users `select` on data + signal channels uniformly — no separate "polling" surface.

## The class distinction crystallized

| Signal class | Mechanism | Recv outcome | User code |
|---|---|---|---|
| **Terminal** (TERM/INT) | Drop SHUTDOWN_TX | `Err(ThreadDiedError::Shutdown)` | Result/expect panics |
| **Non-terminal** (USR1/USR2/HUP) | Send `()` on per-signal channel | `Ok(Some(()))` | Match on event, react |

Terminal = irreversible, cascade-broadcast, panic-class.  
Non-terminal = observable event, fire-once per signal, data-class.

Both share the same wake-pipe + worker. Both ride the substrate's `recv` discipline (per arc 170's shadow-channel architecture). Same `select!` machinery.

## Proposed API surface (sketch)

```scheme
;; Get the per-signal observation receiver
(:wat::runtime::sigusr1-rx)  -> :wat::kernel::Receiver<wat::core::nil>
(:wat::runtime::sigusr2-rx)  -> :wat::kernel::Receiver<wat::core::nil>
(:wat::runtime::sighup-rx)   -> :wat::kernel::Receiver<wat::core::nil>

;; User code reacts to SIGUSR1 alongside data:
(:wat::core::let
  [data-rx (... open some data channel ...)
   sigusr1-rx (:wat::runtime::sigusr1-rx)
   result (:wat::kernel::select
            data-rx     :on-data
            sigusr1-rx  :on-reload)]
  (:wat::core::match result -> :wat::core::nil
    ((:wat::core::Ok :on-data)    (handle-data ...))
    ((:wat::core::Ok :on-reload)  (handle-reload-config))
    ((:wat::core::Err _)          (panic-shutdown))))
```

## Open questions (for DESIGN phase)

1. **Coalescing semantics.** Five SIGHUPs in a burst — does the user see 5 events or 1? The existing atomic-polling layer coalesces ("five SIGHUPs read as one yes" per arc 060+). Bounded channel of size 1 coalesces; unbounded channel preserves count. Which is honest? Most signal-driven workflows want coalesced (one HUP = reload config; multiple HUPs in rapid succession = still one reload). Default: bounded(1) with overflow-drop.

2. **Retire existing atomic-polling API?** The current `(:wat::kernel::stopped?)` / `(sigusr1?)` / `(sigusr2?)` / `(sighup?)` polling primitives are arc 060+. After arc 197 ships, do they retire (per arc 109 § I rename queue) or coexist? Coexistence preserves backward compat; retirement collapses to one canonical path (per `feedback_substrate_owns_not_callers_match` discipline). Verdict probably retire-with-grace-period.

3. **Per-thread vs process-wide subscription.** Signal handlers run in arbitrary thread context. The worker dispatches to per-signal channels — but who CLONES the receiver to observe? Per-thread clones, or process-wide receivers each thread shares? Per-thread is more flexible; process-wide is simpler. Open.

4. **Signal-event payload.** Currently sketched as `Receiver<wat::core::nil>` (each event = one unit). Could carry timestamp, signal number, etc. — but those are pure data the kernel doesn't give us cheaply. Keep at `nil` unless a use case demands richer payload.

5. **Async-signal-safety bookkeeping.** The signal handler must encode WHICH signal fired into the wake-pipe byte. Use the signal number itself (1-31 fit in one byte). Worker reads byte → dispatches by signal-number. Simple. Verify no fancy operations needed in handler.

6. **Cross-process semantics.** A forked child inherits the wake-pipe... no, actually each child sets up its OWN init_shutdown_signal() at bootstrap (per Slice A discipline). Per-process wake-pipes are correct — signals are per-process anyway. Confirm via test.

7. **Interaction with PR_SET_PDEATHSIG (arc 170 Slice C).** When parent dies, kernel sends SIGTERM to child. Child's signal handler writes "T" (terminal) byte to its own wake-pipe. Cascade fires. This is consistent — no change needed to arc 197 design for this scenario.

## Why this matters

Per arc 170 INTERSTITIAL "Wat disciplines its own designers": the wake-pipe is a primitive the substrate INVENTED for the shutdown cascade. The substrate now has a kernel-events bus. Extending it to all observable signals is exactly the kind of move where the substrate's existing patterns make the right shape obvious.

Per `feedback_deferral_bias_is_signal`: if a user articulates an extension AND the bias is to defer it, the bias is the signal it's needed. The user explicitly said *"we could totally shim in other signals"* — that's a recognition the substrate ALREADY has the mechanism, just needs the wiring.

Per arc 110 + the shadow-channel architecture: signal events delivered via recv are *uniform with data*. Users react to SIGHUP the same way they react to a config-channel message — same `select`, same `match`, same `Result/expect` discipline. The substrate-imposed shadow-channel pattern extends naturally to signal-event channels.

## Out of scope (until DESIGN)

- Real-time signals (SIGRTMIN..SIGRTMAX). Useful but separate concern; would need queuing semantics the standard signals don't.
- SIGCHLD handling for spawn-process supervision. Arc 170's PR_SET_PDEATHSIG + structured-exit covers this differently.
- Cross-process signal observation (one process observes signals to another). The wake-pipe is per-process; cross-process would need different mechanism.

## Cross-references

- `wat-rs/docs/arc/2026/05/170-program-entry-points/DESIGN-SHUTDOWN-AWARE-CHANNELS.md` — the wake-pipe primitive's origin design
- `wat-rs/docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` §"How the shadow channel fans out across threads" — the recv/select mechanism arc 197 extends
- `wat-rs/src/runtime.rs:KERNEL_SIGUSR1/2/HUP` — existing atomic-polling layer arc 197 supersedes
- `wat-rs/src/fork.rs:substrate_on_stop_signal` — the signal handler shape arc 197 generalizes
- `man 7 signal-safety` — POSIX async-signal-safety constraints (libc::write is on the list)
- arc 060 — original signal-observation primitives (atomic-polling); supersedes target
- arc 170 SHUTDOWN-AWARE-CHANNELS-BACKLOG — Slice A-E ship the shadow-channel; arc 197 builds on top

## Status

STUB. Awaiting arc 170 closure before DESIGN phase begins. Future arc-197 work will ride on arc-170-shipped substrate (wake-pipe + worker + per-signal handler dispatch).
