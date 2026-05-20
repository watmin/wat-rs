# Arc 214 Slice 3 — Stone E-2 — SCORE

**Stone:** Select persistent ring (reflexive rebuild) + Receiver method extraction
**Date:** 2026-05-19
**Implementor:** claude-sonnet-4-6 (arc 214 Slice 3 Stone E-2 agent)
**Mode:** A (all 42 criteria satisfied)

## Build + test verification

```
cargo build --release      CLEAN (5 pre-existing dead_code warnings; 0 in comms)
cargo test --release --test comms                     34/34 PASS
cargo test --release --test probe_channel_primitive    3/3  PASS
cargo test --release --test probe_pidfd_primitive      2/2  PASS
```

## LOC delta

- `src/comms/process.rs`: +167 insertions, -101 deletions (66 net)
- New file: `SCORE-214-SLICE-3E2-SELECT-PERSISTENT-RING.md` (this document)
- Zero other files touched

## Evidence greps (pre-verification)

```
grep -n "rune:temperare" src/comms/process.rs → 0 hits (fully retired)
grep -n "RefCell<Option<(IoUring" src/comms/process.rs → 1 hit (line 716, struct field)
grep -n "pub(crate) fn read_into_acc" src/comms/process.rs → 1 hit (line 405)
grep -n "pub(crate) fn take_buffered_frame" src/comms/process.rs → 1 hit (line 418)
grep -n "IoUring::new(needed_capacity)" src/comms/process.rs → 1 hit (reflexive rebuild, line 793)
grep -c "take_buffered_frame" src/comms/process.rs → 8 (method def + doc refs + 5 call sites)
grep -c "read_into_acc" src/comms/process.rs → 6 (method def + doc refs + 3 call sites)
```

