# Arc 214 Slice 3 — Stone E-2 — Select persistent ring (reflexive rebuild) + Receiver method extraction

## Mission

Stone E-2 is the SEVENTH and FINAL stepping stone in Slice 3. It proves **two converging concerns** in one stone (per WARD-PASS-3E1.md § "Deferred to Stone E-2"):

1. **Select gains a persistent ring** with reflexive rebuild-on-capacity-mismatch (grow OR shrink). The substrate maintains the invariant `current_capacity == next_power_of_two(arm_count + 1)` at every `select()` entry. Per-call `IoUring::new(ring_capacity)` retires.

2. **Receiver gains method extraction** (`read_into_acc` + `take_buffered_frame`) — Solvere's L2 finding from the E-1 ward pass identified that Select reaches into Receiver internals (`rx.read_fd`, `rx.accumulator`, `rx.ring`) at 4 sites. The braid retires when Receiver exposes methods and Select composes via those.

These two converge in Select::select's body — persisting the ring AND replacing field-access with method-calls happen in the same function. Bundling honors solvere's framing that E-2 IS the natural resolution point.

Migrates the last `rune:temperare(no-reactor)` (line 742; Select POLL_ADD ring) from warm to cold. Closes the entire arc 214 slice 3 io_uring heat catalog.

## Stepping stone roadmap (Slice 3; informational — do not implement beyond E-2)

- **Stone A (shipped):** io_uring bytes-only foundation
- **Stone B (shipped):** cascade-aware 2-arm POLL_ADD
- **Stone C (shipped):** generic `T: HolonRepresentable` + serialization
- **Stone D1 (shipped):** mechanical methods (try_recv + len + close + Clone + traits)
- **Stone D2 (shipped):** `Select<'a, T>` cascade-aware N+1-arm fan-in
- **Stone E-1 (shipped):** Receiver persistent ring (capacity 4; static-need)
- **Stone E-2 (this work):** Select persistent ring (reflexive rebuild) + Receiver method extraction

## Substrate context (substrate-truth verified pre-spawn)

- **`src/comms/process.rs:205-225`** — Receiver<T> struct: `read_fd: OwnedFd`, `accumulator: Accumulator` (Stone E-1 typealias for `RefCell<Vec<u8>>`), `ring: RefCell<IoUring>` (Stone E-1), `_phantom`. E-2 does NOT add fields to Receiver — only methods.

- **`src/comms/process.rs:240-310`** — Receiver::recv body. Currently calls `wait_for_data_or_cascade(read_fd, broadcast_fd, &self.ring)` + `uring_read_into_acc(read_fd, &self.accumulator, &self.ring)` as free functions taking `&Accumulator` + `&RefCell<IoUring>`. E-2 refactors: introduce `Receiver::read_into_acc(&self) -> Result<usize, ()>` method wrapping the existing free function; recv calls `self.read_into_acc()`. Same shape for try_recv.

- **`src/comms/process.rs:716-718`** — Select::select fast-path accumulator scan: `if let Some(frame) = take_frame(&mut rx.accumulator.borrow_mut())`. E-2 introduces `Receiver::take_buffered_frame(&self) -> Option<Vec<u8>>` method wrapping `take_frame(&mut self.accumulator.borrow_mut())`; Select's fast path becomes `if let Some(frame) = rx.take_buffered_frame()`.

- **`src/comms/process.rs:737-749`** — Select::select slow-path ring construction. Inside the `loop { ... }` body, per-call `IoUring::new(ring_capacity)` constructs a fresh ring each iteration. E-2 lifts this OUT of the loop and into a reflexive-rebuild check at the TOP of select().

- **`src/comms/process.rs:824` (approx)** — Select::select Read step: `match uring_read_into_acc(rx.read_fd.as_raw_fd(), &rx.accumulator, &rx.ring)`. E-2 refactors to `match rx.read_into_acc()`.

- **`src/comms/process.rs:835` (approx)** — Select::select partial-frame check after Read: `if let Some(frame) = take_frame(&mut rx.accumulator.borrow_mut())`. E-2 refactors to `if let Some(frame) = rx.take_buffered_frame()`.

