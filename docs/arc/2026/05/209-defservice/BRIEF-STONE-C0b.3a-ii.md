# BRIEF — Stone C0b.3a-ii: the socket `poll'` service multiplexer (process-tier service loop)

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/`
(verify `pwd`; operate only here; `git -C /home/watmin/work/holon/wat-rs`). Design:
`DESIGN-STONE-C0b.3a-ii-socket-poll-service-multiplexer.md` (read it fully). The RED probe
is already on disk + verified RED. Do NOT commit — the Inquisitor weighs. **This is the
DEADLOCK-SURFACE stone — the termination contract below is load-bearing.**

## The work in one paragraph

Add the PROCESS branch to `poll'` (the 3-arg service multiplexer). Today `eval_poll_prime`
(runtime.rs ~23873) is thread-tier only — it `as_any`-downcasts the self-peer + client
receivers to `comms::thread::Receiver<Value>` + the listener to `CrossbeamListener`, and
errors on any socket-backed input. Branch it by `reactor_class`: all-`InMemory` → the
existing thread path (unchanged); all-`Fd` → a NEW process path over `comms::process::Select`
(self-peer fd + client fds via `Select::recv`, the listener fd via `Select::listener`) →
`ServiceEvent`; mixed → clean error. First add `reactor_class()` to `CommListener` (the i-a
named `ReactorClass`, NOT `Option<RawFd>`) + the two impls. The gate probe
`probe_arc209_c0b3aii_process_service_loop` (already on disk) is a spawned process service
that `poll'`-loops and must terminate when the owner drops the handle.

## THE TERMINATION CONTRACT (deadlock-free — pinned, mirror C0b.1b verbatim)

The self-peer is `process::Select` index 0. The owner dropping the `spawn-program'` handle →
the child's input pipe (fd0) EOFs → the self-peer's `process::Receiver` fires `Recv{0}` →
`poll'` returns `ServiceEvent::Shutdown` (do NOT inspect the result) → the service loop
exits → RAII reaps. **The RAII drain the runtime already runs on owner-drop IS the wake —
NO cooperative Stop, NO shutdown channel, NO sleep.** This is the whole point. If the loop
does not terminate on owner-drop, that is **STOP-1** — capture forensics (`gdb` bt +
`/proc/<pid>/task/*/{stack,wchan}`), report; do NOT ship a cooperative-Stop band-aid (it
races the drop — the exact C0b.1b failure that was gdb'd live).

## Read in order (the rooms)

1. `src/kernel/listener.rs` — `CommListener` (add `reactor_class()`) + `CrossbeamListener`
   (`InMemory`) + `SocketListener` (`Fd`); `as_any_ref` already exists (use it for the fd:
   downcast to `SocketListener` → `.listener.as_raw_fd()`).
2. `src/runtime.rs` ~23873 `eval_poll_prime` — the thread template (Select build at ~24025;
   `ServiceEvent` mapping + the self-peer→`:Shutdown` termination at ~24050-24135). The
   process path mirrors this over `process::Select`.
3. `src/runtime.rs` ~23745 — the process `select'` 1-arg arm (the `process::Select` recv
   pattern: collect `&process::Receiver<Value>`, `sel.recv(rx)`, `sel.select()` → decode).
4. `src/comms/process.rs` `Select` — `recv(&Receiver)`, `listener(fd)` (C0b.3a-i),
   `select() -> SelectOutcome` (`Recv{index,result}` | `Listener` | `Shutdown`), and the
   poll-driven non-blocking accept used by `accept'`.
5. `src/comms/mod.rs` `ReactorClass` (i-a) + `CommReceiver::reactor_class`/`as_any`.
6. `tests/probe_arc209_c0b3aii_process_service_loop.rs` — the gate (already on disk).

## Implementation sketch (fill the shape)

**(A) `CommListener::reactor_class`:**
```rust
fn reactor_class(&self) -> crate::comms::ReactorClass;   // CrossbeamListener → InMemory, SocketListener → Fd
```
**(B) `eval_poll_prime` — branch by reactor_class homogeneity.** Compute the class of the
self-peer rx + listener + each client rx (via `reactor_class()`). All `InMemory` → the
existing thread path. All `Fd` → the process path. Mixed → clean `MalformedForm` error.
**(C) The process path** (`process::Select`):
```text
let mut sel = process::Select::new();
sel.recv(self_peer_rx);                  // index 0 = self-peer  (as_any → &process::Receiver<Value>)
for c in client_rxs { sel.recv(c); }     // indices 1..=N = clients   (NB: NOT +2)
sel.listener(socket_listener.listener.as_raw_fd());   // the accept-arm
match sel.select()? {
  SelectOutcome::Recv{index:0, ..}       => ServiceEvent::Shutdown,            // DO NOT inspect result
  SelectOutcome::Recv{index:k, result}   => Ok→Message{idx:k-1, msg} / Err→Closed{idx:k-1},
  SelectOutcome::Listener                => { non-blocking accept (poll-driven, as accept' does) → wrap as Peer → Connection{peer} }
  SelectOutcome::Shutdown                => the substrate-shutdown error arm (as the thread path has it),
}
```
Recover the concrete receivers via `peer.rx.as_any().downcast_ref::<process::Receiver<Value>>()`
(i-a). Build the `ServiceEvent` enum values exactly as the thread path does (same
`SELECT_EVENT_TYPE`/`ServiceEvent` variants + fields).

## Blast radius

`src/kernel/listener.rs` (`reactor_class` + 2 impls), `src/runtime.rs` (`eval_poll_prime`
reactor_class branch + the process path; thread path UNCHANGED). The probe exists.
Expected NO `comms` change (the primitives are shipped); if `process::Select` genuinely
can't compose self-peer+clients (recv) AND the listener (accept-arm) in one ring, STOP-2.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1 (THE DEADLOCK ONE):** the gate probe HANGS (the loop doesn't terminate on
   owner-drop) — STOP, capture forensics (gdb bt + `/proc/<pid>/task/*/{stack,wchan,syscall}`),
   report. Do NOT add a cooperative Stop / shutdown channel (it races the drop — C0b.1b
   proved this). The fix is structural: the self-peer EOF (`Recv{0}`) IS the wake.
