# DESIGN-STONE C0b.3a-ii — the socket `poll'` service multiplexer (process-tier service loop)

> The deadlock-surface culmination of the process tier. Adds the PROCESS branch to
> `poll'` (the 3-arg service multiplexer): self-peer + listener + N clients over ONE
> `process::Select` ring → `ServiceEvent<I,O>`, mirroring the proven thread template
> (C0b.1b) + reusing the shipped process primitives (C0b.3a-i `Select::listener` +
> poll-accept; C0b.2e-ii the proper `Listener`; i-a `ReactorClass`/`as_any`). This is the
> real defservice socket service loop — DEADLOCK-INTOLERABLE; the termination contract is
> pinned below and the gate is a termination-on-owner-drop probe.

## The naming correction (pinned — no `Option<RawFd>`)

`CommListener` gains **`reactor_class() -> ReactorClass`** (the i-a named enum, reused —
`InMemory` crossbeam / `Fd` socket), NOT `listen_fd() -> Option<RawFd>` (which would smuggle
"in-memory" into `None` — the abuse i-a already named away for `CommReceiver`). The listen
fd is recovered the same way the Peer receiver's fd is: via the **`as_any_ref` already on
`CommListener`** → downcast to `SocketListener` → `.listener.as_raw_fd()`. One named
discriminant (`ReactorClass`) across the whole connection surface: Receiver, Listener, and
any future remote.

## Grounded this session (HEAD `4750f4f2`)

- `eval_poll_prime` (runtime.rs ~23873) is **thread-tier only**: it `as_any`-downcasts the
  self-peer + client peer receivers to `comms::thread::Receiver<Value>` and the listener to
  `CrossbeamListener.rx`, errors on any socket-backed input ("socket/remote poll' is
  C0b.3a-ii"). The thread Select build + `ServiceEvent` mapping + termination (runtime.rs
  ~24025-24135) is the TEMPLATE:
  - `thread::Select`: index 0 = self-peer, index 1 = listener (a Receiver), 2..N+1 = clients.
  - `Recv{0}` → **`ServiceEvent::Shutdown`** (do NOT inspect result — the RAII drain wakes it).
  - `Recv{1}` → `wrap_connect_request` → `ServiceEvent::Connection{peer}`.
  - `Recv{k≥2}` → `Ok`→`Message{idx:k-2,msg}` / `Err`→`Closed{idx:k-2}`.
- Process primitives shipped: `process::Select::listener(fd)` + `SelectOutcome::Listener` +
  poll-driven non-blocking accept (C0b.3a-i); the process self-peer (`:wat::program::self-peer`,
  C0b.3a-0) installs a socket `Peer` over fd0(rx)/fd1(tx); the proper `Listener`/`SocketListener`
  (C0b.2e-ii); `CommReceiver::reactor_class`/`as_any` (i-a).
- `process::Select` 1-arg usage (runtime.rs ~23745) shows the pattern: collect
  `&process::Receiver<String>`, `sel.recv(rx)`, `sel.select()` → decode. (Now `Value` not
  `String`, post i-0/i-b.)

## The contract decision (pinned)

**(1) `CommListener::reactor_class`** (named, reused) + impls: `CrossbeamListener → InMemory`,
`SocketListener → Fd`. The fd via `as_any_ref` (already present).

**(2) `eval_poll_prime` branches by `reactor_class` homogeneity** (the self-peer + listener
+ all client receivers): all `InMemory` → the existing thread path; all `Fd` → the NEW
process path; **mixed → clean error** (a service's links are one tier — not a representable-
good state).

**(3) The process path — `process::Select` over the socket tier:**
```text
sel = process::Select::new()
sel.recv(self_peer_rx)         // index 0 = self-peer (owner link)
for c in clients: sel.recv(c_rx)   // indices 1..=N = clients   ⟵ NB: NOT +2; listener is the accept-arm
sel.listener(socket_listener.listener.as_raw_fd())   // the accept-arm (SelectOutcome::Listener)
match sel.select()? {
  Recv{index:0, ..}        → ServiceEvent::Shutdown          // self-peer EOF = owner dropped (RAII drain IS the wake) — DO NOT inspect result
  Recv{index:k≥1, result}  → Ok→Message{idx:k-1,msg} / Err→Closed{idx:k-1}
  Listener                 → accept (non-blocking, poll-driven) → wrap stream as Peer → Connection{peer}
}
```
Index mapping differs from the thread template: process self-peer=0, clients=1..N (Recv),
listener=the `Listener` accept-arm (so `Message.idx = index-1`, not `index-2`). `:Lost`
(transport-break with a cause) stays remote-tier-only (process bare EOF = `:Closed`).

