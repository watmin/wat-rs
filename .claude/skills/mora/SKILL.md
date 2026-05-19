---
name: mora
description: Hunt the pause. The datamancer suffers no mora — every wait must arrive via the wire, not via mechanism. Sleep is a guess; guesses race. Time is I/O; it arrives as an fd-event or it doesn't arrive honestly.
argument-hint: [file-path or directory]
---

# Mora

> *mora* — Latin: delay, pause, hindrance. The thing the datamancer cannot suffer. The thing that, when present in the code, proves the code is guessing about events it should be waiting on.

The fourth spell of the wat-rs grimoire alongside *complectēns* (test composition), *perspicere* (deep types), and *vocare* (caller-vantage). Where those check structure, mora checks **time discipline**: does this code WAIT via mechanism, or via the wire?

A `thread::sleep(20ms)` is a guess. It says: "I think the kernel needs about 20ms to make this observable, so I'll burn 20ms and hope." Sometimes the guess is too short and the test races. Sometimes it's too long and the test is slow. Always it is a lie about how the substrate's events arrive — kernel events arrive when they arrive, observable via poll/epoll/io_uring, NOT after a chosen number of milliseconds.

The datamancer waits with the wire. Time itself is I/O — `clock_gettime(2)`, `nanosleep(2)`, `timerfd_create(2)` are all syscalls. The honest shape is to register the timer as an fd-event and select over it alongside data and broadcast arms. Until that reactor exists, the substrate has no honest sleep. Tests that need to coordinate use the wire (channels, ack signals, EOF on close, POLLHUP on peer-close).

## The principle

Every `wait` in a program answers ONE question: what am I waiting FOR? If the answer is "an event in the world" (data arriving, a peer closing, a timer firing), the wait belongs in a select over the appropriate fds. If the answer is "some milliseconds" without a corresponding event, the wait is a guess — and guesses race.

Mora asks: **is this wait delivered by the wire, or by mechanism?**

The golang `select { case <-ch: ...; case <-time.After(d): ... }` is the reference shape. Even time arrives via channel. Until the wat reactor exposes this shape, the substrate has no sleep. Tests respect this until then.

## What mora flags

Any code that WAITS via a chosen-duration mechanism:

**Direct sleep calls (Level 1 — always flag):**
- `std::thread::sleep(Duration::*)` — the canonical violation
- `thread::sleep(...)` (with std::thread aliased)
- `tokio::time::sleep(...)` / `async_std::task::sleep(...)` — runtime-specific variants
- `std::process::Command::*` patterns that use sleeps to "wait for the process to come up" — STOP; use stdout markers or pidfd
- `loop { ...; thread::sleep(...) }` — busy-wait dressed as a loop

**Timeout-as-mechanism (Level 1 — always flag):**
- `recv_timeout(Duration::*)` — the duration is the mechanism; if you need a deadline, deliver it via a timer-arm in select
- `wait_timeout(Duration::*)` on Condvar (ZERO-MUTEX bans Condvar anyway; double violation)
- Any `*_timeout(Duration)` API where the duration determines when wait STOPS rather than what event delivers
- `Instant::now() + Duration::*` followed by `while Instant::now() < deadline { ... }` — busy-wait

**Test-time pseudo-syncs (Level 2 — flag with rationale check):**
- `thread::sleep` in test setup "to give the kernel a moment" — almost always a lie covering a real race or a missing lock-step
- `std::thread::yield_now()` followed by an assertion — yielding is hoping the scheduler does what you want; that's a guess
- Inline `Duration::from_millis(N)` magic numbers near assertions — even if not in a sleep call, the proximity is a smell

## What mora does NOT flag

**Time-as-data (NOT a wait):**
- `Duration::from_millis(N)` as a CONFIG value passed to a kernel timer (e.g., `timerfd_settime` arm-config) — that's data going INTO a kernel event source, which then becomes an fd-event. Honest.
- `Instant::now()` used for logging timestamps, measuring elapsed time AFTER the fact, or emitting telemetry — that's READING the clock, not waiting on it.
- `Duration` arithmetic in metric calculations — `elapsed.as_micros()`, ordering events by timestamp, etc.

