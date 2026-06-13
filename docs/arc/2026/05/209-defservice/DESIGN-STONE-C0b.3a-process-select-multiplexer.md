# DESIGN-STONE C0b.3a — the PROCESS-tier 3-arg `select'` service multiplexer

> The fifth rung of the process connection tier (`DESIGN-STONE-C0b.2-process-connection-tier.md`).
> C0b.2c shipped the process connection verbs (`listener'`/`connect'`/`accept'` → `SocketPeer'`).
> C0b.3a makes a process **service** possible: the 3-arg `select'` that multiplexes self-peer +
> listener + N client `SocketPeer'`s over ONE `comms::process::Select` ring, returning the same
> `SelectEvent<I,O>` sum as the thread tier, deadlock-free on owner-drop. After this:
> C0b.3b (SO_PEERCRED) → thread+process **parity** → Stone C (defservice).

## The template (thread tier, C0b.1b — grounded, read this session)

`eval_peer_select_prime_3arg` (`runtime.rs:23766`) + the probe
`probe_arc209_c0b1b_select_listener.rs`. The service loop is named recursion:
`(select' self l clients)` → `match` the `SelectEvent`:
- index 0 = **self-peer** `.rx` → `:Shutdown` (owner drops the handle → RAII drain → the link closes → the arm fires)
- index 1 = **listener** → `:Connection [peer]` (accept + wrap; `conj` the new peer — GROW)
- index 2.. = **client peers** → `:Message [idx msg]` / `:Closed [idx]` (EOF → shrink)

Thread tier uses `comms::thread::Select` (crossbeam). `SelectEvent<I,O>` =
`:Shutdown | :Connection[peer<-Peer'<I,O>] | :Message[idx,msg<-O] | :Closed[idx] | :Lost[idx,cause]`.

## What the process tier mirrors

Same `SelectEvent` sum, same loop shape, over a **heterogeneous fd set on one ring** (the
`comms::process::Select` reactor — `process.rs:722`, the autoscaling reflexive-rebuild ring):
- **self-peer** = the service's own input **pipe** (`comms::process::Receiver`); owner-drop → pipe
  **EOF** → `:Shutdown`. (Thread's crossbeam-disconnect becomes pipe-EOF — same "owner link closed"
  structure; deadlock-free-on-drop carries.)
- **listener** = the `SocketListener'` (UnixListener fd) → the reactor watches it for an incoming
  connection → `accept()` → wrap as `SocketPeer'` (reuse `wrap_stream_as_socket_peer`) → `:Connection`.
- **clients** = the `SocketPeer'` read fds → data → `:Message`, EOF → `:Closed`.

## The contract decision (LOCKED via four-questions, 2026-06-13)

**The listener arm is `PollAdd POLLIN` on the listen fd, then `std accept()` — NOT `IORING_OP_ACCEPT`.**
Four-questions verdict: option A (POLL_ADD) wins **Obvious** (one more `PollAdd` arm, identical to
every existing arm; act-by-fired-arm already exists for data/broadcast) and **Simple** (one SQE kind
across the wait; one new act-branch; no new CQE-drain semantics). `IORING_OP_ACCEPT` (B) loses both
(novel op kind + branched CQE-drain) and its only edges — one fewer syscall per accept, no
blocking-mode juggling — are tiebreaker-tier on the *rare* accept path, and Obvious+Simple gate
before UX. Full analysis in the session log.

### The pinned invariant (the real deadlock hinge)

