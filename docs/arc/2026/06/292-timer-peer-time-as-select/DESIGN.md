# Arc 292 — the timer-Peer: `send_after`, time-as-select (`mora`'s keystone)

**Status:** SCOPED (2026-06-22; rev 2 — converged on the `send_after` shape).
Surfaced while designing arc 291/observability: a service that "clocks its own
perf" needs a periodic trigger, and `mora`'s law says the only honest one is *time
arriving via the wire*. Pinning that primitive eliminates `sleep` as a concept and
— it turns out — re-derives Erlang's `send_after`.

## DECISIONS LOCKED (2026-06-22, RED-probe-grounded — supersede the illustrative forms below)

The arc-292 RED probe (`wat-tests/timer-after.wat`) grounded two surface decisions
that correct rev 2's sketch. Where the forms below say `:wat::time::after` or
`Peer'<nil,O>`, read these:

- **D1 — namespace: `:wat::kernel::after` / `:wat::kernel::tick`** (NOT `:wat::time::`).
  They are effectful peer-constructors, so they live beside the other comms verbs
  (`select'`/`recv'`/`connect'`); they *consume* a `:wat::time::` `Duration`. (Honest
  axis: the pure-readout time module must not host an effectful reactor primitive.)
- **D2 (= B1) — the timer is a TIER peer; a LOCUS picks the tier.** The RED probe
  proved `select'` demands `Thread'<I,O> | Process'<I,O>`, not a generic `Peer'`. So:
  **`(:wat::kernel::after <locus> <duration> <msg>) → <Tier>'<nil,O>`** —
  `(after (:wat::spawn::thread) d msg) → Thread'<nil,O>` (crossbeam reactor),
  `(after (:wat::spawn::process) d msg) → Process'<nil,O>` (io_uring reactor).
  Mirrors `(start (thread) state0)`; **`select'` is unchanged** (the probe confirmed a
  `Thread'<nil,keyword>` timer satisfies the set). `after` is 3-arg.
- **D3 — `tick` is ANNIHILATED. There is exactly ONE timer primitive: `after`.**
  Periodic is not a primitive — it is a TCO re-arm of `after`, and **the loop's
  recursion IS the timer's lifecycle** (it stops by not recursing / a shutdown arm;
  no standing timer to cancel, no leak). The two periodicity modes are just *which
  delay you re-arm with*:
  - **fixed-delay** (period = `d` + work-time; drifts) — `after(d)`. For backoff,
    retry, debounce, "wait `d` between attempts."
  - **fixed-rate** (no drift, anchored to absolute deadlines) — re-arm
    `after(next_deadline − now)` where `next_deadline += d`. For cron, steady
    heartbeat, metrics-cadence, rate-limit refill.
  A standing `tick` primitive was rejected: it imposes a stop/cancel surface and a
  leak risk the TCO loop doesn't have, hides the cadence in Rust, and saves nothing
  (re-creating `after` per fire is nanoseconds). The whole `tick` plan was a feature
  whose existence was the defect — killed before it shipped. (`mora`/examinare: one
  primitive, the boss beaten for all time.)

- **D4 — process-tier mechanism: `timerfd`, NOT `IORING_OP_TIMEOUT`** (2026-06-22,
  builder-locked; four-questions-grounded). The "Who blocks for N" table below and
  sub-strike 3 say `IORING_OP_TIMEOUT SQE + TIMER_TOKEN` — **prior state, preserved.**
  Superseded because, grounded against the disk: `comms::process::Receiver<T>` is
  **rigidly fd-backed** (`read_fd: OwnedFd` mandatory; every method assumes it), and
  `select'` consumes a timer as a `Process'<nil,O>` peer (D2). A `timerfd_create(2)`
  receiver drops straight into the existing `read_fd` slot and the existing
  `PollAdd POLLIN` data-arm path — **`Select` is UNCHANGED.** An `IORING_OP_TIMEOUT`
  arm (no fd) would force enum-ifying the warded `Receiver` + reactor surgery
  (`-ETIME`-is-success, `Timespec` lifetime). **io_uring stays the sole waiter either
  way** (io_uring *polls the timerfd*); the doctrine "best-of-breed Linux, io_uring
  reactor" is satisfied — `timerfd` IS best-of-breed Linux, and it is the arc's
  ORIGINAL conception (arc-214 DESIGN: *"timeouts = a timerfd arm"*; R1 path-of-voices
  lists "timerfd-vs-IORING_OP_TIMEOUT" as live grounding). The `IORING_OP_TIMEOUT`
  wording was the drift; this restores timerfd. Cost of timerfd quantified: 1 fd (of
  1,048,576), a few hundred bytes kernel mem, ~3 syscalls/arm — negligible at the
  time-family's ms-to-second cadence; the hrtimer is identical to io_uring's.

