# Arc 214 Slice 3 — Stone E-1 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 15-25 min Mode A. Smaller than D2 because E-1 is purely mechanical refactor (no new structs; no new tests; same surface; same semantics). The diff is parameter additions + field addition + factory + Clone updates.
- **LOC changed:** ~30-60 net delta (additions: ring field, Clone ring construction, pair() ring construction, three helper-signature parameter additions; deletions: two `IoUring::new(...)` lines, rune comments).
- **New files:** 1 (SCORE doc only).
- **Surprises expected:** LOW-MEDIUM. The refactor is mechanical; the risk is RefCell<IoUring> ergonomics + borrow-discipline at the helper signatures.

## Honest-delta watch

### Risk 1 — RefCell<IoUring> borrow_mut() ergonomics

**What:** Helpers operate on `&RefCell<IoUring>` and call `.borrow_mut()` inside. If sonnet borrows the ring outside the helper (e.g., at the caller) and passes `&mut IoUring`, the borrow is held longer than needed. Or if sonnet drops `let mut ring = ring.borrow_mut();` at the top of the helper body, the borrow is held for the whole submit+wait+drain cycle (correct; matches the BRIEF skeleton). The exact pattern matters for not-touching-the-receiver-during-the-op.

**Mitigation:** BRIEF skeleton spells out `let mut ring = ring.borrow_mut();` at the top of each refactored helper. Borrow held for op duration; dropped on return.

### Risk 2 — IoUring imports

**What:** `IoUring` is imported in process.rs at the top. The Receiver struct adds `ring: RefCell<IoUring>`. If sonnet forgets to import `IoUring` into the appropriate context, compile error.

**Mitigation:** Verify import is at module-level (it already is — `use io_uring::{opcode, types, IoUring}` at top of process.rs).

### Risk 3 — pair<T>() factory error handling

**What:** `IoUring::new(4)` returns `io::Result<IoUring>`. `pair<T>()` already returns `std::io::Result<(Sender<T>, Receiver<T>)>`. The new construction must propagate the error. The BRIEF skeleton uses `.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!(...)))`. If sonnet uses `.unwrap()` or `?` on an incompatible Result, compile error or panic-on-allocation-failure.

**Mitigation:** BRIEF skeleton spells out the error wrapping; the `?` works if `IoUring::new` returns `std::io::Error` directly (verify: io-uring 0.7's `IoUring::new` returns `std::io::Result<IoUring>`, so `?` works AND keeps `pair<T>()`'s signature unchanged).

### Risk 4 — Receiver::clone panic path

**What:** Clone panics on `OwnedFd::try_clone` failure today. BRIEF adds a SECOND panic path on `IoUring::new(4)` failure. If sonnet uses `?` instead of `.expect(...)`, the Clone signature would need to change (currently `fn clone(&self) -> Self`; would need `Result<Self, _>` which breaks the Clone trait).

**Mitigation:** BRIEF uses `.expect("IoUring::new(4) failed — kernel io_uring resource exhausted")` matching the existing pattern.

### Risk 5 — Select::select Read-step ring delegation

**What:** Stone D2's `Select::select` calls `uring_read_into_acc(rx.read_fd.as_raw_fd(), &rx.accumulator)`. E-1 adds `&rx.ring`. Select itself doesn't have a persistent ring yet (E-2 territory); its POLL_ADD step still uses per-call `IoUring::new(ring_capacity)`. Risk: sonnet inadvertently uses Select's per-call ring for the Read step (sharing ring across two concurrent operations is problematic in io_uring).

**Mitigation:** BRIEF explicitly says "Select uses the FIRED RECEIVER's ring for the Read step (rx.ring), NOT Select's own ring."

### Risk 6 — Rune deletion vs preservation

**What:** Two `rune:temperare(no-reactor)` doc-comments on the refactored helpers MUST be deleted (helpers no longer construct per-call rings; rune is no longer applicable). The Select rune at `src/comms/process.rs:688` MUST be preserved (Select still constructs per-call ring until E-2). If sonnet deletes ALL `rune:temperare(no-reactor)` references including Select's, the Select rune that names the remaining heat is lost.

