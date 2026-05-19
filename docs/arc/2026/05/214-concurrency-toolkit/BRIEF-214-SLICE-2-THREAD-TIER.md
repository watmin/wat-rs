# Arc 214 Slice 2 — Thread tier

## Mission

Implement the THREAD TIER in a NEW file `src/comms/thread.rs`. crossbeam_channel underneath. Cascade-aware recv via `select! { recv(data), recv(SHUTDOWN_RX) }`. Implements `CommSender<T>` / `CommReceiver<T>` traits from Slice 1.

This is the first concrete tier implementation. Slice 1 minted the type-level shape (traits, errors, SelectOutcome); Slice 2 builds the in-process tier with crossbeam underneath; Slice 3 will build the cross-process tier with io_uring underneath. After Slice 2 lands clean, Slice 3 mirrors its structure with io_uring.

## Substrate context (substrate-truth verified pre-spawn)

- **`SHUTDOWN_RX`** at `src/runtime.rs:179` — `pub static SHUTDOWN_RX: OnceLock<crossbeam_channel::Receiver<()>> = OnceLock::new();`. Initialized by `init_shutdown_signal()` (freeze.rs:233) before any wat code executes. Bootstrap fallback: when SHUTDOWN_RX is `None` (pre-init or test bypass), recv falls back to bare crossbeam recv.

- **Pattern reference (DO NOT TOUCH)**: `src/typed_channel.rs:541+` contains χ-1's existing thread-tier wrapper from arc 213 — `wat::typed_channel::{Sender<T>, Receiver<T>}` with cascade-aware recv. Slice 2 builds the SAME shape in a NEW file using the NEW Slice 1 traits. The χ-1 wrapper stays untouched until Slice 5's migration sweep retires it.

- **Pattern reference (READ-ONLY)**: `src/typed_channel.rs:295-345` — `typed_recv`'s Crossbeam arm shows the canonical cascade-aware select pattern. Slice 2 mirrors this pattern for the new tier.

- **Slice 1 traits + types available at `crate::comms::*`**:
  - `HolonRepresentable` (not used in thread tier; thread channels pass T directly)
  - `CommSender<T>` — Slice 2 implements
  - `CommReceiver<T>` — Slice 2 implements
  - `SendError<T>`, `RecvError`, `TryRecvError`, `CloseError`, `WireError`
  - `ReceiverIndex(pub usize)`, `SelectOutcome<T>`

## Concrete deliverables

### 1. Create `src/comms/thread.rs` with the following items

**Module-level doc:**

```rust
//! # Thread tier — in-process comms via crossbeam_channel
//!
//! Layer 0a tier implementation per arc 214's `DESIGN.md`. Builds on the
//! Slice 1 traits (`crate::comms::{CommSender, CommReceiver, SelectOutcome,
//! ReceiverIndex, SendError, RecvError, TryRecvError, CloseError}`) with
//! `crossbeam_channel` underneath.
//!
//! ## Cascade contract (LOAD-BEARING)
//!
//! Every blocking method auto-wires the substrate's shutdown signal:
//! - `Receiver::recv()` uses `crossbeam_channel::select! { recv(data),
//!   recv(SHUTDOWN_RX) }` — on shutdown, recv returns `Err(RecvError)`
//!   instead of hanging indefinitely.
//! - `Select::select()` registers `SHUTDOWN_RX` as an internal arm —
//!   on shutdown, returns `SelectOutcome::Shutdown` regardless of which
//!   user receivers are pending.
//!
//! Bootstrap fallback: when `SHUTDOWN_RX` is uninitialized (pre-init or
//! test bypass), blocking methods fall back to bare crossbeam recv.
//! Production paths always have `SHUTDOWN_RX` initialized by
//! `freeze.rs:233` before any wat code executes.
//!
//! ## Audience
//!
//! Substrate-internal Rust code (brackets, services, kernel layer
//! dispatch). User code does NOT touch this tier — it uses peer-oriented
//! `:wat::kernel::*` verbs (Slice 4) that internally dispatch here.
```

**`Sender<T>` newtype:**

```rust
/// Thread-tier send endpoint. Wraps `crossbeam_channel::Sender<T>` with
/// the tier-agnostic `CommSender` trait surface. Private inner field
/// prevents bare crossbeam access from outside the tier.
#[derive(Debug)]
pub struct Sender<T> {
    inner: crossbeam_channel::Sender<T>,
}

impl<T> Sender<T> {
    /// Send a value to the channel. Returns `Err(SendError(value))` if
    /// all receivers have been dropped (cf. `crossbeam::SendError`).
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.inner.send(value).map_err(|crossbeam_channel::SendError(v)| SendError(v))
    }

    /// Signal end-of-stream from this sender. Consumes self so the endpoint
    /// is gone after close. Other cloned `Sender` handles (if any) remain
    /// valid. Peer receivers will see `RecvError` / `TryRecvError::Disconnected`
    /// on their next operation only after ALL `Sender` clones close.
    ///
    /// Thread-tier close always succeeds (crossbeam channels don't fail at
    /// close — the Drop impl handles cleanup). Returns `Ok(())` after
    /// consuming self.
    pub fn close(self) -> Result<(), CloseError> {
        // self is dropped at end of scope; crossbeam decrements its
        // internal sender count; when count hits zero, receivers see
        // Disconnected. No fallible operation; always Ok.
        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T: Send + 'static> CommSender<T> for Sender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>> {
        Sender::send(self, value)
    }
    fn close(self) -> Result<(), CloseError> {
        Sender::close(self)
    }
}
```

**`Receiver<T>` newtype with cascade-aware recv:**

```rust
/// Thread-tier receive endpoint. Wraps `crossbeam_channel::Receiver<T>`
/// with cascade-aware blocking recv. Private inner field prevents bare
/// crossbeam access from outside the tier.
#[derive(Debug)]
pub struct Receiver<T> {
    inner: crossbeam_channel::Receiver<T>,
}