**The listen fd MUST be non-blocking.** A blocking `accept()` after a spurious `POLLIN` (connection
RST'd between poll and accept) would hang the service loop — a deadlock the doctrine forbids
([[feedback_vended_primitives_never_deadlock]]). Non-blocking → `EWOULDBLOCK` → re-poll, never blocks.
This is *the* deadlock-safety property of the process service loop, not the accept-op choice.

### The non-blocking resolution (matures C0b.2c — coherent, not grave-patching)

C0b.2c's `listener'` (process) binds a **blocking** UnixListener and its standalone `accept'` does a
**blocking** `listener.accept()`. The non-blocking invariant forces a uniform rework, which C0b.3a
delivers:
- `listener'` (process) sets the UnixListener **non-blocking** at bind (`set_nonblocking(true)`).
- accept becomes **poll-then-non-blocking-accept everywhere** — the same machinery, scoped to the fd
  count: standalone `accept'` polls ONE fd (the honest wire-wait) then non-blocking-accepts; the
  reactor polls N fds (incl. the listener arm) then non-blocking-accepts the fired listener.
- This UNIFIES accept with the rest of the reactor (poll-for-readiness then act — exactly how `recv`
  is poll-then-read). Standalone `accept'` keeps its observable behavior (blocks until a connection),
  now via poll. The accept primitive is shared by `accept'` and the multiplexer.

## Decomposition (stepping stones — split because ii operates on i's settled reactor)

- **C0b.3a-i — accept becomes poll-driven non-blocking + the reactor listener-arm.**
  - `comms::process::Select` gains a listener arm: `fn listener(&mut self, fd: RawFd)` (register the
    listen fd as a poll-only `PollAdd POLLIN` arm) + `SelectOutcome::Listener` (that arm fired). The
    reactor reports readiness; it does NOT accept (the runtime owns wat-value construction).
  - `listener'` (process): `set_nonblocking(true)` at bind.
  - `accept'` (process): rework to poll-one-fd-then-non-blocking-accept (reuse the reactor scoped to
    one listener arm, or a one-fd poll helper) — observable behavior unchanged (blocks until a
    connection), deadlock-free.
  - **Gate:** the C0b.2c standalone round-trip (`probe_arc209_c0b2c`) stays GREEN (accept now
    poll-driven); a unit test of the reactor listener-arm (poll fires when a connection is pending).
- **C0b.3a-ii — the 3-arg `select'` process branch (the service loop).**
  - `eval_peer_select_prime_3arg`: dispatch when arg1 is a `SocketListener'` (vs the thread
    `Receiver`). Build a `comms::process::Select`; register self-peer's process `Receiver` (idx 0) +
    client `SocketPeer'` Receivers (idx 1..) + the listener arm. Map: `Recv{index:0}` (EOF) →
    `:Shutdown`; `Listener` → accept + `wrap_stream_as_socket_peer` → `:Connection`; `Recv{k≥1}` Ok →
    `:Message{k-1}`, Err → `:Closed{k-1}`.
  - `check.rs` `infer_select_prime_3arg` (`:10771`): handle the process tier (arg1 `SocketListener'`,
    clients `SocketPeer'`) → `SelectEvent<I,O>`. Confirm whether it's already peer-kind-generic
    (it extracts I,O from the peers) or needs a SocketListener'/SocketPeer' branch.
  - **Gate:** the process-tier twin of the C0b.1b probe — a process service loop: spawn the service,
    `connect'` clients across the process boundary, grow/serve/shrink, owner-drop → clean shutdown,
    deadlock-free. The hardest gate in the campaign.

## ⛔ RESOLVE FIRST (crawl at draw time — do NOT guess)

**How does a process-tier service obtain its self-peer value (arg0 to `select'`-3arg)?** The thread
template passes a `Peer'<I,O>` the worker received via `spawn_thread_peer`'s self-peer handoff. The
process analog: does `spawn-program' (process)` hand the child a self-peer handle (its own input pipe
as a `Process'`/`SocketPeer'`-shaped value)? Crawl `spawn_process`/the spawn-program' process clause
+ how the child's input pipe is exposed to wat. This determines what the process service passes as
arg0 and what the C0b.3a-ii probe constructs. Ground it before drawing C0b.3a-ii.

## Out of scope = rejected (named, not deferred)

- **`SO_PEERCRED` allow-set** — C0b.3b (security model LOCKED, `DESIGN-STONE-C0b-SECURITY.md`).
- **`(remote)` AF_INET / mTLS** — guaranteed by the AF_UNIX clause; built when `:remote` arrives.
- **defservice defmacro (Stone C)** — consumes this; blocked until parity (C0b.3b done).
- **`IORING_OP_ACCEPT`** — rejected above (four-questions); the listener arm is `PollAdd` + std accept.

## The deadlock contract carries

The service loop terminates on owner-drop, structurally: owner drops the spawn handle → RAII drain →
the self-peer's input pipe EOFs → the reactor's index-0 arm fires → `:Shutdown` → loop returns →
clients drop → the process exits → the owner's reap completes. No cooperative stop. Plus the pinned
non-blocking-listener invariant so the accept arm can never block. [[feedback_vended_primitives_never_deadlock]]
