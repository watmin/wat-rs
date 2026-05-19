# Arc 214 Slice 3 — Stone D2 — Select<'a, T>

## Mission

Stone D2 is the FIFTH of six stepping stones in Slice 3. It proves **one** thing: cascade-aware fan-in over N receivers via io_uring multi-arm POLL_ADD generalized from Stone B's 2-arm pattern to N+1 arms (N data + 1 broadcast).

This is the most novel substrate work in Slice 3 — Stone B's `wait_for_data_or_cascade` poll-then-read pattern generalized to arbitrary-fan-in scale. The diagnostic value of isolating Select in its own stone is high: if a ward finding surfaces here, it can ONLY be about N-arm POLL_ADD orchestration (not about mechanical method extensions, which Stone D1 already proved).

After Stone D2, `comms::process::Select<'a, T>` matches `comms::thread::Select<'a, T>` (Slice 2) — same surface, different transport. Process tier is now structurally indistinguishable from thread tier to consumers.

Stone D2 still defers:
- Persistent IoUring + config tunable (Stone E)

## Stepping stone roadmap (Slice 3; informational — do not implement beyond Stone D2)

- **Stone A (shipped):** io_uring bytes-only foundation
- **Stone B (shipped):** cascade-aware 2-arm POLL_ADD
- **Stone C (shipped):** generic `T: HolonRepresentable` + serialization
- **Stone D1 (just-shipped pre-D2):** mechanical methods (try_recv + len + close + Clone + traits)
- **Stone D2 (this work):** `Select<'a, T>` + Default impl (N+1-arm fan-in)
- **Stone E:** Persistent IoUring per Receiver + `:wat::config::set-process-tier-uring-depth!`

## Substrate context (substrate-truth verified pre-spawn)

- **Stone B's `wait_for_data_or_cascade`** pattern (process.rs; PRESERVED unchanged) — submits POLL_ADD on `[data_fd, broadcast_fd]` via per-call `IoUring::new(4)`; drains CQEs; broadcast wins ties. Stone D2's Select::select generalizes this to N+1 arms.
- **Slice 2 thread tier `Select<'a, T>`** at `src/comms/thread.rs:Select-*` — Stone D2's process tier Select MIRRORS this API surface (struct + new + recv + select + Default) with io_uring underneath instead of crossbeam.
- **Slice 1 `ReceiverIndex` + `SelectOutcome<T>`** at `src/comms/mod.rs`:
  - `pub struct ReceiverIndex(pub usize);` — newtype for the user-facing arm index
  - `pub enum SelectOutcome<T> { Recv { index: ReceiverIndex, result: Result<T, RecvError> }, Shutdown }` — fan-in result
- **Stone C's `decode_frame::<T>`** (PRESERVED) — used by Select::select to convert bytes → T after a frame arrives.
- **Stone A's `take_frame`** (PRESERVED) — used by Select::select fast-path + slow-path.
- **D1's Receiver state model** — each Receiver has `read_fd: OwnedFd` + `accumulator: RefCell<Vec<u8>>` + `_phantom: PhantomData<T>`. Select holds `&'a Receiver<T>` references; user_arm index → receiver lookup.
- **io_uring N+1-arm pattern (extends Stone B):**
  - Per-call IoUring sized to accommodate N+1 POLL_ADD SQEs (power-of-2 ceiling)
  - user_data tokens: `0 = BROADCAST_TOKEN`; `1..=N = data arm tokens (token-1 = user_arm_index)`
  - Submit all N+1 POLL_ADDs (or N if broadcast uninitialized); `submit_and_wait(1)`; drain all ready CQEs
  - Broadcast wins ties; otherwise first data arm wins this call (other arms picked up on next select())

## Concrete deliverables

### 1. Update `src/comms/process.rs` module-level doc

Replace `## Current scope (through Stone D1)` section with:

