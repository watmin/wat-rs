# BRIEF — crash propagation, IN-BAND + tier-symmetric (SUPERSEDES BRIEF-crash-channel-on-connection-peer.md)

> **This supersedes `BRIEF-crash-channel-on-connection-peer.md`** (the side-channel design proven in `stash@{0}`).
> That design worked but was **over-engineered on the thread tier**: it minted a *new String crash channel* and
> shipped it across `connect'` (patching `address.rs` + `listener.rs` + the rendezvous 3-tuple + a RustOpaque) — when
> the **reply channel (`resp_tx`) already sits on the service side** at `accept'`. The builder called it: *"we over-
> engineered how to communicate crash reason on threads; processes we just need to use what we built; threads and
> processes should be basically identical in everything but shared memory."* This brief reuses the EXISTING data
> channel — no new channel, no `connect'` crossing — and does the identical in-band thing on both tiers.

## The gap (PROBED + confirmed this session — grounded)
A connect'd client's connection `Peer'` has NO crash channel *by construction* (`runtime.rs:26623`). When a service
crashes mid-request, its `conn` (holding `resp_tx` to the client) is dropped during the `EvalBreak` unwind, and the
client's `recv'` reports only `recv failed: peer closed / channel disconnected`. **The real reason is captured but
lost.** Probe (`scratchpad/crashprop/scratch_crashprop_probe.{wat,rs}` — the c0b1 connection shape, handler crashes
via `assertion-failed! "BOOM-CRASH-REASON-42"` between `recv'` and `send'`):
- service thread emits `#wat.kernel/AssertionFailure {:message "BOOM-CRASH-REASON-42" ...}` (reason IS produced);
- client `recv'` sees `#wat.runtime/MalformedForm ... "channel disconnected"` (reason IS lost). **GAP CONFIRMED.**

**Decisive structural fact (grounded):** a thrown `RuntimeError` is an `EvalBreak`, and `try` only propagates
`Result`/`Option` *values* (`TryPropagate`, `runtime.rs:12561`) — there is **no wat-level catch** for a thrown crash.
So the fix CANNOT be a wat serve-loop change; it needs a sender that **survives the unwind** (lives outside the wat
env). `stash@{0}` already proved a thread-local survives the `EvalBreak` unwind and can fan a reason to the client
before EOF (ran green: comms 77/77, kernel 402/402). This brief keeps that proven mechanism and DELETES the
new-channel-across-`connect'` half.

## The design — reuse `resp_tx`, in-band, both tiers

**One reserved crash-sentinel frame** rides the EXISTING data channel as the connection's final frame; `recv'`/`select'`
recognize it and raise `Crashed(reason)` (the `PeerRecvError::Crashed` arm already exists, `runtime.rs:25797`; the
worker-peer path already surfaces it). Reply-XOR-crash per request (mirrors the process `err`'s Ok-XOR-Err) → no
concurrency.

### THREAD tier
1. **`accept'` stashes a CLONE of `resp_tx` in a thread-local** `ACTIVE_CONN_SENDERS` (on the service thread) right
   where the connection peer is built — `src/kernel/listener.rs:241` (`Ok(Peer::from_thread(resp_tx, req_rx))`) AND
   its twin in `wrap_connect_request` (`src/runtime.rs`, the `poll'`/2-arg-`select'` path). `resp_tx` is
   `comms::thread::Sender<Value>` — `Clone` (crossbeam). The clone keeps the channel ALIVE past the wat `conn`
   binding's unwind-drop, so the client's `resp_rx` does not EOF until we send + drain.
2. **`spawn_thread_peer`'s catch drains + sends the sentinel.** In BOTH death arms (`src/kernel/spawn.rs:710` panic,
   `:721` `Ok(Err(re))`), BEFORE the existing owner `crash_tx.send(reason)`: drain `ACTIVE_CONN_SENDERS`, and on each
   clone `send` a reserved crash-sentinel `Value` carrying `reason` (a `Value::Enum` with a reserved `type_path`, e.g.
   `:wat::kernel::__PeerCrash__ {reason}`). Same send-before-drop discipline `stash@{0}` proved.
3. **`recv'`/`select'` recognize the sentinel.** In `eval_peer_recv_prime` (`runtime.rs:25797`) and the `select'`
   bare-`Peer'` arm (`runtime.rs:26623-26645`, the "no crash channel" branches): if the received `Value` is the
   reserved crash sentinel → raise `Crashed(reason)` (mirror the worker-peer `Crashed` arm) instead of returning it.

