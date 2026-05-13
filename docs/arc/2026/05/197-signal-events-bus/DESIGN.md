# Arc 197 — Signal-events bus

**Status:** STUB. Captured 2026-05-13 from user direction. Revised same day from per-signal-channel framing to value-bearing-enum framing.

## Correction (added 2026-05-13)

The initial sketch in this stub used per-signal Receivers (`:wat::runtime::sigusr1-rx`, `sigusr2-rx`, `sighup-rx` — one channel per signal kind). User correction:

> *"i don't think `sigusr1-rx) ;; observable signal events` this is a good idea... that's muddying the water - the select should be value bearing, some enum who communiates a signal - i think we should extend shutdown to 'signal' delivery and users write their own panic handlers or responders to HUP, USR1, USR2 and whatever else"*

**The corrected architecture: ONE channel, value-bearing `SignalEvent` enum.** Every signal arrives as a typed value on the same channel. User code matches on the enum variant and writes its own handler/responder. The substrate delivers; the user decides.

The existing arc 170 Slice B "shutdown" channel is the seed of this — it should generalize to deliver ALL signal events, not just terminal ones. Shutdown isn't a special architectural concept; it's a particular `SignalEvent::Sigterm` value that users typically panic on by default.

The rejected per-signal-receiver framing is preserved below per `feedback_inscription_immutable` as historical record. The value-bearing-enum framing is the design target.

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

## Proposed API surface (revised again 2026-05-13 — `Stopped` meta-variant)

User correction in same conversation:

> *"'shutdown' is a signal who wraps both sigint and sigterm - they are identical to me - it arrives as a meta (Stopped :SIGINT) or (Stopped :SIGTERM) users can grab the signal if they want, but they are a shutdown"*

The framing: SIGINT and SIGTERM are NOT distinct kinds at the wat-user level. They're both the same META event — *"stop the program"* — with the specific kind as sub-data the user can drill into if they care. Same model as `ThreadDiedError::Panicked(payload)` — what kind of panic is sub-data.

```scheme
;; ONE channel exposed by the substrate; delivers SignalEvent values
;; for any signal observed by the process.
(:wat::runtime::signal-events) -> :wat::kernel::Receiver<wat::runtime::SignalEvent>

;; SignalEvent enum (substrate-provided)
(:wat::core::enum :wat::runtime::SignalEvent
  ;; META: "the kernel told us to stop." Wraps both SIGINT and SIGTERM
  ;; because they are semantically identical at the user level. Users
  ;; can match on the inner kind if they care to distinguish.
  (Stopped :wat::runtime::StopKind)
  ;; Non-terminal signals — each its own variant because they have
  ;; behaviorally distinct conventional meanings.
  :Sigusr1    ;; user-defined; user decides
  :Sigusr2    ;; user-defined; user decides
  :Sighup     ;; conventionally "reload config"; user decides
  ;; future: SignalEvent grows with new variants as needed
)

;; The inner kind for Stopped — substrate-provided enum
(:wat::core::enum :wat::runtime::StopKind
  :SIGINT
  :SIGTERM)

;; User code multiplexes data + signals via standard recv discipline.
;; Common case: match (Stopped _) without drilling into SIGINT/SIGTERM
;; distinction — they both mean "stop."
(:wat::core::let
  [data-rx (... data channel ...)
   signal-rx (:wat::runtime::signal-events)
   result (:wat::kernel::select
            data-rx    :on-data
            signal-rx  :on-signal)]
  (:wat::core::match result -> :wat::core::nil
    ((:wat::core::Ok :on-data ((:wat::core::Some msg)))
      (handle-data msg))
    ((:wat::core::Ok :on-signal ((:wat::core::Some sig)))
      (:wat::core::match sig -> :wat::core::nil
        ;; Common case: don't care about INT vs TERM — both stop
        ((:wat::runtime::SignalEvent::Stopped _)  (panic-shutdown))
        ;; Behaviorally distinct variants
        (:wat::runtime::SignalEvent::Sighup   (reload-config))
        (:wat::runtime::SignalEvent::Sigusr1  (handle-usr1))
        (:wat::runtime::SignalEvent::Sigusr2  (handle-usr2))))
    ;; ... existing data/signal disconnect/None handling ...
    ))

;; Power-user case: drill into specific stop kind if needed
;; (e.g., logging "user pressed Ctrl-C" vs "supervisor sent SIGTERM")
((:wat::runtime::SignalEvent::Stopped :wat::runtime::StopKind::SIGINT)
  (log "user interrupted") (panic-shutdown))
((:wat::runtime::SignalEvent::Stopped :wat::runtime::StopKind::SIGTERM)
  (log "supervisor requested stop") (panic-shutdown))
```

**Substrate doctrine:** signals are delivered as data. Users write their own
handlers. The substrate does NOT impose "SIGTERM means panic" — it just
delivers SignalEvent::Sigterm. Convention: most user code matches SIGTERM/SIGINT
and panics via assertion-failed!; non-terminal signals dispatch to handlers.

**The shutdown cascade evolves:** arc 170 Slice B's substrate-imposed
shutdown-on-disconnect (Result/expect panics on Err(Shutdown)) becomes a
specific case of "user wrote a handler that panics on SignalEvent::Sigterm."
The substrate stays out of the policy decision; it delivers the event.

This is consistent with the broader doctrine: substrate provides primitives
(here: signal event delivery), users compose discipline (here: panic
handlers for terminal signals, responders for non-terminal). Per
`feedback_substrate_owns_not_callers_match`: the substrate owns DELIVERY;
users own POLICY.

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