## 42-row scorecard

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Module-level doc: "(through Stone E-1)" → "(through Stone E-2)" naming persistent Select ring + Receiver method extraction | PASS | `grep "Current scope (through Stone E-2)" process.rs` → 1 hit (line 17); names both new capabilities |
| 2 | Audience section: Stone E-2 entry removed from the "future" list | PASS | "Slice 4's kernel dispatcher" — no Stone E-2 mention; `grep "E-2.*reflexive" process.rs` → 0 hits in audience section |
| 3 | `Receiver::read_into_acc(&self) -> Result<usize, ()>` minted as `pub(crate)` method with doc comment | PASS | line 405: `pub(crate) fn read_into_acc(&self) -> Result<usize, ()>`; doc comment at lines 395-407 |
| 4 | `Receiver::take_buffered_frame(&self) -> Option<Vec<u8>>` minted as `pub(crate)` method with doc comment | PASS | line 418: `pub(crate) fn take_buffered_frame(&self) -> Option<Vec<u8>>`; doc comment at lines 409-420 |
| 5 | `Receiver::recv` fast-path uses `self.take_buffered_frame()` (2 sites) instead of `take_frame(...)` | PASS | lines 279, 309: `self.take_buffered_frame()` at both fast-path sites |
| 6 | `Receiver::recv` Read step uses `self.read_into_acc()` instead of `uring_read_into_acc(...)` | PASS | line 303: `let n = self.read_into_acc().map_err(\|_\| RecvError)?;` |
| 7 | `Receiver::try_recv` fast-path uses `self.take_buffered_frame()` instead of `take_frame(...)` | PASS | line 337: `if let Some(frame) = self.take_buffered_frame()` |
| 8 | `Receiver::try_recv` Read step uses `self.read_into_acc()` instead of `uring_read_into_acc(...)` | PASS | line 381: `let n = self.read_into_acc().map_err(\|_\| TryRecvError::Disconnected)?;` |
| 9 | `Select<'a, T>` struct gains `ring: RefCell<Option<(IoUring, u32)>>` field with doc comment | PASS | line 716: `ring: RefCell<Option<(IoUring, u32)>>,`; doc comment at lines 707-716 |
| 10 | `Select::new()` initializes `ring: RefCell::new(None)` | PASS | line 732: `ring: RefCell::new(None),` |
| 11 | `Select::select` fast-path uses `rx.take_buffered_frame()` instead of `take_frame(...)` | PASS | line 761: `if let Some(frame) = rx.take_buffered_frame()` |
| 12 | `Select::select` reflexive rebuild block at loop top: pattern-match `ring_slot.as_ref()`; rebuild on None OR capacity mismatch; assign `Some((r, needed_capacity))` | PASS | lines 786-798: match on `ring_slot.as_ref()` with None → true, Some((_, current_cap)) → `*current_cap != needed_capacity`; assign `Some((r, needed_capacity))` |
| 13 | `Select::select` rebuild failure path: `return SelectOutcome::SubstrateError(e)` (not unwrap/expect) | PASS | line 795: `Err(e) => return SelectOutcome::SubstrateError(e),` |
| 14 | `Select::select` inner submission block borrows `self.ring` via `borrow_mut()`; scope releases before Read step | PASS | lines 807-878: explicit `{ let mut ring_slot = self.ring.borrow_mut(); ... }` block; Read step at line 895 is outside |
| 15 | `Select::select` submission block: POLL_ADDs pushed through the persistent ring (not a fresh IoUring) | PASS | `ring_slot.as_mut().unwrap().0` — the persistent ring from `self.ring` |
| 16 | `Select::select` Read step uses `rx.read_into_acc()` (not free function `uring_read_into_acc(...)`) | PASS | line 895: `match rx.read_into_acc()` |
| 17 | `Select::select` partial-frame post-Read check uses `rx.take_buffered_frame()` (not free function `take_frame(...)`) | PASS | line 912: `if let Some(frame) = rx.take_buffered_frame()` |
| 18 | `rune:temperare(no-reactor)` at line ~742 RETIRED + surrounding doc-comment retired | PASS | `grep "rune:temperare" process.rs` → 0 hits; entire rune block + surrounding doc about "pre-E-2 placeholder" gone |
| 19 | Per-call `IoUring::new(ring_capacity)` inside `Select::select`'s loop ELIMINATED | PASS | `IoUring::new(needed_capacity)` only appears inside the reflexive rebuild guard (`if needs_rebuild`); no unconditional per-call construction |
| 20 | `cargo build --release` clean | PASS | 0 errors; 5 pre-existing dead_code warnings in check.rs/runtime.rs; 0 in comms |
| 21 | `cargo test --release --test comms` 34/34 PASS (zero net delta from E-1) | PASS | 34 passed; 0 failed |
| 22 | `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged | PASS | 3 passed; 0 failed |
| 23 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged | PASS | 2 passed; 0 failed |
| 24 | NO new probe tests added | PASS | test file count unchanged; `grep -r "probe_slice3e2" tests/` → 0 hits |
| 25 | Stone A helpers (`take_frame`) UNCHANGED (still called by Receiver::take_buffered_frame internally) | PASS | `take_frame` body at lines 606-618 unchanged; Receiver::take_buffered_frame wraps it |
| 26 | Stone B `PollOutcome` + `wait_for_data_or_cascade` UNCHANGED (still called by Receiver::recv) | PASS | `wait_for_data_or_cascade` body unchanged; Receiver::recv still calls it at line 293 |
| 27 | Stone C `decode_frame` + `Sender::send` + Sender side of `pair<T>()` UNCHANGED | PASS | Sender block lines 96-207 untouched; decode_frame unchanged |
| 28 | Stone D1 close/len/CommSender/CommReceiver trait impls UNCHANGED | PASS | len/close/CommSender/CommReceiver impls unchanged |
| 29 | Stone D2 `Select::new` / `Select::recv` UNCHANGED in signature (Select::new adds 1 line for ring init; Select::recv body unchanged) | PASS | Select::recv body identical; Select::new adds only `ring: RefCell::new(None)` |
| 30 | Stone E-1 Receiver field set (read_fd / accumulator / ring / _phantom) UNCHANGED — methods ADDED only | PASS | Receiver struct definition unchanged; only new methods added in impl block |
| 31 | Stone E-1 Receiver::clone UNCHANGED | PASS | Clone impl at lines 452-486 unchanged |
| 32 | Stone E-1 `pair<T>()` factory UNCHANGED | PASS | pair() body at lines 942-973 unchanged |
| 33 | Stone E-1 free function `uring_read_into_acc` UNCHANGED (still called internally by Receiver::read_into_acc) | PASS | `uring_read_into_acc` body at lines 646-666 unchanged; Receiver::read_into_acc wraps it |
| 34 | Free function `take_frame` UNCHANGED (still called internally by Receiver::take_buffered_frame) | PASS | `take_frame` body at lines 606-618 unchanged |
| 35 | NO config tunable code added (no `set-process-tier-uring-depth!` setter, atomic, validation) | PASS | `grep "uring-depth\|set-process-tier" src/comms/process.rs` → 0 hits |
| 36 | NO `wat_arc170_program_contracts` re-run | PASS | Not run |
| 37 | Dirty tree (src/fork.rs + src/spawn_process.rs + arc 213 δ-2 docs) UNTOUCHED | PASS | Only `src/comms/process.rs` + this SCORE doc modified |
| 38 | `src/typed_channel.rs`, `src/edn_shim.rs`, `src/comms/mod.rs`, `src/comms/thread.rs`, `Cargo.toml` UNTOUCHED | PASS | Zero modifications to these files |
| 39 | Zero modifications outside `src/comms/process.rs` + SCORE doc | PASS | `git diff --name-only` → `src/comms/process.rs` only (source code); SCORE doc new file |
| 40 | SAFETY comments on all unsafe blocks PRESERVED + still honest (lifetimes still hold despite borrow rescoping) | PASS | All unsafe blocks in Select::select carry existing SAFETY comments; broadcast_fd + rx.read_fd lifetimes unchanged |
| 41 | Every new public/pub(crate) item has a doc comment | PASS | `read_into_acc` (lines 395-407) and `take_buffered_frame` (lines 409-420) both have full doc comments; `Select.ring` field has doc comment (lines 707-716) |
| 42 | NO commit (orchestrator commits after verify + ward pass) | PASS | No commit issued |

## Honest deltas

**No surprises.** Stone E-2 executed exactly per the BRIEF skeleton:

1. **Borrow scoping (Risk 1) — CLEAN.** The two explicit `{ ... }` scope blocks in Select::select cleanly separated the reflexive rebuild borrow, the submission borrow, and the Read step (which calls `rx.read_into_acc()` on the Receiver's different RefCell). No compile-time borrow errors; the discipline held.

2. **Reflexive rebuild logic (Risk 2) — CLEAN.** Pattern-match on `ring_slot.as_ref()` with explicit None and capacity-mismatch arms worked exactly as the skeleton showed. The `needs_rebuild` variable makes the decision visible.

3. **SAFETY-of-unwrap comment (Risk 4) — inscribed.** The comment "SAFETY of unwrap: reflexive rebuild above guarantees Some(_)" at line 809 is present and honest.

4. **LOC delta — within estimate.** Net +66 lines vs. the estimated 80-130 range. The Select::select body became more explicit in structure (explicit scope blocks, unwrap justification comments) but the Receiver refactors in recv/try_recv were clean one-liners that saved lines.

5. **`Err(_) | Ok(0)` → split arms.** The BRIEF skeleton showed the Err and Ok(0) cases as separate match arms for clarity. The implementation follows that split (Err arm + Ok(0) arm separately) rather than the combined form from the pre-E-2 code. Behavior-identical; the split form teaches each case independently.

6. **doc comment on struct-level `Select` (before struct definition).** Updated the existing doc-comment on the struct to reference Stone E-2 (removed "Per-call IoUring sized for N+1 POLL_ADD entries (Stone E persistifies)" and replaced with the E-2 statement). No BRIEF entry explicitly listed this but it was the honest thing to do per ward-discipline for doc accuracy.

## Cross-references

- BRIEF-214-SLICE-3E2-SELECT-PERSISTENT-RING.md — work order
- EXPECTATIONS-214-SLICE-3E2-SELECT-PERSISTENT-RING.md — 42-row prediction (all satisfied; Mode A)
- DESIGN.md § "Stone E forward-correction (2026-05-19) — TCO discipline + reflexive rebuild"
- WARD-PASS-3E1-RECEIVER-PERSISTENT-RING.md § "Deferred to Stone E-2" — solvere's plan executed
- SCORE-214-SLICE-3E1-RECEIVER-PERSISTENT-RING.md — prior stone's Mode A delivery
