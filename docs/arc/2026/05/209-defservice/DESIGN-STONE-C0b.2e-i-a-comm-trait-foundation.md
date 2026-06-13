# DESIGN-STONE C0b.2e-i-a — the comms trait-object foundation

> First strike of the connection-`Peer` unification (decomposes the superseded
> `DESIGN-STONE-C0b.2e-i-unify-peer.md` into i-a foundation + i-b collapse).
> **PURELY ADDITIVE.** Gives the unified `select'` (in i-b) the two things it needs
> to drive a `Box<dyn CommReceiver<Value>>`: an explicit reactor-class discriminant
> to group peers, and `Any` to recover the concrete receiver. Nothing is retired
> here; the unified `Peer` struct + the ~70-site rewire are **C0b.2e-i-b** (the
> immediate next stone), which consumes this foundation. This stone is the carrier
> landing first so i-b operates on settled ground.

## Why — the seam this unblocks

The unified `Peer` will hold `Box<dyn CommSender<Value>>` / `Box<dyn CommReceiver<Value>>`
so `send'`/`recv'` call the trait (one arm, transport-blind, organic for remotes).
But `select'` cannot call a trait method: both reactors consume a **concrete**
`&Receiver<T>` —
- `comms::thread::Select::recv(&Receiver<T>)` (thread.rs:235), crossbeam-select;
- `comms::process::Select::recv(&Receiver<T>)` (process.rs:804) + `select()` drives
  the decode through that concrete receiver (`take_buffered_frame` / `read_into_acc`,
  process.rs:827+), reading the fd from it (process.rs:913).

The `Box<dyn …>` itself is already legal (the traits are object-safe — see Grounded).
What `select'` still cannot do through the box is (a) decide *which* reactor demuxes a
peer and (b) get the *concrete* receiver to feed it. This stone adds exactly those two:
a named **reactor-class** discriminant, and `Any` for concrete recovery.

## Grounded this session (HEAD `5791def6`)