- **`io_uring` crate (0.7) ring capacity introspection:** `IoUring::params().sq_entries` returns the configured SQ size. To keep the comparison cheap + explicit, store capacity alongside the ring as a tuple (`RefCell<Option<(IoUring, u32)>>`) — the field carries its own answer; no need to introspect the crate's internals on every select() call.

- **Borrow scoping:** `Select.ring` (RefCell<Option<(IoUring, u32)>>) and `Receiver.ring` (RefCell<IoUring>) are DIFFERENT RefCells; no borrow-conflict. Within Select::select's body, scope the Select-ring borrow so it releases BEFORE calling `rx.read_into_acc()` (which borrows rx.ring). Block-scope with `{ ... }` if needed.

- **Reflexive rebuild discipline (per DESIGN.md § "Stone E forward-correction"):** at every select() entry, compute `needed_capacity = next_power_of_two(arm_count + broadcast_arm + 1).max(2)`. If `ring.is_none()` OR stored capacity `!=` needed_capacity (GROW OR SHRINK), construct new ring; assign to field. Else reuse. Invariant `current_capacity == needed_capacity` holds at every select() entry. The replacement IS the tail call: old ring drops; new ring constructs; receivers + FDs untouched.

## Concrete deliverables

### 1. Update `src/comms/process.rs` module-level doc

Replace `## Current scope (through Stone E-1)` block's last sentence with one that names E-2:

```rust
//! ## Current scope (through Stone E-2)
//!
//! Full API surface matching the thread tier (`crate::comms::thread`).
//! Generic `Sender<T: HolonRepresentable>` / `Receiver<T: HolonRepresentable>`
//! with HolonAST ↔ EDN bytes via wat-edn (Stone C). Cascade-aware multi-arm
//! POLL_ADD (Stone B). io_uring bytes foundation with newline framing
//! (Stone A). Stone D1: try_recv + len + close + Clone + CommSender/
//! CommReceiver trait impls. Stone D2: `Select<'a, T>` — cascade-aware
//! fan-in over N receivers (generalizes Stone B's 2-arm POLL_ADD to
//! N+1 arms; broadcast wins ties). Stone E-1: Receiver owns persistent
//! IoUring (capacity 4) for its lifetime; helpers operate on the
//! Receiver's ring instead of per-call construction. Stone E-2: Select
//! owns a persistent IoUring with reflexive rebuild-on-capacity-mismatch
//! (grow OR shrink); Receiver gains `read_into_acc` + `take_buffered_frame`
//! methods so Select composes via Receiver's surface instead of reaching
//! into its fields.
//!
//! The underlying principle (FDs are the persistent state; io_urings are
//! ephemeral frames sized to the current operation set; substrate maintains
//! the invariant `cap == next_power_of_two(structural_need + 1)` reflexively
//! at every operation entry) is detailed in
//! `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` §
//! "Stone E forward-correction (2026-05-19) — TCO discipline + reflexive rebuild".
```

### 2. Update audience section (line 62)

Replace `Stone E-2's reflexive-rebuild ring persistification` reference:

```rust
//! Substrate-internal Rust code (Stone D's `Select`, Slice 4's kernel
//! dispatcher). User code does NOT touch this tier.
```

(Stone E-2 IS this stone; once shipped the audience entry retires.)

### 3. Mint `Receiver::read_into_acc(&self) -> Result<usize, ()>`

Add to `impl<T: HolonRepresentable> Receiver<T>` (the existing impl block; pick a location near the existing recv/try_recv methods):

