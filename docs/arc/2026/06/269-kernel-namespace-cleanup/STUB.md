# Arc 269 (STUB) — `:wat::kernel::` namespace cleanup (the junk-drawer split)

> **Status: STUB — banked 2026-06-15.** Surfaced by an intueri assessment of the
> kernel/spawn/program taxonomy (cast while placing the `Host` protocol). NOT blocking — the
> host-parity leg's own types are corrected separately + now (the spawn-coherence move: `Bound`/
> `Spawned`/`ServiceEvent` `:wat::kernel::` → `:wat::spawn::`). This arc is the BIG residual re-org.

## The finding (intueri, fidelity 4/10)

`:wat::kernel::` was intended for **syscall-y / low-level OS primitives** but became a **named junk
drawer** (~60 members across unrelated domains). A reader arriving with no context cannot predict
what's in it. It MUMBLES (intueri Level 2) — the name promises a domain it doesn't hold.

- **`:wat::program::`** — COHERENT (running-peer identity: `Env`/`EmptyEnv`/`PeerKind`). Leave it.
- **`:wat::spawn::`** — right intent (host opts/how-to-launch); its contamination (`Bound`/`Spawned`/
  `ServiceEvent` namespaced `kernel` though defined in spawn.wat) is fixed by the spawn-coherence
  move (separate, now). Leave the rest.
- **`:wat::kernel::`** — MUDDLED. This arc.

## What genuinely FITS `kernel` (the syscall-y residue — keep)

The primed comms verbs + their types: `Peer'`/`Listener'`/`Address'`, `send'`/`recv'`/`close'`/
`select'`/`poll'`, `connect'`/`accept'`/`listener'`/`peer-pair'`/`socket-pair'`/`socket-address'`,
`allow'`/`deny'`, `Sender<T>`/`Receiver<T>`/`Channel<T>`/`CommResult<T>`/`Chosen<T>`, `pipe`,
`make-channel`, the signal queries (`sigusr1?`/`sighup?`/`reset-*!`). These are OS-adjacent — they
stay.

## What LIES about `kernel` (move out — the re-org)

- **Diagnostics/failures → `:wat::kernel::diag::` (or `:wat::error::`)**: `Failure`, `Frame`,
  `Location`, `raise!`, `assertion-failed!`, `extract-panics`, `ThreadDiedError`, `ProcessDiedError`,
  `RunResult`, `StartupError`, `failure-from-*`.
- **Sandbox/test conveniences → `:wat::kernel::sandbox::` (or `:wat::test::*`)**: `run-sandboxed*`,
  `drive-sandbox`, `startup-failure-result`, `drain-lines*`.
- **stdio**: `println`/`eprintln`/`readln` — decide (a thin `:wat::io::`-adjacent home vs keep as the
  OS-write primitive). `services::Std{In,Out,Err}Service` already sub-namespaced (`services::`) — honest.
- **Thread/bracket macros → their own home**: `run-threads`/`run-threads-n1`/`run-threads-n3`.
- **`HandlePool`** → spawn-layer orchestration utility (decide home).

## Why banked, not now

It's a **broad callsite sweep** (~60 members, many call sites across wat/ + src/ + tests/). Not the
thing to do mid-leg. Pick it up as a dedicated arc: ground each cluster, decide the sub-namespace
names (intueri each), migrate via `fix-wat`/codemod, gate zero-new. The spawn-coherence move (the
leg's own `Bound`/`Spawned`/`ServiceEvent`) is the cheap, in-scope slice done now; this is the rest.

## To investigate when picked up
- Confirm the cluster boundaries + the sub-namespace names (intueri).
- Whether `kernel::diag::`/`kernel::sandbox::` sub-namespaces vs top-level `:wat::error::`/`:wat::test::`.
- Drive the migration with the self-hosted `fix-wat` codemod (the rename ledger), not by hand.
- Re-cast intueri on the result; target fidelity ≥ 8/10.