2. **STOP-2:** `process::Select` can't register self-peer+clients (recv) AND the listener
   (accept-arm) in ONE ring — STOP, report (C0b.3a-i shipped the listener-arm; expected to compose).
3. **STOP-3:** a fired client `Recv{k}` can't map back to its peer index (registration order
   ≠ index) — STOP, report (the thread template proves the order-preserving mapping; process
   uses k-1 because the listener is the accept-arm, not a recv index).

## The gate

```
cargo test --release -p wat --test probe_arc209_c0b3aii_process_service_loop -- --test-threads=1   # GREEN (105 + terminates)
cargo test --release -p wat --test nursery probe_arc209_c0b1b -- --test-threads=1                  # thread poll' UNCHANGED
cargo test --release -p wat --test nursery connection_primitive -- --test-threads=1                # 1-arg select' unchanged
cargo test --release -p wat --test probe_arc209_c0b2d_named_cross_process                          # cross-process connect-by-name intact
cargo test --release -p wat --test nursery -- --test-threads=1                                     # 895 passed / 4 failed (baseline)
cargo test --release --workspace --no-run                                                          # full surface compiles
```
If the gate probe HANGS rather than passing/failing — that is STOP-1 (the deadlock); do not
wait it out, capture forensics. Report each exact `test result:` line + any STOP/honest delta.
Do NOT commit.

## Prior comparable (copy the shape)

The thread template is `eval_poll_prime`'s existing thread path (runtime.rs ~24025-24135) —
mirror its Select-build + `ServiceEvent` mapping + the self-peer→`:Shutdown` termination over
`process::Select`. `BRIEF-STONE-C0b.2e-ii.md` (the `CommListener` + reactor_class pattern).
The probe mirrors `probe_arc209_c0b1b_select_listener` (thread serve-loop) +
`probe_arc209_c0b2d_named_cross_process` (process service by name + self-peer READY).
