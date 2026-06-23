# BRIEF — arc 292 L1: introduce `ProcessSelectable` enum (pure refactor)

**You are a LEAF executor.** Do ONE bounded mechanical refactor, run the gate, report.
**Do NOT spawn subagents.** Do NOT design beyond this brief. If the work turns out
larger than described below (e.g. the cell type is consumed in more than the 4 named
arms + the construction site), **STOP and report what you found** — do not improvise a
wider change.

## The work (one paragraph)

Today a process-tier peer is stored as `Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>>`.
We are introducing a named enum so that, in a later strike, a *timer* peer can be a
distinct, honestly-named kind of process-tier select'-able (`Timer`) alongside the
spawned-child kind (`Spawned`) — instead of faking a childless child. **This strike (L1)
introduces the enum with ONLY the `Spawned` variant and flips the cell + every consumer
to wrap/match `Spawned`.** It is a PURE REFACTOR: zero behavior change, all existing
process peers are `Spawned`. (The `Timer` variant is added by a later strike; introducing
it then will make rustc's non-exhaustive-match errors point at exactly the sites that must
handle a timer — that is intended, not your concern now.)

## Rooms — read in order (each with why)

1. `src/kernel/spawn.rs:107-126` — the `ProcessPeerCell` type alias (`:116`) and the
   `PROCESS_PEER_TYPE_PATH` const (`:126`). The alias is `Arc<ThreadOwnedCell<Option<
   ProcessPeerBundle>>>`. You will change the inner payload `ProcessPeerBundle` →
   `ProcessSelectable`.
2. `src/kernel/spawn.rs:252-271` — the `ProcessPeerBundle` struct. UNCHANGED — it becomes
   the payload of the `Spawned` variant. Read it so you understand what `Spawned` wraps.
3. `src/kernel/spawn.rs:847-874` — the construction site. `let bundle = ProcessPeerBundle
   {...}; ... ThreadOwnedCell::new(Some(bundle))`. You will wrap: `Some(
   ProcessSelectable::Spawned(bundle))`.
4. `src/runtime.rs:24010-24051` — **send'** arm. Inside `cell.with_ref(...)` the match is
   `None => err, Some(bundle) => bundle.peer.send(...)`. Change `Some(bundle)` →
   `Some(ProcessSelectable::Spawned(bundle))`; inner logic unchanged.
5. `src/runtime.rs:24217-24264` — **recv'** arm. Same shape: `Some(bundle) => bundle.recv()`
   → `Some(ProcessSelectable::Spawned(bundle))`.
6. `src/runtime.rs:24430-24450` — **close'** arm. `cell.with_mut(..., |opt_bundle|
   opt_bundle.take())` then `.ok_or_else(...)` yields the bundle; `bundle.peer.wait()`.
   The `.take()` now yields `Option<ProcessSelectable>`; after the `ok_or_else`, match /
   destructure the `ProcessSelectable::Spawned(bundle)` to recover `bundle` before
   `bundle.peer.wait()`. (If it's a `Timer` — impossible in L1 since none are constructed —
   that arm does not exist yet; with a single-variant enum the destructure is irrefutable.)
7. `src/runtime.rs:24696-24767` — **select'** process arm. The local `type ProcessCell`
   (`:24698`) mirrors the alias — update its inner payload to `ProcessSelectable`. The
   guard loop (`:24750-24766`) matches `None => err, Some(bundle) => { output_rxs.push(
   &bundle.peer.output); err_rxs.push(&bundle.err); }` — change `Some(bundle)` →
   `Some(ProcessSelectable::Spawned(bundle))`.

## Implementation sketch (the strike path — fill it, don't invent the shape)

In `src/kernel/spawn.rs`, near `ProcessPeerBundle` (after its `impl`, ~line 328):

```rust
/// A process-tier select'-able. Today the only kind is a spawned child
/// (`Spawned`); arc 292 L3 adds `Timer` (a timerfd-backed one-shot, no child) as a
/// second NAMED variant — identity is named, never inferred from a None. See
/// docs/arc/2026/06/292-timer-peer-time-as-select/DESIGN.md (D5).
pub enum ProcessSelectable {
    /// A spawned child process and its channels.
    Spawned(ProcessPeerBundle),
}
```

Change the alias (`:116`):
```rust
pub type ProcessPeerCell = Arc<ThreadOwnedCell<Option<ProcessSelectable>>>;
```

Then the 5 sites above: wrap construction in `ProcessSelectable::Spawned(..)` and change
each `Some(bundle)` match to `Some(ProcessSelectable::Spawned(bundle))`. The local
`type ProcessCell` in runtime.rs select' must change its inner payload identically.

`ProcessSelectable` must be exported where `ProcessPeerBundle` / `PROCESS_PEER_TYPE_PATH`
are (same `pub` visibility / module path) so runtime.rs can name it.

## Blast radius (bounded)
- `src/kernel/spawn.rs` — enum def + alias + 1 construction site.
- `src/runtime.rs` — 4 match arms (send'/recv'/close'/select') + the local `ProcessCell`
  alias in select'.
- **No other files.** No new behavior. No `comms/` change. No `kernel/peer.rs` change
  (the `Process` struct is UNTOUCHED — `pidfd` stays mandatory inside `Spawned`).

## STOP triggers (halt + report; do NOT improvise)
1. If `PROCESS_PEER_TYPE_PATH` opaque is downcast to `Option<ProcessPeerBundle>` in MORE
   than the 4 runtime arms named above (grep `ProcessPeerBundle` across `src/` — there are
   ~27 mentions, but most are doc/comment; the live downcast sites are the 4 named), STOP
   and report the extra site(s) — the blast radius assumption is wrong.
2. If wrapping the construction or matching `Spawned` requires touching `kernel/peer.rs`'s
   `Process` struct or any `comms/` file, STOP — that means the boundary is wrong.
3. If a single-variant enum trips a lint that cannot be satisfied without suppressing a
   real signal, STOP and report (do not add a blanket `#[allow]`).

## Gate (run it yourself; report real output)
```
cargo build 2>&1 | tail -20
cargo test --no-fail-fast 2>&1 | tail -40
```
Expected: clean build; the full workspace test suite as green as it is at HEAD (this is a
pure refactor — no test should change status). Report the exact `test result:` lines.

## Shape to copy
The thread tier's equivalent named-variant lives at `src/comms/thread.rs:127`
(`enum ReceiverKind<T> { Channel(..), Timer{..} }`) — same idea (name the kind), different
layer. Your enum is the process-tier peer-level analog.