### PROCESS tier — "just an edn string" (builder steer)
The crash reason is already an EDN string written to fd2/stderr by the panic hook at `emit_structured_exit`
(`src/process/verbs.rs:126`, the single crash-exit tail before `libc::_exit`) — the child STILL HOLDS its client
sockets there (process fds stay open until `_exit`; no unwind-drop problem). Symmetric with the thread tier:
1. The process serve child registers each accepted client socket's `Sender<String>` in a process-global/thread-local
   `ACTIVE_CONN_SOCKETS` (mirror `ACTIVE_CONN_SENDERS`).
2. At `emit_structured_exit`, BEFORE `_exit`: write the SAME crash-sentinel as a final EDN frame on each active client
   socket (the data channel is `comms::process::Sender<String>`, EDN frames — the sentinel is one reserved EDN form).
3. The client's process-tier `recv'` decodes the sentinel frame → `Crashed(reason)` (same recognition as the thread
   sentinel; the frame decode already runs in `recv'`).

### Recognition is ONE predicate, shared
The sentinel check (`is this the reserved __PeerCrash__ shape? → extract reason`) is written ONCE and consumed by both
tiers' `recv'`/`select'` — anti-drift, the way `classify_peer_death` (`spawn.rs:204`) is already tier-generic.

## Why it's simpler than `stash@{0}` (the deletions)
- **NO** new crash channel minted at `connect'`; **NO** `address.rs` change; **NO** `listener.rs` rendezvous 3-tuple
  change; **NO** `PROBE_CRASH_TX_TYPE_PATH` RustOpaque; **NO** `Peer.crash` field / `from_thread_with_crash` /
  `recv_or_crash`. The `resp_tx` clone reuses the channel that already exists on the service side.
- Thread and process now do the **identical** thing (a final in-band sentinel frame on the data channel) — R31/R32
  loci-agnostic, "identical but for shared memory."

## PROBE-FIRST (before the full build — MANDATORY)
The gap is already probed (above). Probe the MECHANISM delta before wiring both tiers + `select'`:
- Minimal thread wiring ONLY: `accept'` clones `resp_tx` into `ACTIVE_CONN_SENDERS`; `spawn_thread_peer` panic-arm
  drains + sends the sentinel; `eval_peer_recv_prime` recognizes it. Run the preserved probe (flip its assert to
  EXPECT `BOOM-CRASH-REASON-42`). GREEN = the in-band reuse works → proceed to `select'` + process tier.
- **STOP + report if:** the `resp_tx` clone does NOT keep the client channel alive past the wat `conn` drop (client
  EOFs before the sentinel arrives), or the sentinel `Value` can't be distinguished from a legitimate reply, or the
  send races the drain. `stash@{0}` proved the survives-unwind half, so this should hold — but prove the *reuse*
  variant, do not assume it.

## The build (only after the mechanism probe is green)
Thread tier (1–3 above) → `select'` parity (`runtime.rs:26623-26645`) → process tier (emit_structured_exit write +
socket registry + client decode) → the shared sentinel predicate. Keep the existing owner `crash_tx`/`err` paths
UNTOUCHED (they work for spawned workers; this only ADDS the connect'd-client in-band frame).

## STOP triggers / constraints
- DO NOT touch the worker-peer crash path (`spawn_thread_peer`'s `crash_tx` → owner) beyond ADDING the drain-before-
  send. DO NOT weaken `recv'`/`select'`'s existing `Crashed` handling.
- DO NOT reintroduce a channel across `connect'` — if you find yourself editing `address.rs`/`listener.rs`'s rendezvous
  tuple, STOP: the design reuses `resp_tx`, which is already service-side.
- If the mechanism probe shows reuse is racy/blocked, STOP and escalate — `stash@{0}` (the side-channel design) is the
  proven fallback; do not hack a partial.

## Gate (orchestrator re-runs ALL)
- `cargo build --release` clean.
- Promote the preserved probe → `tests/comms/probe_arc294_crashprop_inband.{wat,rs}` (thread) + a process twin
  (`tests/process/…`), asserting the client's `recv'` surfaces the REAL reason (the fn/marker), NOT
  `ChannelDisconnected`. Both GREEN.
- `select'` twin: a crashing service + a client blocked in `select'` surfaces `Crashed`, not `Closed`.
- WHOLE FLOOR `cargo nextest run --release`: ZERO NEW failures vs baseline `9e9b778c` (~52). Report the set diff.

## Method
Build/test ONCE to a temp file, grep the FILE. Rebuild before running `target/release/wat`. A mid-edit diagnostic is a
PHANTOM. Commit nothing until the orchestrator weighs it.

## Report back
The mechanism-probe result FIRST (does in-band reuse of `resp_tx` deliver the reason?), then the diff summary, the two
new-test results, the `select'` result, the whole-floor set diff, any STOP hit.