```rust
//! ## Current scope (through Stone D2)
//!
//! Full API surface matching the thread tier (`crate::comms::thread`).
//! Generic `Sender<T: HolonRepresentable>` / `Receiver<T: HolonRepresentable>`
//! with HolonAST ↔ EDN bytes via wat-edn (Stone C). Cascade-aware multi-arm
//! POLL_ADD (Stone B). io_uring bytes foundation with newline framing
//! (Stone A). Stone D1: try_recv + len + close + Clone + CommSender/
//! CommReceiver trait impls. Stone D2: `Select<'a, T>` — cascade-aware
//! fan-in over N receivers (generalizes Stone B's 2-arm POLL_ADD to
//! N+1 arms; broadcast wins ties). Only NO persistent ring / config
//! tunable (Stone E).
```

### 2. Add imports

Update the existing import block in `src/comms/process.rs`:

```rust
use crate::comms::{
    CloseError, CommReceiver, CommSender, HolonRepresentable, ReceiverIndex,
    RecvError, SelectOutcome, SendError, TryRecvError,
};
```

Adds `ReceiverIndex` + `SelectOutcome` to D1's imports.

### 3. Add `Select<'a, T>` + impl + Default

Append at the end of `src/comms/process.rs` (after the `pair<T>()` factory):

```rust
// ─── Select ──────────────────────────────────────────────────────────────────

/// Cascade-aware fan-in over multiple process-tier receivers. Mirrors
/// the thread-tier `Select` shape (`src/comms/thread.rs`) — same API
/// surface, different transport underneath.
///
/// User-registered receivers get `ReceiverIndex`es in registration
/// order (0, 1, 2, ...). The substrate's `SHUTDOWN_BROADCAST_READ_FD`
/// is auto-polled on every `select()` call when initialized — the
/// broadcast arm has no user-facing index; it surfaces as
/// `SelectOutcome::Shutdown`.
///
/// On `select()`:
///   - Broadcast arm fired → `SelectOutcome::Shutdown` (broadcast wins
///     ties; substrate going down; honest reporting per
///     typed_channel.rs:360-364 discipline).
///   - One or more data arms fired → drain the first data CQE; do an
///     io_uring Read on that receiver; accumulate; if a complete frame
///     is decoded → `SelectOutcome::Recv { index, result }`; if partial
///     → loop and re-poll all arms (broadcast can fire mid-drain).
///
/// Per-call IoUring sized for N+1 POLL_ADD entries (Stone E persistifies).
pub struct Select<'a, T: HolonRepresentable> {
    /// User-registered receivers in registration order. The index
    /// into this Vec is the user-facing `ReceiverIndex`.
    receivers: Vec<&'a Receiver<T>>,
    /// Type marker for the payload type T. PhantomData<T> makes
    /// `Select<'a, T>` invariant in T — consistent with `Sender<T>`
    /// and `Receiver<T>`.
    _phantom: PhantomData<T>,
}