- `CommSender` (mod.rs:806) / `CommReceiver` (mod.rs:827) are **already object-safe** —
  PROBE-VERIFIED: `Box<dyn CommSender<Value>>` / `Box<dyn CommReceiver<Value>>` compile
  at HEAD (the probe's only errors are the missing names below; no E0038). The
  `close(self)` by-value method is permitted in a dyn-compatible trait — it is simply
  not callable on the `dyn`, and nothing calls `close` on a boxed connection receiver
  (RAII `Drop` does the cleanup; the `close'` verb is Thread'/Process'-only). **`close`
  is UNCHANGED.** (An earlier predicted object-safety trap was falsified by the probe —
  the reason the probe is written before the brief.)
- The disconfirming fact (probe RED at HEAD): `ReactorClass`, `reactor_class`, `as_any`
  do not exist → E0432 (unresolved `ReactorClass`) + 4×E0599 (`reactor_class`/`as_any`
  not found on `Box<dyn CommReceiver<Value>>`).
- `impl<T: Send + 'static> CommReceiver<T> for thread::Receiver<T>` (thread.rs:178) —
  in-memory crossbeam endpoint; no fd.
- `impl<T: EdnRepresentable> CommReceiver<T> for process::Receiver<T>` (process.rs:484);
  `process::Receiver` owns a single `read_fd`, exposed by `pub(crate) fn poll_fd(&self)
  -> RawFd` (process.rs:399, already used by `Select`'s POLL_ADD).
- `Value: EdnRepresentable + Send` (i-0, `5791def6`) → `thread::pair::<Value>()` (plain
  tuple) and `process::pair::<Value>()` (Result) both exist; the process sender encodes
  internally via `value.to_wire()` (process.rs:172).

## The contract decision (pinned)

**(1) Reactor-class discriminant — a NAMED enum, never `Option<RawFd>`.**
```rust
pub enum ReactorClass { InMemory, Fd }       // which wait-primitive demuxes this receiver
// CommReceiver gains:
fn reactor_class(&self) -> ReactorClass;
```
`thread::Receiver` → `InMemory`; `process::Receiver` → `Fd`.

Rationale: `select'` (in i-b) must group peers by which of the two wait primitives
demuxes them — parked-thread channel-select (in-memory) vs kernel fd-poll (io_uring).
An `Option<RawFd>` would map "in-memory" onto `None` — a meaning smuggled into
presence/absence (Optional-is-a-smell). The named enum states both meanings.
This is a **closed enum on a genuinely fixed axis**: there are exactly two wait
primitives, and a third OS poller (epoll/kqueue) is still `Fd`. The **growing** axis —
N remote transports (UDS → TCP → mTLS) — lives in the `CommReceiver` impls, every one
of which is fd-backed and returns `Fd`. Closed-for-fixed, open-for-growing.
`Fd` carries **no payload**: the io_uring reactor reads the fd from the concrete
receiver it is handed (process.rs:913); a fd in the variant would be a second source
of truth.

**(2) Concrete recovery — `CommReceiver: Any` + `as_any`.**
```rust
pub trait CommReceiver<T>: std::any::Any {
    fn as_any(&self) -> &dyn std::any::Any;
    …
}
```
`select'` (in i-b) downcasts the boxed receiver back to the concrete tier receiver to
feed that tier's `Select` (crossbeam-select needs the crossbeam `Receiver`; process
`Select` needs the process `Receiver`). Confined to the two fixed impls. (`Any`
requires `'static`; both `Receiver<T>` impls are `'static` — the existing process bound
is `T: EdnRepresentable` which is `Send + 'static`; thread is `T: Send + 'static`.)

`CommSender` is UNCHANGED: `send'` calls `.send()` virtually — no class, no downcast.

## The gate (additive capability — structural disconfirm)

The probe `probe_arc209_c0b2eia_boxed_comm_both_tiers` (`tests/comms/trait_object.rs`,
already written + RED-verified at HEAD):
1. Box a `Value` sender + receiver over a **thread** pair and over a **process** pair.
2. Round-trip a `Value` through each (send via `Box<dyn CommSender<Value>>`, recv via
   `Box<dyn CommReceiver<Value>>`), assert equal.
3. Assert `reactor_class`: thread → `InMemory`, process → `Fd`.
4. Downcast each boxed receiver via `as_any` to its concrete type — assert `Some`.

**RED at HEAD (verified):** E0432 (`ReactorClass`) + 4×E0599 (`reactor_class`/`as_any`).
**GREEN after** the three additions.
**Regression:** existing comms round-trips green
(`probe_arc209_c0b2ei0_value_round_trip_over_process_pair` + the slice3 + socket
probes); nursery serial **895/4** (4 known baseline reds only); full workspace test
surface compiles (an additive trait change is a recompile cascade — every binary builds).

## Files touched

- `src/comms/mod.rs` — `pub enum ReactorClass`; `CommReceiver` supertrait `Any` +
  `reactor_class` + `as_any`. (No `CommSender` change. No `close` change.)
- `src/comms/thread.rs` — `CommReceiver` impl: `reactor_class → InMemory`, `as_any → self`.
- `src/comms/process.rs` — `CommReceiver` impl: `reactor_class → Fd`, `as_any → self`.
- `tests/comms/trait_object.rs` — the probe (already on disk).

No `peer.rs` change. No runtime/checker change. No new wat surface. No `select'` change.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** adding `Any` as a `CommReceiver` supertrait breaks object-safety or
   collides with an existing supertrait/blanket impl — STOP, report. (Grounded
   expectation: clean — both impls are `'static`; the trait stays dyn-compatible.)
2. **STOP-2:** an existing `CommReceiver` impl elsewhere (beyond thread/process) lacks
   `'static` and cannot satisfy the new `Any` bound — STOP, report. (Grounded
   expectation: only the two tier impls exist — confirm with a grep.)
3. **STOP-3:** `process::pair::<Value>()` / `thread::pair::<Value>()` do not exist or
   `Value: !EdnRepresentable` — STOP, report (would contradict i-0 `5791def6`).

## Out of scope (rejected — NOT deferred)

- The unified `Peer` struct, the runtime/checker rewire, retiring `SocketPeer'` /
  `SOCKET_PEER_TYPE_PATH`, and `send'`/`recv'` dropping arm-level encoding =
  **C0b.2e-i-b** (the immediate next stone, consuming this foundation).
- `select'` completing over both tiers (crossbeam → `thread::Select`, socket →
  `process::Select`, via `reactor_class` + `as_any`) = **C0b.2e-i-b**.
- The 3-arg process **service** multiplexer (self-peer + listener-arm + accept +
  `SelectEvent`) = **C0b.3a-ii** — a distinct defservice capability, not a leftover
  of this merge.

## The deadlock contract carries

Pure type-contract addition; no transport or lifecycle change. No `close`/`Drop`
change. [[feedback_vended_primitives_never_deadlock]] [[feedback_optional_is_a_smell]]