- **D5 — the timer peer is a `Process'<nil,O>` with `pidfd: None`** (mirrors the
  SHIPPED thread tier's `Thread.join: Option = None` for its timer). Per D2 the timer
  IS a peer, so it presents as a `ProcessPeerBundle{ peer: Process<String,String>,
  err, _lifeline_w }`; `Process` mandates `pidfd: Pidfd`, but a timer has no child.
  Faithful fix = decomplect `Process.pidfd` to `Option<Pidfd>` (honest absence, not a
  sentinel-pidfd lie) — the same shape the thread tier already uses for `join`.

- **STATUS (2026-06-22):** thread tier `after` is **SHIPPED + GREEN** (crossbeam
  `after`, zero-mutex `OwnedMoveCell`); the family rides it (nap/backoff/retry/
  first-deadline). So the "What exists / what's missing" section below ("No `timerfd`,
  no crossbeam after arm, no `sleep`") is **prior state, preserved** — true at scoping,
  now true only of the process tier. The **process tier is the sole remainder.**

> **Prior-state preservation note (per the amend-don't-delete discipline):** every
> `tick` form below (annihilated by D3), every `:wat::time::after`/`tick` spelling
> (renamed to `:wat::kernel::` by D1), and every `IORING_OP_TIMEOUT` reference
> (superseded by D4) is left **in place, marked, recognized as prior state** — not
> deleted. The body records the arc's reasoning journey; the decisions above are the
> current truth. Inline `SUPERSEDED →` pointers mark the worst spots.

## DECISION REVISION — 2026-06-23 (supersedes D2 + D5; D4 holds)

The arg0 surface was re-examined (intueri + solvere cast on it) and reshaped with the
builder. **Prior text for D2 and D5 is left in place above, recognized as superseded.**

- **REV-1 (supersedes D2's "locus picks tier") — arg0 is a declared `PeerKind`, the
  timer is a selectable IN the `select'` vector.** intueri found `(:wat::spawn::thread)`
  as arg0 is a **Level-1 lie** (it promises spawn-config, delivers tier-pick; the whole
  `ThreadOpts`/`ProcessOpts` payload is dead weight — only `class_fqdn` is read); solvere
  found the **same braid** at `eval_listener_prime`. So arg0 is NOT the spawn-locus and NOT
  ambient-reach (builder: *"taking a program config isn't a good solution"*). It is the
  **existing `:wat::program::PeerKind` enum** (`:thread` | `:process`), declared explicitly.
  - **The tier-agnostic IDIOM:** a program that "doesn't care" grabs its own
    `wat.peer-kind` off `(:wat::program::env)` and passes it — *explicit but constant in
    shape*; programs never hardcode their tier (builder: *"i want programs to not care"*).
    Explicit literal `(after :wat::program::PeerKind::process …)` for "I want a specific kind."
  - **`select'` is a vector of selectables; the timer is a member of it** — `(select'
    (Vector client (after kind d msg)))` — Go's `select { case <-After }` / Clojure's
    `(alts! [client (timeout 100)])`. The rejected alternative A′ (timer as a *distinguished
    arg* `(select' (Vector client) timer)`) contradicts what `select'` IS (a fan-in over a
    list) — builder: *"A' doesn't have a vector of selectables.... goofy as shit."*

- **REV-2 (the type mechanism) — `after -> ⟨tier-open⟩'<nil, O>`.** Because a `PeerKind`
  VALUE (literal or env-grabbed) is tier-OPAQUE at check time (enum variants are not
  distinct types — `types.rs:147`; narrowed only inside `match`), `after`'s result is a
  **tier-open peer type that fuses to the concrete tier of the homogeneous `select'` set it
  joins** (head-polymorphism / a tier-open timer type). This PRESERVES static
  tier-homogeneity (`Thread'`≠`Process'` heads don't unify → a mixed real-peer set is still
  a compile error) and carries `O` (msg type-checked). Rejected alternative **B**
  (a `Selectable<I,O>` *supertype* + runtime tier-resolution) FAILED the four-questions:
  it **downgrades a compile-time homogeneity guarantee to runtime** — the type would lie
  about what it enforces. Rejected **D** (bare fresh-var return): an `msg ↔ O` soundness
  hole. The ruthless-correctness pass chose the tier-open type (sound) over the easier-but-
  weaker B.
  - **`select'` relaxes to constrain only `O` (the receive side) across elements, not `I`**
    — a timer has `I = nil` (you never send *to* it) while work-peers have a real `I`; the
    homogeneous set must agree on what you RECEIVE (`O`), not on `I`.

- **REV-3 (supersedes D5's "pidfd: Option") — the timer peer is the `ProcessSelectable`
  enum, not an Optional field.** Per the builder's doctrine *"Option communicating a
  semantic statement rather than presence is screaming for an enum,"* the process-tier peer
  cell is `enum ProcessSelectable { Spawned(ProcessPeerBundle), Timer(TimerPeer) }` (L1,
  SHIPPED `19e78f94`). `pidfd` stays a mandatory field of `Spawned` (a child always has
  one); `Timer` has no pidfd — illegal states unrepresentable. The D5 "pidfd: Option"
  framing above is superseded.

- **D4 (process-tier mechanism = `timerfd`) HOLDS** — L2 SHIPPED (`1e8eefc1`):
  `comms::process::Source { Pipe, Timer }` + `timer()`, io_uring polls the timerfd, sole
  waiter, zero-mutex.

- **REV-4 (three loci, one interface — design tier-spanning things general over THREE).**
  thread, process, and the **deferred remote** share ONE peer interface (a peer = optional
  tx + N rx; `select'` multiplexes all rx). **process ≈ remote**, nearly identical: process
  = 1 tx (stdin) + **2 rx** (stdout=`output`, stderr=`err`); remote = 1 tx + **1 rx**
  (stdout+stderr **multiplexed**); thread = crossbeam; a timer = **0 tx + 1 rx** (the
  simplest peer). LAW: the tier-open `Timer'<O>` unify rule (REV-2) is **"a timer fuses
  into a peer of ANY tier"** — keyed on "the other side is a known peer-tier head," NOT a
  hardcoded thread/process pair → `remote` slots in for free. `:wat::program::PeerKind`
  will grow `:remote` (deferred). `select'` stays N-rx-per-peer general. No silent 2-only
  assumption anywhere.

## The doctrine

> **Every temporal behaviour is a timer that delivers a typed message into a
> `select`.** There is no `sleep` verb — not as a primitive, not as a fallback,
> not once. A delay is a `select'` over a timer that delivers a message; nothing
> else is honest.

`sleep`, `timeout`, `cron`, `heartbeat`, `backoff`, `debounce`, `rate-limit`,
`watchdog`, `deadline` are **not** distinct mechanisms — they are all *usages* of
one primitive: arm a timer to deliver message `M` after/every `d`, handle `M` in a
`select'` arm. We add ONE thing; the whole time-family falls out, and `select'`
does not change.

## The primitive — `send_after`, re-derived

A timer **delivers a caller-chosen, typed message** after a delay. Its output type
**is** the `select'` set's type `O`, because *you hand it the `O` to emit*:

> **SUPERSEDED → D1 (namespace `:wat::kernel::`) + D3 (`tick` annihilated).** The two
> illustrative forms below are PRIOR STATE, preserved: the real verb is
> `(:wat::kernel::after d msg)`, and there is no `tick` — periodic is a TCO re-arm of
> `after`. Kept here to show the `send_after` shape as first reached.

```clojure
(:wat::time::after d msg)    ;; → Peer'<nil, O>, delivers `msg` (an O) ONCE after d
(:wat::time::tick  d msg)    ;; → Peer'<nil, O>, delivers `msg` every d (periodic)
```

This is Erlang's `erlang:send_after(Time, Dest, Msg)` — "deliver `Msg` after
`Time`." Because the timer produces an `O`, it drops into the **homogeneous**
`select'` next to real peers with **zero `select'` change**:

```clojure
;; timeout = a homogeneous select' — work-peer AND timer BOTH yield O.
(:wat::core::match (:wat::kernel::select'
                     (:wat::core::Vector :Peer' work-peer (:wat::time::after d :Op::Timeout)))
    -> :R
  ((:wat::service::ServiceEvent::Message _ (:Op::Work v)) (handle v))
  ((:wat::service::ServiceEvent::Message _ :Op::Timeout)  (on-timeout)))
```

### Why this shape (the decisions, locked)

- **The timer delivers `O`, not a fixed `Instant`/`nil`, and there is no distinct
  `Timer` type.** It is a `Peer'<nil, O>`. The caller chooses the message *and* its
  type. This is what keeps `select'` homogeneous and unchanged — the timer is just
  another source yielding the set's `O`.
- **It is already the wat idiom.** defservice protocols are *already enums*
  (`Op = Get | Put`). Add `Op::Timeout` / `Op::Tick` and the serve loop's *existing*
  match handles it. **A timed service is a service with a timer arming `Op::Tick`
  into its own `select'` set — zero serve-loop change.** The `init-fn` arms it; the
  pure handler reacts to `Op::Tick` like any op.
- **Go's heterogeneous `select` stays a deferred door.** Go avoids unifying sources
  into one enum (each `case` independently typed); the `send_after` way requires the
  sources to share `O` (wrap a divergent source in a variant if ever needed). The
  enum way *always* works and needs no new machinery, so heterogeneous-`select` is
  perpetually-awaiting-definition — *don't build the forcing function* until a real
  need for genuinely-different-typed multi-source waiting appears.

## The whole time-family (usages, no new mechanism)

> **SUPERSEDED → D3 (`tick` annihilated).** The `(tick …)` rows below are PRIOR STATE,
> preserved. Current truth: `heartbeat/cron` and `rate-limit` are a TCO re-arm of
> `(after (deadline−now) msg)` (fixed-rate, absolute-anchored); `retry-backoff` is a
> re-arm of `(after d msg)` (fixed-delay). One primitive, `after`; the loop is the
> lifecycle.

| usage | how |
|---|---|
| **sleep / wait** | `(select' [(after d nil)])`, ignore the message |
| **timeout** | `(after d :Op::Timeout)` as an arm beside the work peer |
| **heartbeat / cron** | `(tick d :Op::Tick)` |
| **retry-backoff** | nap with a growing `d` (tail-recursive) |
| **debounce** | re-arm `(after d :Flush)` on each event; only the last fires |
| **rate-limit** | `(tick interval :Token)` refills; drop work when no token |
| **watchdog** | `(after d :Deadline)`; each heartbeat re-arms it; firing = peer went dark |
| **deadline propagation** | pass the `(after …)` peer down; it fires for the whole subtree at once |

All of them: arm a timer to deliver `M`, match `M` in `select'`. TCO, never `loop`.

## Who blocks for N — the kernel, always (grounded, both tiers)

`mora`'s point: **nothing in wat ever waits N units.** The timer is the *timeout arm
of the one blocking call the reactor already makes*; the kernel is the waiter.

> **SUPERSEDED → D4 (process = `timerfd`) + D3 (`tick` annihilated).** The process row's
> `IORING_OP_TIMEOUT SQE + TIMER_TOKEN` and the thread row's `crossbeam::tick` are PRIOR
> STATE, preserved. Current truth: process tier = a `timerfd_create(2)` receiver polled
> by the existing io_uring `PollAdd` data-arm (no `TIMER_TOKEN`, `Select` unchanged —
> io_uring still the sole waiter); thread tier = `crossbeam::after` only (no `tick`).
> The "kernel is the only waiter, both tiers" claim holds unchanged.

| tier | the one blocking call | timer is… | the waiter |
|---|---|---|---|
| **process** | `io_uring_enter` (CQE wait, `src/comms/process.rs`) | an `IORING_OP_TIMEOUT` SQE + a new `TIMER_TOKEN` beside `DATA`/`BROADCAST`/`LISTENER`; on CQE, deliver `msg` | kernel hrtimer → CQE |
| **thread** | `crossbeam Select::select()` → `park_timeout` (`src/comms/thread.rs`) | a `crossbeam_channel::{after,tick}` `Receiver` registered as a Select arm; map its fire → `msg` | kernel futex (timeout) |

No background timer thread (crossbeam `after`/`tick` are helper-thread-free; io_uring
timeout is a kernel hrtimer). No userspace sleep. No busy-spin.

## Why this is RIGHT, not merely equivalent (the anti-hang property)

Because a delay is a `select'`, the timer shares its set with `SHUTDOWN_RX` / the
broadcast cascade — so a delay wakes on **whichever fires first, the deadline OR
shutdown**. A bare `thread::sleep(d)` is **uninterruptible**: it holds a thread past
kill for the full `d`, blocking teardown — *exactly* the arc-170 "leaks/hangs" class
(the branch this work sits on). `mora` didn't forbid `sleep` for purity; it forbade
it because the naive sleep **is** the hang. `send_after`-into-`select'` makes every
wait cascade-interruptible by construction, killing the class.

## What exists / what's missing (grounded)

> **SUPERSEDED → STATUS (thread tier SHIPPED).** This section is PRIOR STATE as of
> scoping, preserved. The thread-tier `after` is now BUILT + GREEN (crossbeam `after`,
> zero-mutex), so "❌ the timer — nothing makes time arrive" is true only of the
> **process tier** now. The grounding (`:wat::time::*` = pure clock; `mora` held for
> lack of arrival) remains accurate.

- ✅ `:wat::time::*` — the **clock**: `now`, `epoch-nanos`, `Duration` units,
  iso8601 (`src/time.rs`). All *pure readouts* — values, not arrivals.
- ✅ the reactors — process `io_uring` (token-multiplexed), thread `crossbeam
  Select` (arm-multiplexed). Both already block on a multiplexed set.
- ✅ `select'` (`runtime.rs:24503`) — homogeneous fan-in, returns `ServiceEvent`.
- ❌ **the timer** — nothing makes time *arrive*. No `timerfd`, no
  `IORING_OP_TIMEOUT`, no `crossbeam::{after,tick}` arm, no `sleep` (kept that way).
  `mora` was *held* only because wat could never wait on time at all. This arc
  builds the arrival, as `send_after`.

## The one check-detail (verify at strike)

`select'` currently requires all elements be "the same tier (first element's
type_path decides)" (`runtime.rs:24494`). For the timer (`Peer'<nil,O>`) to sit
beside work (`Peer'<I,O>`), `select'` must constrain **only the output `O` + the
tier — never the send-side `I`** (it never sends). If the checker over-constrains on
the full `Peer'<I,O>`, relaxing it to "same `O`, same tier" is the small enabling
change. Confirm exactly what it constrains before building.

## Sub-strikes

> **SUPERSEDED → STATUS + D3 + D4 (the list below is PRIOR STATE, preserved).**
> Current: (0) DESIGN ✓; (1) RED probe ✓ thread + ✓ process (`timer-after-process.wat`,
> ignore-marked, `90f29cd3`); (2) thread tier ✓ SHIPPED; (3) **process tier = the
> sole remainder — `timerfd` (D4), NOT `IORING_OP_TIMEOUT`; `pidfd: Option` (D5)**;
> (4) wat surface = `:wat::kernel::after` (D1), no `tick` (D3); (5) family = ✓ proven
> on `after`. The original sub-strike text (kept below) names `IORING_OP_TIMEOUT`,
> `:wat::time::`, and `tick` — read it as the journey, not the current plan.

0. **DESIGN** (this doc) — doctrine + `send_after` primitive + the family.
1. **RED probe** — `(select' [(after (millis 50) :tick)])` returns `:tick` ~50ms
   later; a `(select' [work (after d :timeout)])` returns `:timeout` when work is
   silent; and a cascade probe: a nap is woken EARLY by shutdown, not the deadline.
   RED at HEAD (no `after`/`tick`). Commit before build.
2. **thread tier** — a timer `Receiver` (crossbeam `after`/`tick`) registerable as a
   `thread::Select` arm, delivering the caller's `msg`; `select()` becomes
   deadline-aware automatically.
3. **process tier** — `IORING_OP_TIMEOUT` SQE + `TIMER_TOKEN` in the io_uring
   reactor; on CQE, surface the caller's `msg`.
4. **wat surface** — `:wat::time::after` / `:wat::time::tick` returning
   `Peer'<nil,O>`; the `select'` `I`-relaxation (per the check-detail above).
5. **the family, as wat (no new Rust)** — `sleep`/`nap`, `timeout`, `heartbeat`,
   `retry-backoff`, etc. as tail-recursive helpers over `after`/`tick` + `select'`.

## Out of scope (affirmative cuts)
- **A bare `sleep` verb** — never. The cut is the doctrine: a delay is a select.
- **Heterogeneous Go-style `select`** — deferred door; the `send_after`/enum way
  covers every need without it.
- Hierarchical timing wheels / hi-res scheduling — the kernel hrtimer + futex are
  the scheduler; we do not reimplement them.

## Done = the gate
The RED probe goes GREEN on BOTH tiers: a one-shot `after` delivers its message at
the deadline; a periodic `tick` delivers repeatedly; a `select'` over `[work,
after]` returns the timeout message when work is silent; and a nap in-flight is
woken EARLY by the shutdown cascade (the anti-hang property). `mora` is then
enforceable: every delay in the corpus is a `select'`, and a grep for `sleep` finds
nothing.

## Sequencing
Foundational + independent of arc 291/290 (rides existing reactors; no defservice
change). Prerequisite for the observability arc (the metrics heartbeat is a
`(tick d :Op::Tick)` arm). Build order stays 291 → 290; arc 292 can land alongside
or just after.

## Lineage
Not borrowed — re-derived. `send_after` is Erlang/OTP's timer; the convergence is
`mora` (time is I/O) + the homogeneous `select'` + protocol-enum messages landing on
the same answer Erlang reached for distributed, fault-tolerant, timed services.
Erlang's *semantics* via a Clojure/Haskell surface.
