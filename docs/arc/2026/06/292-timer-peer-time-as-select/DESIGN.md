# Arc 292 — the timer-Peer: time-as-select (`mora`'s keystone)

**Status:** SCOPED (2026-06-22). Surfaced while designing arc 291/292 reporting:
a service that "clocks its own perf" needs a periodic trigger — and the only
honest way to obtain one is `mora`'s law: *time is I/O; it arrives via the wire,
or it doesn't arrive honestly.* This arc builds the one primitive that makes that
law buildable, and in doing so **eliminates `sleep` as a concept**.

## The doctrine

> **The only way to obtain a time delay is to `select` on a timer.** There is no
> `sleep` verb — not as a primitive, not as a fallback, not once. A delay is a
> `select'` over a timer-Peer; nothing else is honest.

`sleep` is not a separate problem — it is the timer-Peer wearing a disguise.
`sleep(d)` ≡ select on a **one-shot** timer and discard the tick. `timeout`,
`cron`, `heartbeat`, `retry-backoff` are all usage patterns of the same one
primitive. We add ONE thing; the whole time-family falls out.

## The primitive: a select-able timer-Peer

A timer is an ordinary select-able `Peer'`:
- `(:wat::time::after d)` — one-shot: fires once after `d`.
- `(:wat::time::tick  d)` — periodic: fires every `d`.

You consume it exactly like any peer — `(:wat::kernel::select' timer)` /
multiplexed in `poll'` alongside real peers. A tick *arrives* as a
`ServiceEvent::Message`; you never ask "what time is it and is it past X" — the
wire tells you.

## Who blocks for N — the kernel, always (grounded, both tiers)

`mora`'s whole point: **nothing in wat ever waits N units.** The timer is the
*timeout arm of the one blocking call the reactor already makes*, and the kernel
is the waiter. Two tiers, symmetric, different syscall:

| tier | the one blocking call | timer is… | the waiter |
|---|---|---|---|
| **process** | `io_uring_enter` (CQE wait, `src/comms/process.rs`) | an `IORING_OP_TIMEOUT` SQE + a new `TIMER_TOKEN` beside `DATA`/`BROADCAST`/`LISTENER` | kernel hrtimer → CQE |
| **thread** | `crossbeam Select::select()` → `park_timeout` (`src/comms/thread.rs`) | a `crossbeam_channel::{after,tick}` `Receiver` registered as a Select arm | kernel futex (timeout) |

No background timer thread (crossbeam `after`/`tick` are helper-thread-free; io_uring
timeout is a kernel hrtimer). No userspace sleep. No busy-spin. The thread parks in
the *same* call that waits on sockets/channels; the kernel wakes it on whichever
fires first.

## Why this is RIGHT, not merely equivalent (the anti-hang property)

Because a delay is a `select'`, the timer shares its set with **`SHUTDOWN_RX` / the
broadcast cascade**. So a delay wakes on **whichever fires first — the deadline OR
shutdown**. A bare `thread::sleep(d)` is **uninterruptible** — it holds a thread
past kill for the full `d`, blocking teardown. That uninterruptible wait *is* the
arc-170 "leaks/hangs" class (the branch this work sits on). `mora` didn't forbid
`sleep` for purity — it forbade it because the naive sleep **is** the hang.
Expressing every delay as a select on a timer makes "wait for time"
**cascade-interruptible by construction**, which structurally kills the class.

```clojure
;; sleep, done right — a one-shot timer you select on; wakes at the deadline OR on shutdown
(:wat::core::defn :app::nap [d <- :wat::time::Duration] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::select' (:wat::time::after d)) -> :wat::core::nil
    ((:wat::service::ServiceEvent::Message _) nil)))

;; a heartbeat — a tail-recursive serve fn (NO loop; TCO), selecting on a periodic tick
(:wat::core::defn :app::heartbeat [t <- :Peer'  conn <- :…  sink <- :…] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::select' t) -> :wat::core::nil
    ((:wat::service::ServiceEvent::Message _)
       (:wat::core::let [_ (:telemetry/log sink (:app/stats conn (:app/stats-request)))]
         (:app::heartbeat t conn sink)))))         ;; tail call, TCO
```

## What exists / what's missing (grounded)

- ✅ `:wat::time::*` — the **clock**: `now`, `epoch-nanos`, `Duration` units
  (`Nanosecond`…`Day`), iso8601 (`src/time.rs`). All *pure readouts* — values.
- ✅ the reactors — process `io_uring` (token-multiplexed), thread `crossbeam
  Select` (arm-multiplexed). Both already block on a multiplexed set.
- ❌ **the timer** — nothing makes time *arrive*. No `timerfd`, no
  `IORING_OP_TIMEOUT` submission, no `crossbeam::tick` arm, no `sleep` (and we
  keep it that way). `mora` has been *held* only because wat could never wait on
  time at all. This arc builds the missing arrival.

## Sub-strikes

0. **DESIGN** (this doc) — the doctrine + the primitive.
1. **RED probe** — `(select' (after (millis 50)))` returns a tick ~50ms later;
   and a cascade probe: a nap interrupted early by shutdown wakes on the cascade,
   not the deadline. RED at HEAD (no `after`/`tick`).
2. **thread tier** — a timer `Receiver` registerable as a `thread::Select` arm
   (crossbeam `after`/`tick`); `select()` becomes deadline-aware automatically.
3. **process tier** — `IORING_OP_TIMEOUT` SQE + `TIMER_TOKEN` in the io_uring
   reactor; the CQE surfaces as a timer `ServiceEvent`.
4. **wat surface** — `:wat::time::after` / `:wat::time::tick` returning a
   select-able `Peer'`; unify both tiers behind it.
5. **derived usages (as wat, no new Rust)** — `nap`/`sleep`-shaped helper,
   `timeout` (select over [work-peer, after]), `cron`/`heartbeat` (tick), and
   `retry-backoff` (after with growing `d`) — all tail-recursive, all select-based.

## Out of scope (affirmative cuts)
- **A bare `sleep` verb** — never. The cut is the doctrine. A delay is a select.
- Hierarchical timing wheels / high-resolution scheduling — the kernel's hrtimer
  and futex are the scheduler; we do not reimplement them.

## Done = the gate
The RED probe goes GREEN on BOTH tiers: a one-shot `after` fires at its deadline;
a periodic `tick` fires repeatedly; and a nap in-flight is woken EARLY by the
shutdown cascade (proving the anti-hang property). `mora` is then enforceable:
every delay in the corpus is a `select'`, and a grep for `sleep` finds nothing.

## Sequencing
Foundational + independent of arc 291/290 (rides existing reactors; needs no
defservice change). It is the **prerequisite for the observability arc** (the
metrics heartbeat is a timer-widget). Build order remains 291 → 290; arc 292 can
land alongside or just after, and the observability arc follows it.