impl<'a, T: HolonRepresentable> Select<'a, T> {
    /// Construct a new cascade-aware Select. Empty until receivers
    /// are registered via `recv`. The broadcast arm is NOT registered
    /// here — it's polled per-`select()` call based on the current
    /// `SHUTDOWN_BROADCAST_READ_FD` atomic value (idempotent-set per
    /// substrate init).
    pub fn new() -> Self {
        Self {
            receivers: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Register a receiver. Returns the `ReceiverIndex` the caller
    /// will see in `SelectOutcome::Recv { index, .. }` when this
    /// receiver fires. Index reflects registration order (0 for first
    /// registered, 1 for second, etc.).
    pub fn recv(&mut self, rx: &'a Receiver<T>) -> ReceiverIndex {
        let user_idx = self.receivers.len();
        self.receivers.push(rx);
        ReceiverIndex(user_idx)
    }

    /// Block until any registered receiver has a complete frame OR
    /// substrate shutdown fires. Returns the outcome.
    ///
    /// Fast path: check all receivers' accumulators for a buffered
    /// complete frame; if found, return that immediately (no io_uring).
    ///
    /// Slow path: per-call IoUring; submit POLL_ADD for each data fd
    /// + broadcast fd (when initialized); wait for any to fire; drain
    /// CQEs; broadcast wins ties; if a data arm fired, Read from that
    /// arm into its accumulator; if a complete frame is decoded,
    /// return; if partial, loop.
    pub fn select(&mut self) -> SelectOutcome<T> {
        // Fast path — any accumulator already has a complete frame?
        for (i, rx) in self.receivers.iter().enumerate() {
            if let Some(frame) = take_frame(&mut rx.accumulator.borrow_mut()) {
                return SelectOutcome::Recv {
                    index: ReceiverIndex(i),
                    result: decode_frame::<T>(&frame),
                };
            }
        }

        let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD
            .load(std::sync::atomic::Ordering::SeqCst);

        loop {
            // Per-call IoUring sized for N+1 POLL_ADD entries.
            // user_data scheme: 0 = broadcast; 1..=N = data arms.
            let arm_count = self.receivers.len() + if broadcast_fd >= 0 { 1 } else { 0 };
            // io-uring crate requires power-of-2-or-greater capacity.
            // Use next_power_of_two to round up; floor at 2.
            let ring_capacity = ((arm_count.max(1)).next_power_of_two() as u32).max(2);

            let mut ring = match IoUring::new(ring_capacity) {
                Ok(r) => r,
                Err(_) => {
                    // io_uring setup failure is fail-stop at this layer.
                    // Honest reporting: surface as a synthetic Recv with
                    // Err(RecvError) on arm 0. Index is arbitrary because
                    // no arm actually fired — the SUBSTRATE failed.
                    return SelectOutcome::Recv {
                        index: ReceiverIndex(0),
                        result: Err(RecvError),
                    };
                }
            };

            const BROADCAST_TOKEN: u64 = 0;

            if broadcast_fd >= 0 {
                let poll_broad = opcode::PollAdd::new(
                    types::Fd(broadcast_fd),
                    libc::POLLHUP as u32,
                )
                .build()
                .user_data(BROADCAST_TOKEN);
                // SAFETY: broadcast_fd is owned by the substrate worker
                // and remains valid for the lifetime of submit_and_wait.
                unsafe {
                    if ring.submission().push(&poll_broad).is_err() {
                        return SelectOutcome::Recv {
                            index: ReceiverIndex(0),
                            result: Err(RecvError),
                        };
                    }
                }
            }

            for (i, rx) in self.receivers.iter().enumerate() {
                let poll_data = opcode::PollAdd::new(
                    types::Fd(rx.read_fd.as_raw_fd()),
                    (libc::POLLIN | libc::POLLHUP) as u32,
                )
                .build()
                .user_data((i + 1) as u64);
                // SAFETY: rx.read_fd is owned by the Receiver pointed to
                // by 'a; remains valid for the lifetime of submit_and_wait.
                unsafe {
                    if ring.submission().push(&poll_data).is_err() {
                        return SelectOutcome::Recv {
                            index: ReceiverIndex(0),
                            result: Err(RecvError),
                        };
                    }
                }
            }

            if ring.submit_and_wait(1).is_err() {
                return SelectOutcome::Recv {
                    index: ReceiverIndex(0),
                    result: Err(RecvError),
                };
            }

            // Drain ALL ready CQEs — both broadcast and data arms may
            // fire simultaneously. Broadcast wins ties.
            let mut fired_broadcast = false;
            let mut first_data_arm: Option<usize> = None;
            while let Some(cqe) = ring.completion().next() {
                if cqe.result() < 0 {
                    return SelectOutcome::Recv {
                        index: ReceiverIndex(0),
                        result: Err(RecvError),
                    };
                }
                let token = cqe.user_data();
                if token == BROADCAST_TOKEN {
                    fired_broadcast = true;
                } else {
                    // token in 1..=N; arm index is (token - 1).
                    let arm = (token - 1) as usize;
                    // First data arm wins; later ones ignored this call
                    // (picked up on next select() iteration).
                    if first_data_arm.is_none() {
                        first_data_arm = Some(arm);
                    }
                }
            }

            // Broadcast wins ties — substrate going down.
            if fired_broadcast {
                return SelectOutcome::Shutdown;
            }

            let arm_idx = match first_data_arm {
                Some(i) => i,
                None => {
                    // Defensive — submit_and_wait(1) returned but no
                    // CQE drained. Should not happen; retry.
                    continue;
                }
            };

            // Read from the fired arm — do ONE io_uring Read.
            let rx = self.receivers[arm_idx];
            let read_fd = rx.read_fd.as_raw_fd();
            let mut read_ring = match IoUring::new(2) {
                Ok(r) => r,
                Err(_) => {
                    return SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: Err(RecvError),
                    };
                }
            };
            let mut buf = [0u8; 4096];
            let read_e = opcode::Read::new(
                types::Fd(read_fd),
                buf.as_mut_ptr(),
                buf.len() as _,
            )
            .build()
            .user_data(1);
            // SAFETY: read_e's buf pointer outlives submit_and_wait
            // because buf is on this function's stack and not freed
            // until after the wait completes.
            unsafe {
                if read_ring.submission().push(&read_e).is_err() {
                    return SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: Err(RecvError),
                    };
                }
            }
            if read_ring.submit_and_wait(1).is_err() {
                return SelectOutcome::Recv {
                    index: ReceiverIndex(arm_idx),
                    result: Err(RecvError),
                };
            }
            let cqe = match read_ring.completion().next() {
                Some(c) => c,
                None => {
                    return SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: Err(RecvError),
                    };
                }
            };
            let result = cqe.result();
            if result <= 0 {
                // Error (<0) or EOF (==0) → arm disconnected.
                return SelectOutcome::Recv {
                    index: ReceiverIndex(arm_idx),
                    result: Err(RecvError),
                };
            }
            let n = result as usize;
            rx.accumulator
                .borrow_mut()
                .extend_from_slice(&buf[..n]);

            if let Some(frame) = take_frame(&mut rx.accumulator.borrow_mut()) {
                return SelectOutcome::Recv {
                    index: ReceiverIndex(arm_idx),
                    result: decode_frame::<T>(&frame),
                };
            }
            // Partial bytes; no complete frame yet. Loop and re-poll
            // all arms (broadcast can fire mid-drain).
        }
    }
}

