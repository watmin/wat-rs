# BRIEF — Stone 6b-ii-α: wire the parent→child lineage send for the forms-server

> For a single-hop sonnet Shadowdancer. Do NOT spawn sub-agents. Work only in
> `~/work/holon/wat-rs`. Commit nothing; the orchestrator weighs your diff + re-runs the gate itself.
> Grounded against HEAD `9edf0b2f`, branch `arc-170-gap-j-v5-deadlock-state`.

## The work (one paragraph)

A spawned `(process)` child can send to its owner over the lineage channel (the self-peer) — proven —
but the OWNER cannot send back to the child over that same channel. `(send' svc state0)` from the parent
fails with `"send failed: channel disconnected"`. The gate probe
`tests/probe_arc272_6b_state_over_lineage.rs` (currently `#[ignore]`, RED) isolates exactly this: the
child autobinds, hands its `Address'` up (works), then `(recv' self)` for its initial state; the parent
`(recv' svc)` the addr (works), then `(send' svc (Counter 1000))` — which fails. Diagnose the precise
cause, then fix it so a parent can send a value to a forms-server child over the lineage channel and the
child receives it via `(recv' self)`. The probe must go GREEN (returns 1005) with the test's `#[ignore]`
removed.

## What the disk already tells you (do not re-derive — confirm + extend)

- `comms::process::Sender` is a bare `{ write_fd: OwnedFd }` (`src/comms/process.rs:309`); `send()` is a
  pure `libc::write` that returns `SendError` only on **EPIPE / write failure** (`process.rs:330-372`).
  There is NO liveness/ring pre-check. So `"channel disconnected"` ⇒ **EPIPE ⇒ every read end of the
  child's input pipe is already closed when the parent writes.**
- The child was ALIVE through its `(send' self addr)` — the parent's `(recv' svc)` returned the addr.
  So the child closes/loses its input-pipe read ends (or exits) at/after that point, around `(recv' self)`.
- The forms-server child's owner-link self-peer is built FRESH from `dup(fd0)`/`dup(fd1)`
  (`src/process/verbs.rs:391-410`, `run_forms_as_server_child`): `self_peer_read_fd = dup(fd0)`,
  `self_peer_write_fd = dup(fd1)`, then `Peer::from_socket(self_peer_tx.reinterpret::<String>(),
  self_peer_rx)`. fd 0 itself is ALSO taken by `stdin_reader = PipeReader::from_owned_fd(fd0)`
  (`verbs.rs:418-419`).
- The parent holds the original `input_tx` Sender on the bundle (`src/kernel/spawn.rs:726`,
  `Process { input: input_tx, .. }`); `send' svc` routes through the PROCESS arm
  (`src/runtime.rs:23572-23612`, `bundle.peer.send(edn_str)` → `Process::send` →
  `input_tx.send`, `src/kernel/peer.rs:339`). `recv' svc` routes through `bundle.recv()`
  (`runtime.rs:23779+`) and WORKS.

## Phase 1 — diagnose (name the cause, do not guess)

Instrument and determine WHY the child's input-pipe read ends are gone when the parent sends. Strongly
suspected: the child EXITS at/around `(recv' self)` — either the forms-server self-peer's `recv'` returns
`Disconnected`/EOF immediately (so the child's `main` returns and the process exits, closing fd0 +
dup(fd0)), or an fd-ownership interaction closes the read ends. Confirm by capturing the child's exit:
- temporarily add an eprintln/trace in `run_forms_as_server_child` and/or run the child's `main` steps,
  OR inspect the child exit code / the err-channel, to see whether the child reaches `(recv' self)` and
  what that call returns.
- Determine: does `(recv' self)` block (child stays alive) or return/raise (child exits)? That single
  fact decides the fix.

## Phase 2 — fix the named cause

Make the parent→child lineage send functional and symmetric with the proven child→parent direction. The
honest fix keeps the constant `spawn-program'` 2-arg surface and the existing bundle shape; it wires the
parent's `send'` to a channel the forms-server child actually reads via `(recv' self)`, with the child
blocking (not exiting) until the value arrives. Do NOT add a second transport or change the public verb
surface. The fix lands in the forms-server child setup (`src/process/verbs.rs run_forms_as_server_child`)
and/or the process peer/bundle wiring (`src/kernel/spawn.rs spawn_process_peer`, `src/kernel/peer.rs`,
`src/comms/process.rs`) — wherever the diagnosis points.

## Rooms (read in order)

1. `tests/probe_arc272_6b_state_over_lineage.rs` — the gate. Understand the child `main` + parent flow.
2. `src/process/verbs.rs:376-443` — `run_forms_as_server_child`: the child self-peer + stdin_reader
   construction over fd 0/1. The fd-ownership crux lives here.
3. `src/kernel/spawn.rs:612-740` — `spawn_process_peer`: the pipe pairs, the child dup2 of fd 0/1/2, the
   parent bundle (`input_tx`).
4. `src/runtime.rs:23572-23612` (send' PROCESS arm) + `23779+` (recv' PROCESS arm) — how `send'`/`recv'`
   on `svc` route.
5. `src/kernel/peer.rs:300-370` — `Process` struct `send`/`recv`/close (the parent bundle's peer).
6. `src/comms/process.rs:300-410` — `Sender` (`write_fd`, `send`=write/EPIPE) + `sender_receiver_from_split_fds`.

The proven mirrors to copy the shape from: `tests/probe_arc272_6a_capability_handoff.rs` and
`tests/probe_arc209_c0b3aii_process_service_loop.rs` (child→parent self-peer send, both GREEN).

## Blast radius

`src/process/verbs.rs` + `src/kernel/spawn.rs` + `src/kernel/peer.rs` + `src/comms/process.rs` as the
diagnosis requires; remove the `#[ignore]` on the gate test. NO public verb-surface change (no new
`send'`/`recv'`/`spawn-program'` arity or args). NO new transport type. NO change to the thread tier.

## STOP triggers (halt and report; ship nothing)

1. STOP if the fix would require changing the `spawn-program'` / `send'` / `recv'` public surface (arity,
   new args, new verb) — that is a design change, not a wiring fix; surface it.
2. STOP if the cause is NOT in the parent→child lineage wiring (e.g. it turns out to be a type-decode bug
   unrelated to the pipe) — report the real cause; do not patch around it.
3. STOP if making the probe green requires the child to busy-wait/sleep/poll on a timer — the handoff must
   be lock-step over the wire (ZERO-MUTEX; no `mora`), as the child→parent direction already is.

## Gate (the orchestrator will re-run these itself)

- `cargo test --release -p wat --test probe_arc272_6b_state_over_lineage -- --include-ignored --test-threads=1`
  → GREEN (returns 1005), with `#[ignore]` removed from the test.
- `cargo test --release -p wat --lib -- --test-threads=1 | grep "test result"` → still **929 passed /
  36 failed** (the 36 are pre-existing; add zero new).
- `cargo build --release -p wat` → clean.

Report: the named root cause (Phase 1 finding), the exact files+lines changed, the gate results from your
OWN run, and any surprise.
