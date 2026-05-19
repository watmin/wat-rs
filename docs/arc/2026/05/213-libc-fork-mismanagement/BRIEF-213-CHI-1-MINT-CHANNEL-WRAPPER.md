# Arc 213 stone χ-1 — Mint `wat::channel::{Sender<T>, Receiver<T>}` wrapper

## Substrate gap

The substrate has TWO channel domains:

1. **Wat-runtime Value layer** (`src/typed_channel.rs:203` typed_send, `:295` typed_recv) — operates on `runtime::Value`; routes through `SHUTDOWN_RX` cascade-aware `crossbeam_channel::select!` (line 304-313). Already chokepointed.

2. **Rust-level T-typed channels** — substrate-internal plumbing uses bare `crossbeam_channel::{Sender<T>, Receiver<T>}` directly. **35 bare-method sites bypass the cascade discipline** across 4 files:

   | Op | Sites | Per file |
   |---|---|---|
   | `.recv()` | 15 | 9 thread_io, 5 runtime, 1 freeze |
   | `.send()` | 19 | 15 thread_io, 3 runtime, 1 spawn |
   | `.try_recv()` | 1 | 1 runtime |

When the substrate's shutdown cascade fires (`init_shutdown_signal` in freeze.rs drops SHUTDOWN_TX → crossbeam intrusive park-list wakes all parked recvs on SHUTDOWN_RX clones), only callers routing through cascade-aware `select!` wake. **The 15 bare-recv sites do not wake. Parent process hangs. Orphan accumulates.** This is the documented diagnosis at INTERSTITIAL § 2026-05-18 (post-δ-1 investigation) "Channel-cascade-completeness wall."

The χ doctrine (load-bearing, user 2026-05-18): *"we are our own users — and i don't want to observe this failure ever again."* The fix is structural: substrate-owned channel newtypes; bare crossbeam usage becomes a compile error outside the wrapper module. Same shape as arc 198 `restricted_to`, arc 203 struct-restricted, arc 212 ζ `WatAST::children()`.

## Mission

**χ-1 mints the wrapper. Additive only. No migration.**

Add to `src/typed_channel.rs` (alongside existing Value-layer code):

```rust
// === Rust-level T-typed channel chokepoint (arc 213 χ) ===
//
// Wraps crossbeam_channel::{Sender<T>, Receiver<T>} for substrate-internal
// T-typed plumbing. Recv routes through SHUTDOWN_RX cascade per arc 213
// χ doctrine: "we are our own users — and i don't want to observe this
// failure ever again." Bare crossbeam usage becomes restricted in χ-3.

/// Error types re-exported for mechanical migration parity with
/// crossbeam_channel. Callers see identical Result shapes.
pub use crossbeam_channel::{RecvError, SendError, TryRecvError};

/// χ-1: T-typed Sender wrapper. Currently a thin newtype; cascade
/// semantics are on the Receiver side. Mechanical migration parity
/// with crossbeam_channel::Sender<T>::send signature.
pub struct Sender<T> {
    inner: crossbeam_channel::Sender<T>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.inner.send(value)
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

/// χ-1: T-typed Receiver wrapper. recv() routes through SHUTDOWN_RX
/// cascade-aware select — when substrate shutdown fires, parked recvs
/// wake with Err(RecvError) instead of hanging indefinitely.
///
/// Bootstrap fallback: when SHUTDOWN_RX is not yet initialized (pre-init
/// or test bypass), recv falls back to bare crossbeam recv. Production
/// paths always have SHUTDOWN_RX initialized by freeze.rs:233 before any
/// wat code executes.
pub struct Receiver<T> {
    inner: crossbeam_channel::Receiver<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, RecvError> {
        // Cascade-aware select — mirrors typed_recv's Crossbeam arm
        // (src/typed_channel.rs:304-313). On SHUTDOWN_RX signal, recv
        // returns Err(RecvError) — caller treats as channel-died and
        // unwinds. Identical to crossbeam_channel's Disconnected.
        let shutdown_rx = crate::runtime::SHUTDOWN_RX.get();
        match shutdown_rx {
            Some(srx) => {
                crossbeam_channel::select! {
                    recv(&self.inner) -> msg => msg,
                    recv(srx) -> _ => Err(RecvError),
                }
            }
            None => self.inner.recv(),  // bootstrap fallback
        }
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        // Non-blocking; cascade-irrelevant (returns immediately).
        self.inner.try_recv()
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (Sender { inner: tx }, Receiver { inner: rx })
}

pub fn bounded<T>(n: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = crossbeam_channel::bounded(n);
    (Sender { inner: tx }, Receiver { inner: rx })
}
```