```rust
/// Issue one io_uring Read on `self.read_fd` into `self.accumulator`
/// using `self.ring`. Returns `Ok(n)` where `n` is bytes appended
/// (0 means EOF / peer closed write end), or `Err(())` on io_uring
/// SQE submission, submit_and_wait, or CQE error.
///
/// Encapsulates the field access pattern `(self.read_fd.as_raw_fd(),
/// &self.accumulator, &self.ring)` so callers — including
/// `Select::select`'s Read step — compose via this surface instead of
/// reaching into the Receiver's private fields. Per Solvere ward (E-1
/// ward pass 2026-05-19; resolution deferred to E-2).
pub(crate) fn read_into_acc(&self) -> Result<usize, ()> {
    uring_read_into_acc(self.read_fd.as_raw_fd(), &self.accumulator, &self.ring)
}
```

### 4. Mint `Receiver::take_buffered_frame(&self) -> Option<Vec<u8>>`

Add similarly:

```rust
/// Pull the first newline-terminated frame out of `self.accumulator`
/// if one is buffered. Returns `None` when no `'\n'` is present
/// (caller should read more bytes via `read_into_acc`).
///
/// Encapsulates the accumulator borrow + `take_frame` call pattern
/// so callers — including `Select::select`'s fast-path scan and
/// partial-frame post-Read check — compose via this surface instead
/// of reaching into the Receiver's accumulator field. Per Solvere
/// ward (E-1 ward pass 2026-05-19; resolution deferred to E-2).
pub(crate) fn take_buffered_frame(&self) -> Option<Vec<u8>> {
    take_frame(&mut self.accumulator.borrow_mut())
}
```

### 5. Refactor `Receiver::recv` to use the new methods (DRY)

At `Receiver::recv` (lines ~240-310):
- The 2 fast-path `take_frame(...)` calls become `self.take_buffered_frame()`
- The Read-step `uring_read_into_acc(read_fd, &self.accumulator, &self.ring)` becomes `self.read_into_acc()`

### 6. Refactor `Receiver::try_recv` to use the new methods (DRY)

At `Receiver::try_recv` (line ~320-360):
- Fast-path `take_frame(...)` call becomes `self.take_buffered_frame()`
- Read-step `uring_read_into_acc(read_fd, &self.accumulator, &self.ring)` becomes `self.read_into_acc()`

### 7. Add `ring` field to `Select<'a, T>` struct

Update the struct definition (lines ~673-682):

```rust
pub struct Select<'a, T: HolonRepresentable> {
    /// User-registered receivers in registration order. The index
    /// into this Vec is the user-facing `ReceiverIndex`.
    receivers: Vec<&'a Receiver<T>>,
    /// Persistent io_uring (Stone E-2) — lazy-initialized on first
    /// `select()` call; reflexively rebuilt on capacity mismatch
    /// (grow OR shrink) when the registered receiver set's structural
    /// need changes. Stored alongside its capacity as a tuple to
    /// avoid crate-internal introspection per call.
    ///
    /// The invariant `cap == next_power_of_two(arm_count + 1)` holds
    /// at every `select()` entry — see DESIGN.md § "Stone E forward-
    /// correction (2026-05-19) — TCO discipline + reflexive rebuild".
    ring: RefCell<Option<(IoUring, u32)>>,
    /// Type marker for the payload type T. PhantomData<T> makes
    /// `Select<'a, T>` invariant in T — consistent with `Sender<T>`
    /// and `Receiver<T>`.
    _phantom: PhantomData<T>,
}
```

### 8. Update `Select::new()` to initialize `ring: RefCell::new(None)`

```rust
pub fn new() -> Self {
    Self {
        receivers: Vec::new(),
        ring: RefCell::new(None),
        _phantom: PhantomData,
    }
}
```

### 9. Refactor `Select::select` to use the persistent ring + Receiver methods

The new body. Key changes:
- Fast-path scan: `rx.take_buffered_frame()` instead of `take_frame(&mut rx.accumulator.borrow_mut())`
- Reflexive rebuild check at the TOP of the loop body (BEFORE submission setup)
- Inner loop borrows `self.ring` (the Select's persistent ring), pushes POLL_ADDs, submits, drains CQEs
- Borrow scope: release the Select ring borrow BEFORE the Read step (which borrows the Receiver's ring)
- Read step: `rx.read_into_acc()` instead of `uring_read_into_acc(...)`
- Partial-frame post-Read check: `rx.take_buffered_frame()` instead of `take_frame(...)`

Skeleton:

```rust
pub fn select(&mut self) -> SelectOutcome<T> {
    // Fast path — any accumulator already has a complete frame?
    for (i, rx) in self.receivers.iter().enumerate() {
        if let Some(frame) = rx.take_buffered_frame() {
            return SelectOutcome::Recv {
                index: ReceiverIndex(i),
                result: decode_frame::<T>(&frame),
            };
        }
    }

    // Group L hoist: current_broadcast_fd() is invariant across loop iterations
    // (cascade fd doesn't change once initialized). Call once before the loop;
    // see helper's rune:sequi(ambient-context) for rationale.
    let broadcast_opt = current_broadcast_fd();

    loop {
        // Compute the structural need: N data arms + 1 broadcast arm (if init).
        // io-uring crate requires power-of-2-or-greater capacity.
        let arm_count = self.receivers.len() + if broadcast_opt.is_some() { 1 } else { 0 };
        let needed_capacity = ((arm_count.max(1)).next_power_of_two() as u32).max(2);

        // Reflexive rebuild discipline (Stone E-2) — at every loop entry,
        // ensure cap == needed_capacity. Lazy init on first call; rebuild
        // on capacity mismatch (grow OR shrink). The replacement IS the
        // tail call: old ring drops; new ring constructs; receivers + FDs
        // untouched. Substrate maintains the invariant reflexively; users
        // never see the io_uring entry count.
        {
            let mut ring_slot = self.ring.borrow_mut();
            let needs_rebuild = match ring_slot.as_ref() {
                None => true,
                Some((_, current_cap)) => *current_cap != needed_capacity,
            };
            if needs_rebuild {
                match IoUring::new(needed_capacity) {
                    Ok(r) => *ring_slot = Some((r, needed_capacity)),
                    Err(e) => return SelectOutcome::SubstrateError(e),
                }
            }
        }
        // Select-ring borrow released; safe to call Receiver methods below
        // (Receiver borrows its own ring; different RefCell).

        const BROADCAST_TOKEN: u64 = 0;

        // Scope-bounded borrow for SQE pushes + submit_and_wait + CQE drain.
        // arm_idx_opt is determined inside this scope; the Read step happens
        // AFTER the borrow releases.
        let arm_idx_opt: Option<usize> = {
            let mut ring_slot = self.ring.borrow_mut();
            // SAFETY of unwrap: reflexive rebuild above guarantees Some(_).
            let ring = &mut ring_slot.as_mut().unwrap().0;

            if let Some(broadcast_fd) = broadcast_opt {
                let poll_broadcast = opcode::PollAdd::new(
                    types::Fd(broadcast_fd),
                    libc::POLLHUP as u32,
                )
                .build()
                .user_data(BROADCAST_TOKEN);
                // SAFETY: broadcast_fd is owned by the substrate worker
                // and remains valid for the lifetime of submit_and_wait.
                unsafe {
                    if ring.submission().push(&poll_broadcast).is_err() {
                        return SelectOutcome::SubstrateError(
                            std::io::Error::other("io_uring SQE push (broadcast POLL_ADD) failed: submission queue full"),
                        );
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
                        return SelectOutcome::SubstrateError(
                            std::io::Error::other("io_uring SQE push (data POLL_ADD) failed: submission queue full"),
                        );
                    }
                }
            }

            if let Err(e) = ring.submit_and_wait(1) {
                return SelectOutcome::SubstrateError(e);
            }

            // Drain ALL ready CQEs — both broadcast and data arms may
            // fire simultaneously. Broadcast wins ties.
            let mut fired_broadcast = false;
            let mut first_data_arm: Option<usize> = None;
            while let Some(cqe) = ring.completion().next() {
                if cqe.result() < 0 {
                    return SelectOutcome::SubstrateError(
                        std::io::Error::from_raw_os_error(-cqe.result()),
                    );
                }
                let token = cqe.user_data();
                if token == BROADCAST_TOKEN {
                    fired_broadcast = true;
                } else {
                    let arm = (token - 1) as usize;
                    if first_data_arm.is_none() {
                        first_data_arm = Some(arm);
                    }
                }
            }

            // Broadcast wins ties — substrate going down.
            if fired_broadcast {
                return SelectOutcome::Shutdown;
            }
            first_data_arm
        };
        // Select-ring borrow released here.

        let arm_idx = match arm_idx_opt {
            Some(i) => i,
            None => {
                // Defensive — submit_and_wait(1) returned but no
                // CQE drained. Should not happen; retry.
                continue;
            }
        };

        // Read from the fired arm via Receiver's surface method —
        // Stone E-2 + Solvere finding closure. The Receiver borrows
        // ITS OWN ring (different RefCell from Select's); no conflict
        // with the Select-ring borrow released above.
        let rx = self.receivers[arm_idx];
        match rx.read_into_acc() {
            Err(_) => {
                return SelectOutcome::Recv {
                    index: ReceiverIndex(arm_idx),
                    result: Err(RecvError),
                };
            }
            Ok(0) => {
                // EOF — peer closed write end.
                return SelectOutcome::Recv {
                    index: ReceiverIndex(arm_idx),
                    result: Err(RecvError),
                };
            }
            Ok(_) => {}
        }

        if let Some(frame) = rx.take_buffered_frame() {
            return SelectOutcome::Recv {
                index: ReceiverIndex(arm_idx),
                result: decode_frame::<T>(&frame),
            };
        }
        // Partial bytes; no complete frame yet. Loop and re-poll
        // all arms (broadcast can fire mid-drain).
    }
}
```

### 10. Retire the `rune:temperare(no-reactor)` at line ~742

The rune disappears entirely. Stone E-2's persistent ring + reflexive rebuild eliminates the per-call construction the rune named. The doc-comment surrounding the rune (which references "Stone E-2 (task #394)" as the future deferral) also retires — Stone E-2 IS this work; the deferral closed.

### 11. Tests — preserve all 34; verify they still pass

E-2 is mechanically invisible to test surface. NO new tests; ALL 34 existing tests must pass unchanged:

- `tests/comms/foundation.rs` — preserved
- `tests/comms/thread.rs` — preserved
- `tests/comms/process.rs` — preserved (all `probe_slice3c_*` + `probe_slice3d1_*` + `probe_slice3d2_*` tests)

## Verification

```
cargo build --release                                       # MUST be clean
cargo test --release --test comms                           # 34/34 PASS (zero net delta from E-1)
cargo test --release --test probe_channel_primitive         # 3/3 PASS unchanged
cargo test --release --test probe_pidfd_primitive           # 2/2 PASS unchanged
```

Per `feedback_no_hang_vector_in_additive_scorecard`: **DO NOT** run `wat_arc170_program_contracts`.

## Out of scope (STOP triggers)

- **DO NOT add config tunable** — DISQUALIFIED by four-questions per DESIGN.md § "Stone E forward-correction (2026-05-19)". No `:wat::config::set-process-tier-uring-depth!` setter.
- **DO NOT add new probe tests** — Stone E-2 is mechanically invisible to test surface; 34/34 existing tests prove correctness via behavior preservation.
- **DO NOT modify** Stone A's `take_frame` free function (still called by the new Receiver method), Stone B's `PollOutcome` / `wait_for_data_or_cascade` (still takes ring param; Receiver::recv routes through it), Stone C's `decode_frame` / `Sender::send` / Sender side of `pair<T>()`, Stone D1's close / len / CommSender/CommReceiver trait impls, Stone E-1's `Receiver` field set / Clone / pair() factory ring construction (E-2 ONLY adds methods to Receiver; does not change its fields).
- **DO NOT touch the dirty tree** — `src/fork.rs` + `src/spawn_process.rs` are arc 213 territory; preserved live-replication state.
- **DO NOT touch** `src/typed_channel.rs`, `src/edn_shim.rs`, `src/comms/mod.rs`, `src/comms/thread.rs`, `Cargo.toml`.
- **DO NOT run** `wat_arc170_program_contracts`.
- **DO NOT keep the free function** `uring_read_into_acc` or `take_frame` if their only callers are now the Receiver methods. (`take_frame` is still used by the Receiver method internally; KEEP. `uring_read_into_acc` is still used internally by Receiver::read_into_acc + still by Receiver::recv's path through wait_for_data_or_cascade + uring_read_into_acc; KEEP.) Both free functions REMAIN — they're the underlying implementations; the methods are thin wrappers.
- **DO NOT keep** the per-call `IoUring::new(ring_capacity)` inside Select::select's loop. The reflexive rebuild at the loop top is the ONLY construction.
- **ZERO modifications** outside the 1-file scope (`src/comms/process.rs`) + SCORE doc.

## Pre-emptive ward discipline (lessons from Stones A-E1 + Phase 1/2 cleanup + E-1's 9-ward pass)

1. **Module-level doc update** (Stone B gaze L1 lesson) — replace "Current scope (through Stone E-1)" with "(through Stone E-2)" naming the persistent Select ring + Receiver method extraction.
2. **Audience section retires Stone E-2 entry** — the line reading "Stone D's `Select`, Stone E-2's reflexive-rebuild ring persistification, Slice 4's kernel dispatcher" loses its Stone E-2 mention since E-2 IS this work.
3. **SQE counts honest** — Stone E-2 doesn't change Receiver's ring; the struct doc + field doc SQE counts (corrected in E-1 ward pass) stay accurate.
4. **Doc comments on every new method** — `Receiver::read_into_acc` + `Receiver::take_buffered_frame` need doc comments naming their solvere-closure rationale.
5. **`unsafe` blocks carry SAFETY comments** — preserved from D2; the borrow scoping changes but the fd lifetime invariants are unchanged.
6. **RefCell borrow discipline** — Select::select scopes the Select-ring borrow inside `{ ... }` blocks so it releases BEFORE Receiver method calls (which borrow the Receiver's separate ring). Pattern matches Stone E-1's borrow discipline.
7. **NO PANIC PATHS on the ring borrow** — RefCell borrow_mut() can panic on double-borrow. The substrate's threading model never shares a Select across threads. Within Select::select, no recursive Select-ring use exists.
8. **Receiver method visibility** — `pub(crate)` per the substrate-internal-only audience; matches Receiver::recv/try_recv visibility pattern (those are `pub` because they're the user-facing API; the new methods are internal helpers).
9. **The rune at line 742 retires entirely** — the surrounding comment about "Stone E-2 (task #394) will persistify" was forward documentation that NOW points at COMPLETED work. The comment retires alongside the rune.
10. **NO new tests** — E-2 is mechanically invisible; existing tests prove correctness via behavior preservation. 34/34 ratio preserved.

## Concrete deliverables list

1. **Edit** `src/comms/process.rs` —
   - Module-level doc updated (through Stone E-2)
   - Audience section trimmed (Stone E-2 retires from the "future" list)
   - `Receiver::read_into_acc(&self) -> Result<usize, ()>` + `Receiver::take_buffered_frame(&self) -> Option<Vec<u8>>` minted (pub(crate))
   - `Receiver::recv` + `Receiver::try_recv` refactored to call the new methods
   - `Select<'a, T>` struct gains `ring: RefCell<Option<(IoUring, u32)>>` field
   - `Select::new()` initializes `ring: RefCell::new(None)`
   - `Select::select` refactored: fast-path uses `rx.take_buffered_frame()`; reflexive rebuild at top of loop; inner loop borrows persistent ring; Read step uses `rx.read_into_acc()`; partial-frame check uses `rx.take_buffered_frame()`
   - `rune:temperare(no-reactor)` at line ~742 + surrounding doc-comment retires