**(4) THE TERMINATION CONTRACT (deadlock-free, pinned — same as C0b.1b):** the self-peer is
Select index 0. The owner dropping the `spawn-program'` handle → the child's input pipe
(fd0) closes → the self-peer's `process::Receiver` sees EOF → `process::Select` fires
`Recv{0}` → `poll'` returns `:Shutdown` → the service loop exits → RAII reaps. **The drain
the runtime ALREADY runs on owner-drop IS the wake — no cooperative Stop, no separate
shutdown channel.** This is the whole point; the gate proves it (termination-on-drop).

## The gate (the deadlock-surface gate — termination-on-drop is load-bearing)

**RED probe** `probe_arc209_c0b3aii_process_service_loop` (write + verify RED at HEAD):
a `(process)` service that `(listener' (process) addr)` + named-recurses a `poll'`-loop
over `(self-peer, listener, clients)` — `:Connection`→grow, `:Message`→serve (e.g. echo
`n→n+1`)+reply, `:Closed`→shrink, `:Shutdown`→return (exit). A parent spawns it, `connect'`s
a client by name, sends, receives the reply; then **drops the service handle and the join
must complete (no hang)** — the termination-on-owner-drop assertion is the deadlock gate.
RED at HEAD (socket `poll'` errors "C0b.3a-ii"); GREEN after.

**Regression:** `c0b1b` (thread `poll'` service loop — the thread path must stay byte-green),
`connection_primitive`, `c0b2c` (process listener/accept/connect), `c0b3a0` (process
self-peer). Nursery serial **895/4** (baseline only) + full workspace compiles.

## Files touched

`src/kernel/listener.rs` (`CommListener::reactor_class` + 2 impls), `src/runtime.rs`
(`eval_poll_prime`: reactor_class branch + the process path; the thread path unchanged),
the new probe. Possibly `src/comms/process.rs` ONLY if `Select` needs a read-after-`Listener`
accommodation — expected NONE (the primitives are shipped). No checker change (`poll'` is
tier-blind on `Peer'`/`Listener'`). No `comms` trait change beyond what i-a already ships.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1 (the deadlock one):** the process service loop does NOT terminate on owner-drop
   (the probe hangs) — STOP, capture forensics (gdb bt + `/proc/<pid>/task/*/{stack,wchan}`),
   report; do NOT ship a cooperative-Stop workaround (that races the drop — the C0b.1b
   lesson). The fix must be structural (the self-peer EOF IS the wake).
2. **STOP-2:** `process::Select` cannot register the self-peer + clients (Recv) AND the
   listener (accept-arm) in ONE ring → STOP, report (C0b.3a-i shipped the listener-arm; this
   should compose).
3. **STOP-3:** mapping a fired client `Recv{k}` back to its peer index is ambiguous (registration
   order ≠ index) — STOP, report (the thread template proves the order-preserving mapping).

## Out of scope (rejected — NOT deferred)

- `Address'` proper entity (raw-`Sender`/`SocketAddress'` fictions → one `Address'`) = **C0b.2e-iii**.
- SO_PEERCRED allow-set = **C0b.3b**. Remote (`AF_INET`+mTLS) = a later `CommSender`/`CommReceiver`/`CommListener` impl.
- `:Lost` (transport-break cause) population = remote-tier only (process EOF = `:Closed`).

## The deadlock contract carries — and is the gate

This stone IS the deadlock-surface. The termination contract (4) is structural (self-peer
EOF = the RAII-drain wake); the gate (termination-on-drop probe) proves it; the weigh
re-runs it. Mirror C0b.1b verbatim; never a cooperative Stop. [[feedback_vended_primitives_never_deadlock]]
[[feedback_race_fix_structural_not_reproduced]] [[feedback_capture_before_kill_rare_state]]