**Mitigation:** BRIEF explicitly lists 2 rune deletions (helpers) and preserves Select's rune.

### Risk 7 — Test count preservation

**What:** All 34 existing tests must continue to pass. No new tests are added (E-1 is mechanically invisible). If sonnet adds a probe test out of habit, test count grows; ward may flag.

**Mitigation:** BRIEF explicitly says "NO new tests."

### Risk 8 — Doc comment scope creep

**What:** Doc comments need updates on Receiver struct + ring field + refactored helpers. If sonnet rewrites doc comments on UNTOUCHED items (e.g., Sender), gaze may flag inconsistency or scope drift.

**Mitigation:** BRIEF lists exactly which doc comments to update.

### Risk 9 — Preserving Stones A-D2 unchanged

**What:** E-1 is purely additive to Receiver's state model + helper signatures + Select's Read-step call site. ALL other behavior must be preserved exactly.

**Mitigation:** BRIEF's STOP triggers list every helper / method as UNCHANGED except the named ones.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | Module-level doc: "(through Stone D2)" → "(through Stone E-1)" naming Receiver persistent ring | YES |
| 2 | `Receiver<T>` struct gains `ring: RefCell<IoUring>` field with doc comment naming Stone E-1 | YES |
| 3 | `uring_read_into_acc` signature: `(fd, acc, ring: &RefCell<IoUring>)` | YES |
| 4 | `uring_read_into_acc` body: `let mut ring = ring.borrow_mut();` at top; no `IoUring::new(2)` | YES |
| 5 | `uring_read_into_acc` rune:temperare(no-reactor) doc-comment DELETED | YES |
| 6 | `wait_for_data_or_cascade` signature: `(read_fd, broadcast_fd, ring: &RefCell<IoUring>)` | YES |
| 7 | `wait_for_data_or_cascade` body: `let mut ring = ring.borrow_mut();` at top; no `IoUring::new(4)` | YES |
| 8 | `wait_for_data_or_cascade` rune:temperare(no-reactor) doc-comment DELETED | YES |
| 9 | `Receiver::recv` passes `&self.ring` to wait_for_data_or_cascade | YES |
| 10 | `Receiver::recv` passes `&self.ring` to uring_read_into_acc | YES |
| 11 | `Receiver::recv` doc-comment rune-reference lines removed | YES |
| 12 | `Receiver::try_recv` passes `&self.ring` to uring_read_into_acc | YES |
| 13 | `Receiver::clone` constructs fresh `IoUring::new(4)` via `.expect(...)`; clones get independent rings | YES |
| 14 | `Receiver::clone` doc comment updated to name fresh ring + `!Sync` rationale | YES |
| 15 | `pair<T>()` factory constructs ring via `IoUring::new(4)?` (or .map_err) and stores in Receiver | YES |
| 16 | `Select::select`'s Read step at line 778 passes `&rx.ring` as third arg | YES |
| 17 | Select's per-call POLL_ADD ring (line 689) UNCHANGED — rune:temperare(no-reactor) PRESERVED | YES |
| 18 | `cargo build --release` clean | YES |
| 19 | `cargo test --release --test comms` 34/34 PASS (zero net delta from Phase 2) | YES |
| 20 | `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged | YES |
| 21 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged | YES |
| 22 | NO new probe tests added | YES |
| 23 | Stone A helpers (take_frame) UNCHANGED | YES |
| 24 | Stone B PollOutcome enum UNCHANGED | YES |
| 25 | Stone C `decode_frame` / `Sender::send` UNCHANGED | YES |
| 26 | Stone D1 methods (close/len/try_recv body except `&self.ring` arg) + trait impls UNCHANGED | YES |
| 27 | Stone D2 `Select::new` / `Select::recv` UNCHANGED; `Select::select` only the Read-step call site changes | YES |
| 28 | Sender<T> struct UNCHANGED | YES |
| 29 | NO config tunable code added (`set-process-tier-uring-depth!` setter, atomic, validation) | YES |
| 30 | NO `wat_arc170_program_contracts` re-run | YES |
| 31 | Dirty tree (src/fork.rs + src/spawn_process.rs + arc 213 δ-2 docs) UNTOUCHED | YES |
| 32 | `src/typed_channel.rs`, `src/edn_shim.rs`, `src/comms/mod.rs`, `src/comms/thread.rs`, `Cargo.toml` UNTOUCHED | YES |
| 33 | Zero modifications outside `src/comms/process.rs` + SCORE doc | YES |
| 34 | SAFETY comments on all unsafe blocks PRESERVED + still honest (lifetimes still hold) | YES |
| 35 | Every modified item has updated doc comment | YES |
| 36 | NO commit (orchestrator commits after verify + ward pass) | YES |

## Mode classification

- **Mode A:** all 36 criteria satisfied
- **Mode B (acceptable):**
  - Risk 1 fires (borrow scope wrong): one test hangs or panics; sonnet investigates
  - Risk 3 fires (factory error handling wrong): cargo build error; sonnet adjusts
  - Risk 5 fires (Select uses wrong ring): one Select test hangs; sonnet investigates
  - One probe test fails: sonnet investigates
- **Mode C (failure):**
  - Touched any file outside `src/comms/process.rs` + SCORE doc
  - Added a config tunable
  - Added new probe tests
  - Modified Sender / Select::new / Select::recv / Stone A-C helpers
  - Persisted Select's POLL_ADD ring (E-2 territory)

## Calibration metadata

- **Orchestrator confidence:** HIGH. E-1 is mechanical refactor; the BRIEF skeleton is exhaustive; risk is in RefCell-borrow ergonomics and Select-Read-step delegation, both spelled out. The Phase 1/2 vigilia cleanup established the helper boundaries E-1 is refactoring.
- **Risk factors:** RefCell borrow scope; rune deletion vs preservation; Select's two-ring nature (per-call POLL_ADD ring stays; Read step delegates to Receiver's ring).

## Ward pass prediction

- gaze: 0-1 (Receiver struct has 4 fields now; ring field doc-comment must be clear; ring naming acceptable since rings ARE the io_uring concept)
- forge: 1-2 (RefCell<IoUring> for `&self` interior mutability — same shape as RefCell<Vec<u8>> precedent; possible mumble on whether ring borrow ergonomics surface a place-where-value-belongs concern; Clone's `.expect(...)` pattern preserved from try_clone precedent)
- reap: 0 (rune deletions are reap-equivalent; no other dead code)
- sever: 0-1 (helper refactor cleanly preserves separation; Receiver owns ring; helpers operate on borrowed ring; no concerns braided)
- temper: 0-1 (Stone E-1 closes the per-call IoUring temperare runes; Select still has its rune; honest)
- cleave: N/A (no new parallel code; existing disjoint boundaries preserved)
- scry: N/A (no wat-level surface changes)
- ignorant: N/A (Stone E-1 is implementation; INSCRIPTION not at this stone)

Total: 1-5 findings. Round 2 unlikely — purely mechanical work.

## Cross-references

- BRIEF-214-SLICE-3E1-RECEIVER-PERSISTENT-RING.md — this stone's work order
- DESIGN.md § "Stone E forward-correction (2026-05-19) — TCO discipline + reflexive rebuild" — the architectural reframe E-1 implements
- BRIEF-214-SLICE-3D2-SELECT.md — Stone D2 (last shipped pre-E-1)
- BRIEF-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md — Stone B (the 2-arm POLL_ADD pattern E-1 keeps; refactored to use Receiver's ring)
- SCORE-COMMS-CLEANUP-PHASE-1.md + SCORE-COMMS-CLEANUP-PHASE-2.md — vigilia cleanup that established the helper boundaries
- `feedback_iterative_complexity` — Stone E split into E-1 + E-2 per four-questions