impl<T> Receiver<T> {
    /// Cascade-aware blocking recv. Routes through `SHUTDOWN_RX` via
    /// `crossbeam::select! { recv(data), recv(SHUTDOWN_RX) }`. When
    /// substrate shutdown fires, parked recvs wake with `Err(RecvError)`
    /// instead of hanging indefinitely.
    ///
    /// Bootstrap fallback: when `SHUTDOWN_RX` is `None`, falls back to
    /// bare crossbeam recv. Production paths always have SHUTDOWN_RX
    /// initialized before wat code executes.
    pub fn recv(&self) -> Result<T, RecvError> {
        let shutdown_rx = crate::runtime::SHUTDOWN_RX.get();
        match shutdown_rx {
            Some(srx) => {
                crossbeam_channel::select! {
                    recv(&self.inner) -> msg => msg.map_err(|_| RecvError),
                    recv(srx) -> _ => Err(RecvError),
                }
            }
            None => self.inner.recv().map_err(|_| RecvError),
        }
    }

    /// Non-blocking recv. Returns `Err(TryRecvError::Empty)` when no value
    /// is currently available; `Err(TryRecvError::Disconnected)` when all
    /// senders have dropped. Cascade-irrelevant (does not block).
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.inner.try_recv().map_err(|e| match e {
            crossbeam_channel::TryRecvError::Empty => TryRecvError::Empty,
            crossbeam_channel::TryRecvError::Disconnected => TryRecvError::Disconnected,
        })
    }

    /// Number of values currently queued in the channel awaiting recv.
    /// Non-blocking; cascade-irrelevant. Trivial passthrough to
    /// `crossbeam::Receiver::len`.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Signal end-of-stream from this receiver. See `CommReceiver::close`
    /// for multi-handle semantics. Thread-tier close always succeeds.
    pub fn close(self) -> Result<(), CloseError> {
        // Drop happens at end of scope; crossbeam decrements receiver count.
        Ok(())
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T: Send + 'static> CommReceiver<T> for Receiver<T> {
    fn recv(&self) -> Result<T, RecvError> { Receiver::recv(self) }
    fn try_recv(&self) -> Result<T, TryRecvError> { Receiver::try_recv(self) }
    fn len(&self) -> usize { Receiver::len(self) }
    fn close(self) -> Result<(), CloseError> { Receiver::close(self) }
}
```

**`Select<'a, T>` cascade-aware fan-in:**