**Honest blocking on events (NOT a sleep):**
- `crossbeam_channel::select! { recv(data) -> ..., recv(SHUTDOWN_RX) -> ... }` — blocks on channel events; no duration; honest.
- `ring.submit_and_wait(1)` on io_uring — blocks on kernel event completion; no duration; honest.
- `Receiver::recv()` (blocking) — wakes when channel-event fires; honest.
- `libc::poll(fds, nfds, -1)` (infinite timeout) — blocks on fd events; honest.
- `libc::epoll_wait(epfd, ..., -1)` (infinite timeout) — same.
- A future timerfd-arm in select — time arriving as an fd-event; honest (and the SHAPE this spell is preparing the ground for).

**Lines marked with the rune** `rune:mora(<category>) — <reason>` (see § "The rune" below). The rune is for genuine, attested exceptions only — not for hiding violations.

## The rune

A few situations may genuinely warrant a mechanism-based wait with no honest alternative:

- **Embedded/no-kernel context** — running on a target without timerfd/epoll/io_uring. Document the target.
- **Calibrated benchmark loops** — measuring throughput requires a known duration; the "sleep" IS the calibration. Document the measurement intent.
- **Interfacing with a third-party library** whose API demands a polling-with-sleep pattern. Name the library + cite the API constraint.

In each case, mark the line with the rune:

```rust
// rune:mora(calibration) — fixed-window throughput measurement; duration IS the metric basis
thread::sleep(Duration::from_secs(1));
```

The category names the WHY in one word:
- `calibration` — measurement requires known duration
- `external-api` — third-party constraint
- `no-kernel` — embedded/standalone target
- `pre-reactor` — substrate-internal placeholder until the reactor lands (use sparingly)

The reason after `—` explains the specific constraint. A rune without a reason is itself a Level 1 violation.

## The four questions

Mora's findings must pass through the four questions before they ship as code changes:

- **Obvious?** When this wait fails (races, hangs, varies in timing), will the failure mode reveal which event was being awaited? If the failure is "test sometimes fails after 5s timeout" with no further structure, the wait is opaque — Level 1. If the failure is "POLLHUP didn't fire on broadcast_fd within submit_and_wait window" with a clear event identification, the wait is honest.
- **Simple?** Is the wait expressible as ONE event-source select? If yes → honest. If the wait requires multiple sleeps + retries + escape conditions → that's mechanism layered on mechanism; remove the lot and use select.
- **Honest?** Does the wait name what it's waiting FOR? `recv()` waits for data. `select` waits for any-arm. `sleep(20ms)` waits for... 20ms? — that's the wait naming its OWN MECHANISM, not the event it claims to coordinate with. Dishonest.
- **Good UX?** Does the wait scale with kernel performance? An honest event-wait gets faster on faster kernels; a sleep takes the same time regardless. If the wait punishes a faster system, it's mechanism not event.

A wait that answers YES to all four runs cleanly. Any NO is a violation.

## What mora sees

### The canonical violation — sleep-as-race-mitigation

```rust
#[test]
fn it_works() {
    let (tx, rx) = pair::<String>().expect("pair");
    tx.send("hello".to_string()).expect("send");
    thread::sleep(Duration::from_millis(20)); // "give the kernel time"
    let result = rx.try_recv();
    assert_eq!(result, Ok("hello".to_string()));
}
```

The sleep is a lie. `libc::write(2)` is synchronous — the bytes ARE in the kernel pipe buffer when `tx.send()` returns. `libc::poll(timeout=0)` reads current kernel state — it sees POLLIN immediately. No timing assumption needed.

The honest shape:

```rust
#[test]
fn it_works() {
    let (tx, rx) = pair::<String>().expect("pair");
    tx.send("hello".to_string()).expect("send");
    // libc::write returns after bytes are in the kernel buffer;
    // try_recv via poll(timeout=0) sees POLLIN immediately.
    let result = rx.try_recv();
    assert_eq!(result, Ok("hello".to_string()));
}
```

