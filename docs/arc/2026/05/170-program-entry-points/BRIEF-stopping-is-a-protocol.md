# BRIEF — stopping is a protocol (arc 170)

## The work

A service that **crashes** is currently more polite to its clients than one that **stops**.

The crash path is a protocol, built and correct: `serve-dispatch-op'-broadcast`
(`src/kernel/peer.rs`) loops every client, `notify_peer_crashed_best_effort()`s the
reason-free `PEER_CRASHED_SENTINEL` at each, the admin channel carries the real reason,
then it exits. Tell the clients, tell the admin why, exit.

The stop path is a signal. `substrate_on_stop_signal` (`src/process/child.rs`) sets the
flag — correct — and then writes the wake pipe, which drives the worker into
`trigger_shutdown()` (`src/runtime.rs`), which **drops a sender**. No ask, no notice, no
confirmation. Every peer discovers the absence, wherever it happens to be standing —
including services mid-transaction, holding what `ZERO-MUTEX.md` calls the lock and
waiting to send what it calls the release.

**This brief makes stopping a protocol, using pieces that already exist.**

## What already exists — verify each before building anything

- **The graceful-stop protocol, per service.** `{base}::Admin::Stop` (`wat/service.wat`),
  handled by returning `:wat::service::Outcome::Stop [state reply]` — "reply, THEN stop",
  i.e. the service finishes its CURRENT op and stops at its own safe point — confirmed by
  `Status::Stopped [final-state]` (arc 291 3a-ii-β).
- **A generated caller for it.** `(defn <fqdn>/stop [h <- Handle] -> state-ty …)`
  (`wat/service.wat` ~:1455): sends `Admin::Stop` down the lineage peer, recv's
  `Status::Stopped`, returns the final state. Owner-only and unforgeable — it requires the
  `Handle`, which clients never receive.
- **The Handles.** `start-primed-stdio` (`wat/kernel/services/stdio-primes.wat`) returns the
  three stdio Handles and *"Rust holds the returned tuple (keeping each admin lineage Peer'
  — hence each service — alive for the process lifetime)"*. The runtime holds the keys to
  the door it currently reaches past.
- **The client-facing notice.** `notify_peer_crashed_best_effort` — reason-free,
  non-blocking, on the existing data channel.
- **The doctrine.** `sigusr1`/`sigusr2`/`sighup` handlers are one atomic store each. The
  comment at `src/runtime.rs` states it: *"the kernel MEASURES; userland owns the
  transitions."* Only the stop handler departs from it.

## The contract, pinned — read this before touching anything

**1. The handler measures. It does not transition.** Same shape as its three siblings.

**2. There is NO TIMEOUT, and adding one would be wrong.** Ask what service fails to answer
`Admin::Stop`: one that is mid-op answers when the op completes — bounded by the work, which
is what lock-step means — and one that is wedged has a bug that a timeout would *hide*. A
deadline is also a guessed number nothing in the system knows, which `mora` forbids outright
(*"sleep is a guess; guesses race"*).

**The escalation already exists and is called SIGKILL.** It is not ours to build, and the
deadline belongs to the supervisor, which already has one — systemd `TimeoutStopSec`, Docker
`--stop-timeout`, Kubernetes `terminationGracePeriodSeconds`. All of them send SIGTERM, wait
their configured time, then SIGKILL. A second timeout inside the process would duplicate a
policy the operator has already configured and guess a number they have already chosen.

**3. If a service never answers, the process must HANG VISIBLY, naming the service it is
waiting on.** That is diagnostics, not a timer. A silent hang is a mystery; a hang that says
which service is wedged is a bug report.

**4. A client never learns a service's crash reason** (arc 294) — it gets the reason-free
notice; the reason rides the admin channel. That ruling is unchanged and this brief does not
touch it.

## ⚠ THE CRUX — the wake and the sever are currently the SAME EVENT

