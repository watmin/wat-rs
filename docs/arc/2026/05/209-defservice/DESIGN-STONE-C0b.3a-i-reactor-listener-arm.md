# DESIGN-STONE C0b.3a-i — the reactor listener-arm + poll-driven non-blocking accept

> The reactor capability C0b.3a-ii's service loop needs: the autoscaling `comms::process::Select`
> ring must watch a **listen fd** for an incoming connection, alongside the data (Receiver) arms —
> one ring, the whole pile. And the accept path becomes **poll-then-non-blocking-accept** so it can
> never block (the deadlock hinge). C0b.3a-i ships the capability and dogfoods it by reworking
> standalone `accept'` onto it. C0b.3a-ii then wires the 3-arg `select'` process branch.

## Where we are (grounded, read this session)

- `comms::process::Select` (`process.rs:722`) — the autoscaling reactor: one lazy `RingSlot`,
  reflexive rebuild to `next_power_of_two(arm_count)`, `recv(rx)` registers a data arm, `select()`
  (`:813`) arms uniform `PollAdd` SQEs (broadcast `POLLHUP` token 0; data `POLLIN|POLLHUP` token
  i+1), one `submit_and_wait`, drains CQEs (broadcast wins ties), then reads the fired data arm via
  `rx.read_into_acc()`. Empty-guard at `:816` errors if no receivers + no broadcast.
- `SelectOutcome<T>` (`comms/mod.rs:778`) = `Shutdown | Recv { index, result }`. `pub`.
- `Select` is `pub`; reachable from integration tests (`tests/comms/process.rs:35` uses it).
- C0b.2c: `listener'` (process) binds a **blocking** `UnixListener`; standalone `accept'` (process)
  does a **blocking** `listener.accept()` (`runtime.rs` accept' process arm). `SocketListener'`
  opaque wraps the `UnixListener`.
- `Select::listener` / `SelectOutcome::Listener` are **grep-absent at HEAD** (the structural "RED").

## The contract decisions (pinned)

1. **Listener arm = `PollAdd POLLIN` on the listen fd** (NOT `IORING_OP_ACCEPT`) — four-questions
   verdict (C0b.3a parent design): one more uniform `PollAdd` arm; the accept is the act-after.
2. **The listen fd MUST be non-blocking** — the deadlock hinge. A spurious `POLLIN` (connection
   RST'd between poll and accept) must yield `EWOULDBLOCK` → re-poll, never a blocking `accept()`.
3. **Accept is poll-then-non-blocking-accept everywhere** — standalone `accept'` polls one fd (a
   `Select` with just the listener arm) then non-blocking-accepts; the C0b.3a-ii reactor polls N fds
   (incl. the listener arm) then non-blocking-accepts. One mechanism, scoped to the fd count.

## The mechanism

**Reactor (`comms::process` + `comms/mod.rs`):**
- `SelectOutcome<T>` gains a unit variant **`Listener`** (the listen arm fired; the caller accepts).
  ⚠️ This is a shared enum — every `match SelectOutcome` site must gain a `Listener` arm (the thread
  tier never produces it → `unreachable!`/error there). Compile the FULL test surface (the kill).
- `Select` gains `listener_fd: Option<RawFd>` + `pub fn listener(&mut self, fd: RawFd)` (one listener
  per service). In `select()`:
  - empty-guard (`:816`) also allows a listener arm (`receivers empty && broadcast none &&
    listener none` → error; a listener arm makes it non-empty).
  - `arm_count`/`needed_capacity` include the listener arm (+1).
  - push `PollAdd POLLIN` for `listener_fd` with a dedicated token (`LISTENER_TOKEN = u64::MAX`,
    outside the broadcast(0)/data(1..=N) range).
  - CQE drain: track `fired_listener`. Priority: **broadcast > data > listener** (shutdown first;
    serve existing clients before accepting new — prevents accept-flood starving existing). So:
    broadcast → `Shutdown`; else first data arm → `Recv`; else if `fired_listener` → `Listener`.

**Non-blocking accept (`runtime.rs`):**
- `listener'` (process): `set_nonblocking(true)` on the `UnixListener` after bind.
- `accept'` (process): build a `Select::<String>::new()` once, `sel.listener(fd)`, then loop:
  `select()` → `Listener` → `listener.accept()` → `Ok(stream)` wrap as `SocketPeer'` (reuse
  `wrap_stream_as_socket_peer`) | `WouldBlock` → re-`select()` (same `sel`, ring reused) | `Err` →
  clean error; `Shutdown` → clean error (shutdown during accept). Observable behavior unchanged
  (blocks until a connection), now poll-driven + deadlock-safe. Dogfoods the listener-arm.

## Gate-shape (honest — a primitive addition, not a wat composition)

There is **no pre-committed wat-surface RED probe** for this stone: the listener-arm has no wat
surface until C0b.3a-ii, and a test of the new `Select::listener`/`SelectOutcome::Listener` cannot
compile at HEAD (the API is absent) — so it can't be committed RED-first. The disconfirming fact is
structural: **the API is grep-absent** (verified). The gate is:
1. A Rust reactor test (`tests/comms/process.rs`, `Select` is pub) shipped WITH the impl:
   bind a non-blocking abstract-UDS `UnixListener`, `Select::listener(fd)`, a thread `connect`s,
   `select()` returns `SelectOutcome::Listener`. (FM-2-bis's own escape clause: a substrate-primitive
   addition is built-then-tested, not probe-disconfirmed.)
2. **Regression: `probe_arc209_c0b2c` stays GREEN** — standalone `accept'` now routes through the
   `Select` listener-arm + non-blocking accept; its still-passing round-trip is the end-to-end proof
   the listener-arm works through the wat verbs.

## Files touched

- `src/comms/mod.rs` — `SelectOutcome::Listener` variant.
- `src/comms/process.rs` — `Select.listener_fd` + `Select::listener` + the `select()` listener arm
  (guard, capacity, SQE, CQE-priority); the reactor unit test.
- `src/runtime.rs` — `listener'` (process) `set_nonblocking`; `accept'` (process) poll-then-accept
  rework.
- (ripple) every `match SelectOutcome` site gains a `Listener` arm — grep `SelectOutcome::` to find
  them (`runtime.rs` select' 1-arg/3-arg-thread, brackets coordinator); thread-tier → `unreachable!`.

## Out of scope = rejected (named, not deferred)

- **The 3-arg `select'` process branch + service loop** — C0b.3a-ii (the listener-arm's real
  consumer; the wat-surface RED lives there).
- **`IORING_OP_ACCEPT`** — rejected (four-questions).
- **`SO_PEERCRED`** — C0b.3b.
- **The lazy-Receiver-ring optimization** — closed as a non-issue (no idle rings;
  [[feedback_curated_note_mechanism_must_be_grounded]]).

## The deadlock contract carries

The non-blocking listen fd is the load-bearing invariant: the reactor's accept can never block, so a
service loop polling the listener arm can never hang on a spurious wakeup. [[feedback_vended_primitives_never_deadlock]]