## Smoke probe

Create `tests/probe_channel_primitive.rs` with 3 tests:

```rust
//! Arc 213 χ-1 smoke probe — wat::channel wrapper basic semantics.
//! Cascade-awareness verified by χ-4's 50-trial replication proof under
//! real runtime conditions; this probe verifies the wrapper itself
//! behaves as a channel.

use wat::typed_channel::{unbounded, RecvError, TryRecvError};

#[test]
fn probe_chi1_unbounded_round_trip() {
    let (tx, rx) = unbounded::<i32>();
    tx.send(42).expect("send");
    assert_eq!(rx.recv().expect("recv"), 42);
}

#[test]
fn probe_chi1_sender_drop_triggers_recv_err() {
    let (tx, rx) = unbounded::<i32>();
    drop(tx);
    assert!(matches!(rx.recv(), Err(RecvError)));
}

#[test]
fn probe_chi1_try_recv_empty_returns_empty() {
    let (_tx, rx) = unbounded::<i32>();
    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));
}
```

## Verification

```
cargo build --release                                  # must be clean
cargo test --release --test probe_channel_primitive    # 3/3 pass
```

**That is the FULL verification. STOP here.**

**DO NOT run `cargo test --release --test wat_arc170_program_contracts` or any other baseline cargo test.** That test is the EXACT hang vector χ exists to fix (it exercises t15_spawn_process_child_panic_disconnects_recv_and_exits_nonzero which hangs on the cascade-completeness gap). Re-running it as a "baseline" verification produces orphan processes + indefinite hangs.

χ-1 is purely additive: ONLY adds new symbols in src/typed_channel.rs + a new test file. It does NOT modify any existing code path. Therefore "baseline unchanged" is true IN PRINCIPLE — cargo build --release succeeding is sufficient evidence that no existing code was perturbed.

## Out of scope (STOP triggers)

- **DO NOT migrate any of the 35 bare-recv/send/try_recv sites.** That's χ-2.
- **DO NOT add `#[restricted_to(...)]` to `crossbeam_channel` imports.** That's χ-3.
- **DO NOT modify `src/fork.rs` or `src/spawn_process.rs`.** Those are δ-1's uncommitted dirty-tree replication artifacts. Leave them alone.
- **DO NOT touch existing typed_send / typed_recv / SenderInner / ReceiverInner code.** The Value-layer chokepoint is already correct; χ-1 is the SIBLING T-typed wrapper.
- **DO NOT investigate the δ-1 hang replication itself.** That replication is the orchestrator's responsibility post-χ-4 (50-trial proof).
- If `cargo build --release` shows ANY error after your edits: STOP, do NOT iterate-to-green, report the error and your last edit verbatim. χ-1 is purely additive and should compile clean on first attempt.
- If the probe's 3 tests don't all PASS: STOP and report. Do not modify probe semantics to make them pass.

## Concrete deliverables

1. Edits to `src/typed_channel.rs` adding the wrapper code above (placement: after existing typed_try_recv at line 407 OR at end of file — sonnet's choice based on local convention)
2. New file `tests/probe_channel_primitive.rs` with the 3 tests above
3. SCORE doc `docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-CHI-1-MINT-CHANNEL-WRAPPER.md` per EXPECTATIONS scorecard

## Critical constraints

- DO NOT commit. Orchestrator commits after independent SCORE verification.
- DO NOT touch any file outside `src/typed_channel.rs` + `tests/probe_channel_primitive.rs` + the SCORE doc.
- The dirty tree (src/fork.rs + src/spawn_process.rs) MUST remain untouched — it is the precious δ-1 replication preserved per `feedback_defect_fix_or_panic_never_revert`.
- Use the existing CWD; do not cd to subdirs.

## Cross-references

- Arc 213 DESIGN (stone chain documented in INTERSTITIAL § "Channel-cascade-completeness wall")
- `src/typed_channel.rs:203` typed_send (sibling code at the Value layer)
- `src/typed_channel.rs:295` typed_recv (the cascade-aware pattern this wrapper mirrors)
- `src/typed_channel.rs:304-313` the crossbeam select! shutdown discipline being replicated
- INTERSTITIAL § 2026-05-18 (post-δ-1 investigation) "Channel-cascade-completeness wall (arc 213 χ) + the 'we are our own users' doctrine" — the load-bearing doctrine
- Same-shape precedents: arc 198 `restricted_to`, arc 203 struct-restricted, arc 212 ζ `WatAST::children()`