impl<'a, T: HolonRepresentable> Default for Select<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}
```

### 4. Extend `tests/comms/process.rs` with Stone D2 tests

KEEP all 6 existing `probe_slice3c_*` tests + all 10 existing `probe_slice3d1_*` tests unchanged.

ADD 2 new `probe_slice3d2_*` tests:

```rust
// Add to imports at top of file (D1 likely already added some of these):
use wat::comms::process::Select;
use wat::comms::{ReceiverIndex, SelectOutcome};

// Append AFTER the existing 16 tests:

#[test]
fn probe_slice3d2_select_picks_fired_receiver() {
    // Verifies Select returns the correct ReceiverIndex + value when
    // exactly one of two registered receivers has a queued frame.
    let (tx_a, rx_a) = pair::<String>().expect("pair a");
    let (_tx_b, rx_b) = pair::<String>().expect("pair b");
    tx_a.send("hello-a".to_string()).expect("send to rx_a");
    // Give the kernel a moment to deliver.
    thread::sleep(Duration::from_millis(20));
    let mut sel: Select<String> = Select::new();
    let idx_a = sel.recv(&rx_a);
    // Register rx_b too so Select genuinely has two arms;
    // returned index intentionally unused.
    let _idx_b = sel.recv(&rx_b);
    match sel.select() {
        SelectOutcome::Recv { index, result } => {
            assert_eq!(index, idx_a, "fired index must match the receiver with data");
            assert_eq!(result, Ok("hello-a".to_string()), "result must carry the sent value");
        }
        SelectOutcome::Shutdown => panic!("unexpected Shutdown"),
    }
}

