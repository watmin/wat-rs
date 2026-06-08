# BRIEF — Stone 6.1: the wall falls (typed_channel dies; the seam lifts home)

> Slice 6 per the campaign map: "retire typed_channel.rs — typed_channel dies
> here." Post-5.1 the file is a SHIM whose survivors are perfected, live
> forms: the transport-polymorphic SenderInner/ReceiverInner seam (the
> engineered two-tier polymorphism that made the 5.1 floor-move possible),
> the Send/RecvOutcome surface, and the typed ops. Per the 8.1w/8.2w
> precedent: survivors lift to a home; the condemned file dies; behavior
> identical. The dead fork/spawn paths the compiler already names are
> Stone 6.2 — NOT this stone.

## Required reading (in order)
1. `src/typed_channel.rs` — whole (694 lines). Every item moves or dies.
2. `src/services/mod.rs` + `src/services/peer.rs` — the home-shape precedent
   (index + flat re-exports; doc travels with code).
3. `docs/ZERO-MUTEX.md` § Honest caveats (the AtomicBool closed-flag's
   documented standing).

## The gate (committed, RED at HEAD)
`tests/nursery/probe_arc214_stone61_typed_channel_dead.rs` — file absence +
zero `typed_channel::` paths (self-excluding from birth). GREEN by the lift.

## The home's shape (pinned)

`src/channel/` — the wat-surface channel layer (the seam between
`Value::wat__kernel__Sender/Receiver` and the comms tiers):

- **`src/channel/mod.rs`** — module doc (the two-tier transport story:
  tier-1 comms in-memory carrying Values; tier-2 PipeFd carrying
  line-delimited EDN; the Option-B inner-enum decision and the wire/error
  semantics — today's typed_channel module doc CARRIES, updated to name the
  new home and the Stone-6.1 lift) + `pub mod inner; pub mod transfer;` +
  flat pub-use re-exports of every public name.
- **`src/channel/inner.rs`** — the seam: `SenderInner`, `ReceiverInner`
  (verbatim, docs included), the four constructors (`sender_from_comms`,
  `receiver_from_comms`, `sender_from_pipe`, `receiver_from_pipe`).
- **`src/channel/transfer.rs`** — the movement surface: `SendOutcome`,
  `RecvOutcome`, `typed_send`, `sender_close`, `typed_recv`,
  `typed_try_recv`, `try_as_comms_receiver`, `make_pipe_channel_pair`,
  `make_thread_peer_pair_for_test` (alive — a test binary consumes it).

## The kill

- **`bounded<T>` DIES.** Its two live tenants convert to
  `crate::comms::thread::pair::<SpawnOutcome>()`:
  - `src/spawn.rs:176`
  - `src/runtime.rs:19151`
  The receiving/sending code adapts to comms' Sender/Receiver API at those
  two sites (read the surrounding code; the comms pair is depth-1, same as
  bounded(1) — semantics preserved). The test use at
  `tests/wat_arc170_typed_channel_pipes.rs:581` converts the same way.
- `git rm src/typed_channel.rs` once the lift compiles.
- `src/lib.rs`: `pub mod typed_channel` dies; re-exports repoint to
  `channel`.
- Sweep every `crate::typed_channel::`/`wat::typed_channel::` →
  `crate::channel::`/`wat::channel::` (src: runtime.rs ~52 refs, fork.rs,
  spawn.rs, check.rs, value/value.rs, comms/process.rs, types.rs, lib.rs;
  tests: ~10 files — `cargo check --all-targets` + the gate-probe's scan
  are the completeness checks). Comments referencing the old path update to
  the new home; retirement-narration comments in the dying file die with it.
- `tests/wat_arc170_typed_channel_pipes.rs` — the FILE NAME carries the dead
  module's name; rename to `tests/wat_arc170_channel_pipes.rs` (names must
  not lie; check for a build.rs/test-mod registry that needs the rename
  reflected — 254.T auto-discovers, verify).

## Behavior-identical
NO logic edits beyond the two bounded→pair conversions. NO renames of items
(SenderInner stays SenderInner — paths change, names do not). The wat
surface (make-channel, send, recv, select, close, the peer verbs) must be
byte-identical in behavior.

## Gates
1. Gate-probe 61 → 2/2 GREEN.
2. `cargo test --release --lib -p wat` → 943/0/1.
3. `cargo test --release --test nursery` → 863/4/4 (the 4 known parked-255
   reds; your gate +2).
4. `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` → 12/0/0.
5. The renamed pipes binary green: `setsid timeout 120 cargo test --release
   --test wat_arc170_channel_pipes -- --test-threads=1` (process tests —
   enveloped).
6. `cargo check --all-targets` → 0 errors.
7. `cargo clippy --release --lib -p wat` → zero findings in src/channel/.

## STOP triggers (rejection criteria)
- STOP-1: a bounded→pair conversion turns out behavior-CHANGING (the
  surrounding code depends on crossbeam-specific semantics comms lacks).
  Report the exact dependency; ship nothing.
- STOP-2: an untraceable red outside the known baseline.

## Constraints
- Commit NOTHING — the orchestrator scores and commits.
- The probe files are read-only ground truth.
- Work only in /home/watmin/work/holon/wat-rs/.