```rust
/// Cascade-aware fan-in over multiple thread-tier receivers. Auto-registers
/// `SHUTDOWN_RX` as an internal arm; user-registered receivers get
/// `ReceiverIndex`es in registration order.
///
/// When `select()` fires, the shutdown arm wins iff substrate shutdown
/// signaled (returns `SelectOutcome::Shutdown`); otherwise the fired
/// user receiver's index + recv result is returned.
pub struct Select<'a, T: Send + 'static> {
    inner: crossbeam_channel::Select<'a>,
    /// Crossbeam-internal index for SHUTDOWN_RX (if registered).
    /// `None` only in bootstrap (SHUTDOWN_RX uninitialized).
    shutdown_arm: Option<usize>,
    /// User-registered receivers in registration order: (crossbeam-arm-idx,
    /// receiver ref). The user-facing index is the position in this Vec.
    user_arms: Vec<(usize, &'a Receiver<T>)>,
}

impl<'a, T: Send + 'static> Select<'a, T> {
    /// Construct a new cascade-aware Select. Auto-registers `SHUTDOWN_RX`
    /// internally so callers don't manually wire shutdown into every Select.
    pub fn new() -> Self {
        let mut inner = crossbeam_channel::Select::new();
        let shutdown_arm = crate::runtime::SHUTDOWN_RX
            .get()
            .map(|srx| inner.recv(srx));
        Self {
            inner,
            shutdown_arm,
            user_arms: Vec::new(),
        }
    }

    /// Register a receiver. Returns the `ReceiverIndex` the caller will
    /// see in `SelectOutcome::Recv { index, .. }` when this receiver fires.
    /// Index is the registration order (0 for first registered, 1 for
    /// second, etc.) — independent of the crossbeam-internal arm index.
    pub fn recv(&mut self, rx: &'a Receiver<T>) -> ReceiverIndex {
        let crossbeam_idx = self.inner.recv(&rx.inner);
        let user_idx = self.user_arms.len();
        self.user_arms.push((crossbeam_idx, rx));
        ReceiverIndex(user_idx)
    }

    /// Block until any registered receiver fires OR substrate shutdown
    /// signals. Returns the outcome — `Recv { index, result }` for a user
    /// receiver firing, `Shutdown` when the cascade fires.
    pub fn select(&mut self) -> SelectOutcome<T> {
        let oper = self.inner.select();
        let fired_crossbeam_idx = oper.index();

        // Shutdown arm fired?
        if self.shutdown_arm == Some(fired_crossbeam_idx) {
            // Consume the shutdown signal to complete the operation
            // (crossbeam requires consuming the SelectedOperation).
            let srx = crate::runtime::SHUTDOWN_RX
                .get()
                .expect("shutdown_arm was Some so SHUTDOWN_RX must be initialized");
            let _ = oper.recv(srx);
            return SelectOutcome::Shutdown;
        }

        // Find the user-arm that fired.
        let (user_pos, &(_, rx)) = self
            .user_arms
            .iter()
            .enumerate()
            .find(|(_, (cb_idx, _))| *cb_idx == fired_crossbeam_idx)
            .expect("registered receiver fired but not found in user_arms");

        let result = oper.recv(&rx.inner).map_err(|_| RecvError);
        SelectOutcome::Recv {
            index: ReceiverIndex(user_pos),
            result,
        }
    }
}

impl<'a, T: Send + 'static> Default for Select<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}
```

**Factories:**

```rust
/// Create an unbounded thread-tier channel pair. Both endpoints are
/// cascade-aware (Receiver's recv wakes on shutdown).
pub fn pair<T: Send + 'static>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (Sender { inner: tx }, Receiver { inner: rx })
}

/// Create a bounded thread-tier channel pair with the given capacity.
/// Senders block on `send` when the channel is full (no cascade for
/// blocking sends — that's a future arc if needed).
pub fn bounded<T: Send + 'static>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = crossbeam_channel::bounded(capacity);
    (Sender { inner: tx }, Receiver { inner: rx })
}
```

**Imports at the top of the file:**

```rust
use crate::comms::{
    CloseError, CommReceiver, CommSender, ReceiverIndex, RecvError,
    SelectOutcome, SendError, TryRecvError,
};
```

**Wire up `pub mod thread;` in `src/comms/mod.rs`** at the end of the existing file (after the SelectOutcome enum; e.g., as the last item in the module).

### 2. Create smoke probe `tests/probe_comms_thread.rs`