This is the load-bearing thing to understand before writing code, and the reason a naive
"delete the sever" is a regression rather than a fix.

The broadcast pipe signals by **HUP-on-drop**: the worker drops `broadcast_w_fd`, and every
reader polling it for `POLLHUP` wakes. That single act is simultaneously *"wake up"* and
*"you have been torn down"*. Readers poll it for `POLLHUP` only — `src/io.rs`,
`src/channel/transfer.rs`, and the io_uring arm in `src/comms/process.rs` (whose doc says
`broadcast fd: POLLHUP (worker dropped write-end on shutdown)`).

So removing the sever without separating the two leaves nothing to wake anyone — including
the stdin read that `3e297846` just brought into the multiplex, which polls that very fd.

**Separate them.** The wake becomes a **written byte** (`POLLIN`) rather than a dropped write
end (`POLLHUP`). The pattern already exists in the same file: the signal handler wakes the
worker by `libc::write(fd, &byte, 1)`. Readers then poll the broadcast for `POLLIN | POLLHUP`
— `POLLIN` means *a stop was requested, decide*, `POLLHUP` keeps its current meaning of
*torn down*. The teardown drop stays, and moves to LAST.

## The phases — each independently verifiable

**Phase 1 — the broadcast means WAKE, not SEVER.** Write a byte instead of (before) dropping;
readers add `POLLIN`. Nothing else changes: the drop still follows immediately, so behaviour
is unchanged and the floor must stay exactly as green as it is now. This phase is pure
mechanism and is the one that unblocks the rest.

**Phase 2 — the worker asks.** On wake, before any teardown: broadcast the reason-free notice
to clients, then call `<svc>/stop` on each held Handle and await `Status::Stopped`. Then tear
down.

**Phase 3 — the handler stops transitioning.** Once Phase 2 lands, `substrate_on_stop_signal`
is a single atomic store, matching `sigusr1`/`sigusr2`/`sighup`.

Do them in that order. Phase 3 before Phase 2 trades a flaky test for hangs.

## The acceptance tests — already on disk, both currently failing for ONE cause

`wat_cli::sigterm_to_cli_cascades_via_polling_contract` and
`wat_cli::sigterm_reaches_a_program_blocked_on_stdin` both pass isolated and both fail under
full-floor load at `println "READY"` — `stdio-write-out`, a service torn down between a write
and its ack. **Both go green when a service is no longer killed mid-transaction.** That is the
definition of done for this stone; do not modify either test.

## STOP triggers — ship nothing and report

- **STOP-1.** If, after Phase 1, any participant can still only be woken by the sever, STOP
  and NAME it. That participant is the whole reason the sever exists and the design must
  account for it before Phase 3.
- **STOP-2.** If calling `<svc>/stop` from the stop path would run wat code in a signal
  handler, STOP. It must happen in the worker thread, in normal context — the existing
  handler comment explains why (`crossbeam::Sender::send` is not async-signal-safe).
- **STOP-3.** If any phase seems to need a timeout, sleep, or retry-with-backoff to work,
  STOP and report what you were trying to bound. Per the contract above that is a design
  error, not an implementation detail.
- **STOP-4.** If the write path can only be made to pass by having `stdio-write-out` swallow
  a stop, STOP. That is papering over the sever rather than removing it, and it would put
  back the exact class of mask this arc spent a day removing.

## Blast radius

`src/process/child.rs` (the handler) · `src/runtime.rs` (the worker + `trigger_shutdown` +
the broadcast) · `src/io.rs`, `src/channel/transfer.rs`, `src/comms/process.rs` (the three
poll sites gaining `POLLIN`) · `src/freeze.rs` and/or `src/services/` (reaching the held
Handles). No new wat surface. No changes to `wat/service.wat`'s stop protocol — it is
correct and this brief consumes it.

## Gate

`cargo build --release --all-targets` clean, zero warnings. The orchestrator weighs the floor
centrally, including the two acceptance tests above under full load.