2. **New file** SCORE doc: `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-3E2-SELECT-PERSISTENT-RING.md`

Estimated LOC: ~80-130 net delta (additions: 2 new methods + new struct field + reflexive rebuild block + 2 method-call site changes in recv/try_recv + 3 method-call site changes in Select::select; deletions: per-call IoUring::new in Select::select loop + its rune + surrounding comment + 3 field-access sites in Select::select replaced with method calls).

## Critical constraints

- **DO NOT commit.** Orchestrator commits after SCORE verification + ward pass.
- **Anchor cwd:** `/home/watmin/work/holon/wat-rs/`
- **Use `git -C`** for git ops

## Cross-references

- DESIGN.md § "Stone E forward-correction (2026-05-19) — TCO discipline + reflexive rebuild" — the architectural reframe E-2 ships
- BRIEF-214-SLICE-3E1-RECEIVER-PERSISTENT-RING.md + SCORE + WARD-PASS — Stone E-1 (just-shipped pre-E-2)
- WARD-PASS-3E1-RECEIVER-PERSISTENT-RING.md § "Deferred to Stone E-2 (explicit inscription)" — the solvere finding closure plan E-2 executes
- BRIEF-214-SLICE-3D2-SELECT.md — Stone D2 (Select N+1-arm fan-in; E-2 persistifies its ring)
- `feedback_iterative_complexity` — Stone E bundled as E-1 + E-2 per four-questions
- `feedback_substrate_owns_not_callers_match` — Receiver method extraction codifies this at the Receiver/Select boundary