Tests (all with REAL assertions; no `_` bindings without follow-up checks — per Slice 1's gaze L1 lesson):

```rust
//! Arc 214 Slice 2 smoke probe — verify thread tier round-trip + cascade.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use wat::comms::{ReceiverIndex, RecvError, SelectOutcome, TryRecvError};
use wat::comms::thread::{bounded, pair, Select};

#[test]
fn probe_slice2_unbounded_round_trip() {
    let (tx, rx) = pair::<i64>();
    tx.send(42).expect("send");
    assert_eq!(rx.recv().expect("recv"), 42);
}

#[test]
fn probe_slice2_bounded_round_trip() {
    let (tx, rx) = bounded::<i64>(4);
    tx.send(1).expect("send 1");
    tx.send(2).expect("send 2");
    assert_eq!(rx.len(), 2);
    assert_eq!(rx.recv().expect("recv 1"), 1);
    assert_eq!(rx.recv().expect("recv 2"), 2);
    assert_eq!(rx.len(), 0);
}

#[test]
fn probe_slice2_sender_drop_triggers_recv_err() {
    let (tx, rx) = pair::<i64>();
    drop(tx);
    assert_eq!(rx.recv(), Err(RecvError));
}

#[test]
fn probe_slice2_try_recv_empty_returns_empty() {
    let (_tx, rx) = pair::<i64>();
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

#[test]
fn probe_slice2_try_recv_disconnected_after_sender_drop() {
    let (tx, rx) = pair::<i64>();
    drop(tx);
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
}

#[test]
fn probe_slice2_clone_sender_multi_producer() {
    let (tx, rx) = pair::<i64>();
    let tx2 = tx.clone();
    thread::spawn(move || { tx.send(1).ok(); });
    thread::spawn(move || { tx2.send(2).ok(); });
    let a = rx.recv().expect("recv 1");
    let b = rx.recv().expect("recv 2");
    let mut got = [a, b];
    got.sort();
    assert_eq!(got, [1, 2]);
}

#[test]
fn probe_slice2_clone_receiver_multi_consumer() {
    let (tx, rx) = pair::<i64>();
    let rx2 = rx.clone();
    tx.send(99).expect("send");
    // Either receiver can consume; verify ONE of them got it.
    let from_a = rx.try_recv();
    let from_b = rx2.try_recv();
    assert!(matches!((from_a, from_b),
        (Ok(99), Err(TryRecvError::Empty)) |
        (Err(TryRecvError::Empty), Ok(99))));
}

#[test]
fn probe_slice2_select_picks_fired_receiver() {
    let (tx_a, rx_a) = pair::<i64>();
    let (_tx_b, rx_b) = pair::<i64>();
    tx_a.send(7).expect("send a");
    let mut sel: Select<i64> = Select::new();
    let idx_a = sel.recv(&rx_a);
    let _idx_b = sel.recv(&rx_b);
    match sel.select() {
        SelectOutcome::Recv { index, result } => {
            assert_eq!(index, idx_a);
            assert_eq!(result, Ok(7));
        }
        SelectOutcome::Shutdown => panic!("unexpected Shutdown"),
    }
}

#[test]
fn probe_slice2_select_indices_match_registration_order() {
    let (_tx_a, rx_a) = pair::<i64>();
    let (_tx_b, rx_b) = pair::<i64>();
    let (_tx_c, rx_c) = pair::<i64>();
    let mut sel: Select<i64> = Select::new();
    let idx_a = sel.recv(&rx_a);
    let idx_b = sel.recv(&rx_b);
    let idx_c = sel.recv(&rx_c);
    assert_eq!(idx_a, ReceiverIndex(0));
    assert_eq!(idx_b, ReceiverIndex(1));
    assert_eq!(idx_c, ReceiverIndex(2));
}

#[test]
fn probe_slice2_close_idempotent_with_clones() {
    use wat::comms::CommSender;
    let (tx, rx) = pair::<i64>();
    let tx2 = tx.clone();
    // Close one clone; the other should still work.
    CommSender::close(tx).expect("close 1");
    tx2.send(5).expect("send via remaining clone");
    assert_eq!(rx.recv().expect("recv"), 5);
}
```

10 tests covering: round-trip (unbounded + bounded), sender-drop, try_recv (empty + disconnected), Clone semantics (sender + receiver), Select firing + index ordering, close multi-clone behavior.

## Verification

```
cargo build --release                                       # must be clean
cargo test --release --test probe_comms_thread              # 10/10 PASS
cargo test --release --test probe_comms_foundation          # 3/3 PASS (Slice 1 unchanged)
cargo test --release --test probe_channel_primitive         # 3/3 PASS (χ-1 unchanged)
cargo test --release --test probe_pidfd_primitive           # 2/2 PASS (α unchanged)
```

Per `feedback_no_hang_vector_in_additive_scorecard`: **DO NOT** run `wat_arc170_program_contracts` or any workspace tests. The cascade-completeness proof for the broader substrate is χ-4 / Slice 5 territory.

## Out of scope (STOP triggers)

- **DO NOT touch `src/typed_channel.rs`** (arc 213 χ-1 wrapper; Slice 5 migrates callers later)
- **DO NOT implement process tier** (Slice 3 territory; src/comms/process.rs)
- **DO NOT add wat-level dispatch** (Slice 4 territory; src/kernel/)
- **DO NOT migrate existing callers** (Slice 5 territory)
- **DO NOT touch the dirty tree** (src/fork.rs + src/spawn_process.rs — arc 213 δ-1)
- **DO NOT run wat_arc170 or workspace tests** (per the additive-scorecard discipline)
- **ZERO modifications** outside `src/comms/thread.rs` (new) + `src/comms/mod.rs` (1 line `pub mod thread;`) + `tests/probe_comms_thread.rs` (new) + SCORE doc (new)

## Pre-emptive ward discipline (lessons from Slice 1)

The Slice 1 ward pass surfaced 9 findings (1 gaze L1, 4 gaze L2, 3 forge L1, 1 reap). Avoid those patterns in Slice 2 by construction:

1. **All public items get doc comments** (gaze L2 lesson on `len`). Every `pub fn` and `pub struct` must have a doc comment explaining what it does + WHY callers should care.
2. **Tests have REAL assertions** (gaze L1 lesson). NO bare `_`-bindings without follow-up `assert_eq!` / `assert_ne!` / `assert!(matches!())`. Every test name must honestly describe the assertion strength.
3. **Use struct variants** when field identity matters (gaze L2 + forge L1 lesson on SelectOutcome::Recv). The `SelectOutcome::Recv { index, result }` pattern from Slice 1 IS the model — Select::select() returns the struct-variant form.
4. **Newtype + accessor pattern** for error-message-style types (forge L1 lesson on CloseError/WireError). All Slice 1 error types already follow this; Slice 2 uses them as-is (don't re-mint string-wrapper errors).
5. **Comments explain WHY not WHAT** (gaze L2 lesson on TryRecvError "OR"). Each doc comment should say why a thing matters or what the caller must understand — not parrot the type signature.

## Concrete deliverables list

1. New file: `src/comms/thread.rs` (~180-220 LOC: traits impls + cascade-aware recv + Select + factories + module doc)
2. Edit: `src/comms/mod.rs` (add `pub mod thread;` at the appropriate position; trailing the SelectOutcome block)
3. New file: `tests/probe_comms_thread.rs` (~100-130 LOC: 10 smoke tests with real assertions)
4. SCORE doc: `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-2-THREAD-TIER.md`

## Critical constraints

- DO NOT commit. Orchestrator commits after independent SCORE verification + ward pass.
- Anchor cwd: `/home/watmin/work/holon/wat-rs/` — `pwd` as first action; reject any `.claude/worktrees/` path.
- Use `git -C` for any git operations.

## Cross-references

- `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` — full arc 214 design; Slice 2 description
- `docs/arc/2026/05/214-concurrency-toolkit/WARD-PASS-1-FOUNDATION-PRIMITIVES.md` — Slice 1's ward round-trip; lessons to pre-empt
- `src/typed_channel.rs:541+` — χ-1's existing wrapper (PATTERN REFERENCE; DO NOT TOUCH)
- `src/typed_channel.rs:295-345` — typed_recv's cascade-aware select pattern (READ-ONLY reference)
- `src/runtime.rs:179` — SHUTDOWN_RX definition
- `src/comms/mod.rs` — Slice 1 traits + error types + SelectOutcome + ReceiverIndex
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — per-slice trust gate protocol
- `feedback_no_hang_vector_in_additive_scorecard` — why no wat_arc170 in verification
- `feedback_defect_fix_or_panic_never_revert` — dirty tree preservation