No sleep. The wire delivers the synchronization.

### The other canonical violation — busy-wait dressed as a loop

```rust
let deadline = Instant::now() + Duration::from_secs(5);
loop {
    if condition_met() { break; }
    if Instant::now() > deadline { panic!("timeout"); }
    thread::sleep(Duration::from_millis(10));
}
```

Three violations stacked: the sleep is a guess, the deadline is mechanism (not event), the loop pretends to be a wait when it's actually polling. The honest shape: register the condition's event source (channel, fd, signal) and select over it with an optional timer-arm (when the reactor ships).

### A subtle case — process startup coordination

```rust
let child = Command::new("my-server").spawn().expect("spawn");
thread::sleep(Duration::from_secs(1)); // "wait for the server to come up"
// ... connect to the server ...
```

The sleep is hoping the server starts in <1s. It might. It might not. The honest shape: the server writes a "ready" marker to stdout (or a sentinel file, or pidfd-ready notification); the parent reads-until-marker before connecting. The wire IS the synchronization.

### Honest patterns (NOT flagged)

```rust
// Cascade-aware blocking recv. Wakes on data OR shutdown via crossbeam select.
let val = rx.recv()?;

// io_uring submit_and_wait — blocks on kernel completion event.
ring.submit_and_wait(1)?;

// Multi-arm POLL_ADD over [data_fd, broadcast_fd] — blocks until any fires.
let outcome = wait_for_data_or_cascade(read_fd, broadcast_fd)?;

// Logging timestamp — reading the clock, not waiting on it.
log::info!("operation took {:?}", start.elapsed());

// Future: timerfd as a select arm — time arriving as fd-event.
let timer = timerfd_create(...)?;
timer.arm(Duration::from_millis(100))?;
select.add_arm(timer.as_raw_fd());
let outcome = select.wait()?;
match outcome {
    Outcome::Data(d) => handle(d),
    Outcome::Timer => handle_timeout(),
    Outcome::Shutdown => unwind(),
}
```

## How to invoke

```
/mora path/to/file.rs
/mora src/comms/
/mora tests/
```

Mora walks the target, surfaces every wait-by-mechanism finding, classifies each (Level 1 violation, Level 2 mumble, rune-attested exception, or honest event-wait), and reports.

A finding is one of:
- **Level 1 violation** — a wait via mechanism (sleep, timeout-as-deadline, busy-wait) with no rune and no honest event-wait alternative ruled out. Must be removed or wired via event.
- **Level 2 mumble** — a wait that *might* be honest depending on context (e.g., a `Duration` near an assertion); requires the four-questions to settle.
- **Rune-attested exception** — `rune:mora(<category>) — <reason>` is present and the reason holds. Acknowledge; do not flag.
- **Honest event-wait** — channel-recv, fd-poll, ring-completion, etc. Not flagged.

## What mora is preparing

The wat reactor (future arc) will expose timerfd as a select-arm. After that lands, the honest "sleep" looks like:

```clojure
(let [timer (:wat::time::after 100ms)
      outcome (:wat::kernel::select' [data-rx timer])]
  (match outcome
    (:Data v)  (handle v)
    (:Timer)   (handle-timeout)))
```

Time arrives via the wire, exactly as the golang `select { case <-time.After(d): }` shape. Until then, mora rejects sleep-as-mechanism everywhere, and tests use the wire alone for coordination. After the reactor lands, mora amends its discipline: time-as-channel-event is honest; sleep-as-mechanism remains forbidden.

## Cross-references

- `/forge` — types enforce contracts; mora extends this discipline to time-as-type (events vs mechanisms)
- `/temper` — efficiency-debt at appropriate scales; sleep-as-mechanism is both inefficient AND incorrect
- `/complectens` — test composition; mora's findings often surface where complectens flags recurring patterns
- The golang `select { case <-time.After(d): ... }` pattern — the reference shape for time-as-channel-event
- Linux `timerfd_create(2)` + `epoll(7)` / `io_uring` — the kernel primitives for time-as-fd-event