#[test]
fn probe_slice3d2_select_indices_match_registration_order() {
    // Verifies ReceiverIndex reflects registration order (0, 1, 2)
    // independent of any io_uring internal token scheme.
    let (_tx_a, rx_a) = pair::<String>().expect("pair a");
    let (_tx_b, rx_b) = pair::<String>().expect("pair b");
    let (_tx_c, rx_c) = pair::<String>().expect("pair c");
    let mut sel: Select<String> = Select::new();
    let idx_a = sel.recv(&rx_a);
    let idx_b = sel.recv(&rx_b);
    let idx_c = sel.recv(&rx_c);
    assert_eq!(idx_a, ReceiverIndex(0), "first registered receiver must be index 0");
    assert_eq!(idx_b, ReceiverIndex(1), "second registered receiver must be index 1");
    assert_eq!(idx_c, ReceiverIndex(2), "third registered receiver must be index 2");
}
```

2 new tests covering: single-arm fire (data path with cascade-arm registered) + multi-arm registration order. Total file after D2: 6 (Stone C) + 10 (Stone D1) + 2 (Stone D2) = 18 tests.

## Verification

```
cargo build --release                                       # MUST be clean
cargo test --release --test comms                           # 31 PASS total (foundation 3 + thread 10 + process 16 prior + 2 new D2)
cargo test --release --test probe_channel_primitive         # 3/3 PASS unchanged (arc 213 χ-1; flat layout)
cargo test --release --test probe_pidfd_primitive           # 2/2 PASS unchanged (arc 213 α; flat layout)
```

Per `feedback_no_hang_vector_in_additive_scorecard`: **DO NOT** run `wat_arc170_program_contracts`.

## Out of scope (STOP triggers)

- **DO NOT add config tunable** — Stone E
- **DO NOT optimize to persistent IoUring** — Stone E
- **DO NOT modify** Stone A's `take_frame`, Stone B's `wait_for_data_or_cascade` / `PollOutcome`, Stone C's `decode_frame` / `Sender::send` / `Receiver::recv` / `pair<T>()`, Stone D1's methods + trait impls
- **DO NOT touch the dirty tree**
- **DO NOT touch `src/typed_channel.rs`**, `src/edn_shim.rs`, `src/comms/mod.rs`, `Cargo.toml`
- **DO NOT run** `wat_arc170_program_contracts`
- **ZERO modifications** outside the 2-file scope (`src/comms/process.rs` adds Select struct + impl + Default; `tests/comms/process.rs` adds 2 tests + Select/ReceiverIndex/SelectOutcome imports) + SCORE doc

## Pre-emptive ward discipline (lessons from Slices 1+2 + Stones A+B+C+D1)

1. **Module-level doc update** (Stone B gaze L1 lesson) — replace "Current scope (through Stone D1)" with "(through Stone D2)" naming the new Select capability.
2. **NO struct doc updates needed** — Sender/Receiver are unchanged in Stone D2 (Select is a separate struct).
3. **All new `unsafe` blocks carry SAFETY comments** (Stone A round-1 forge lesson) — Select::select has TWO unsafe blocks (POLL_ADD push + Read push); EACH needs a SAFETY comment naming the fd lifetime invariant.
4. **`PhantomData<T>` for Select** — invariance discipline; document briefly.
5. **Broadcast wins ties** — same substrate-invariant as Stone B; doc-comment names this.
6. **CQE drain via `while let Some(cqe)`** (Stone B precedent) — both arms may fire simultaneously; NOT `if let Some(cqe)`.
7. **Bail-out pattern: synthetic SelectOutcome::Recv with ReceiverIndex(0) + Err(RecvError)** on io_uring substrate failures — honest at this layer; the index is arbitrary because no arm actually fired.
8. **Fast-path accumulator iteration order** — `for (i, rx) in self.receivers.iter().enumerate()` — natural index order; matches registration order priority.
9. **Probe test names** use `probe_slice3d2_*` prefix; previous tests (3c + 3d1) are unchanged.

## Concrete deliverables list

1. **Edit** `src/comms/process.rs` — module-level doc updated; imports add `ReceiverIndex` + `SelectOutcome`; `Select<'a, T>` struct + impl with `new` + `recv` + `select` + `Default` impl appended at end of file
2. **Edit** `tests/comms/process.rs` — preserve 16 existing tests; add 2 new `probe_slice3d2_*` tests; add Select/ReceiverIndex/SelectOutcome imports
3. **New file** SCORE doc: `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-3D2-SELECT.md`

Estimated LOC: ~150-180 LOC added to `src/comms/process.rs`; ~70-90 LOC added to `tests/comms/process.rs`. Total stone delta: ~220-270 LOC.

## Critical constraints

- **DO NOT commit.** Orchestrator commits after SCORE verification + 5-ward pass.
- **Anchor cwd:** `/home/watmin/work/holon/wat-rs/`
- **Use `git -C`** for git ops

## Cross-references

- BRIEF-214-SLICE-3D1-MECHANICAL-METHODS.md — Stone D1 (just-shipped pre-D2)
- BRIEF-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md — Stone B (2-arm pattern; D2 generalizes to N+1)
- WARD-PASS-3A through 3D1 — prior round-trips
- `src/comms/thread.rs` — Slice 2 thread-tier Select<'a, T> MIRROR REFERENCE
- `src/comms/mod.rs` — Slice 1 `ReceiverIndex` + `SelectOutcome<T>`
- `feedback_iterative_complexity` — D2 split from original Stone D per four-questions
