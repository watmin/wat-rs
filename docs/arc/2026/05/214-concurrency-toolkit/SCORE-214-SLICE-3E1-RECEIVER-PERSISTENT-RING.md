# Arc 214 Slice 3 — Stone E-1 — SCORE
# Receiver persistent ring (capacity 4)

**Date:** 2026-05-19
**Agent:** claude-sonnet-4-6

## Result

Mode A — all 36 criteria satisfied.

## Build output

```
cargo build --release
Finished `release` profile [optimized] target(s) in 18.71s
```

Clean. 5 pre-existing dead_code warnings in check.rs / runtime.rs — unrelated to comms.

## Test results

```
cargo test --release --test comms
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

cargo test --release --test probe_channel_primitive
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

cargo test --release --test probe_pidfd_primitive
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## LOC delta

`src/comms/process.rs`: 87 insertions, 50 deletions → net +37 lines.
Total: 874 lines (was 838).

## Scorecard (36 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Module-level doc: "(through Stone D2)" → "(through Stone E-1)" naming Receiver persistent ring | PASS | grep -n "through Stone E-1" → line 17 |
| 2 | `Receiver<T>` struct gains `ring: RefCell<IoUring>` field with doc comment naming Stone E-1 | PASS | lines 211-217; field doc says "Persistent io_uring (Stone E-1)" |
| 3 | `uring_read_into_acc` signature: `(fd, acc, ring: &RefCell<IoUring>)` | PASS | lines 579-583 |
| 4 | `uring_read_into_acc` body: `let mut ring = ring.borrow_mut();` at top; no `IoUring::new(2)` | PASS | line 584; grep for `IoUring::new(2)` → 0 hits |
| 5 | `uring_read_into_acc` rune:temperare(no-reactor) doc-comment DELETED | PASS | grep for rune:temperare → 1 hit (Select POLL_ADD only) |
| 6 | `wait_for_data_or_cascade` signature: `(read_fd, broadcast_fd, ring: &RefCell<IoUring>)` | PASS | lines 460-464 |
| 7 | `wait_for_data_or_cascade` body: `let mut ring = ring.borrow_mut();` at top; no `IoUring::new(4)` in helper | PASS | line 468; no per-call IoUring::new in the helper |
| 8 | `wait_for_data_or_cascade` rune:temperare(no-reactor) doc-comment DELETED | PASS | 1 rune:temperare hit total (Select only) |
| 9 | `Receiver::recv` passes `&self.ring` to wait_for_data_or_cascade | PASS | line 253: `wait_for_data_or_cascade(read_fd, broadcast_fd, &self.ring)?` |
| 10 | `Receiver::recv` passes `&self.ring` to uring_read_into_acc | PASS | line 262: `uring_read_into_acc(read_fd, &self.accumulator, &self.ring)` |
| 11 | `Receiver::recv` doc-comment rune-reference lines removed | PASS | old "rune:temperare(no-reactor) for rationale" comment replaced with "uses the Receiver's persistent ring (Stone E-1)" |
| 12 | `Receiver::try_recv` passes `&self.ring` to uring_read_into_acc | PASS | lines 340-341: `uring_read_into_acc(read_fd, &self.accumulator, &self.ring)` |
| 13 | `Receiver::clone` constructs fresh `IoUring::new(4)` via `.expect(...)`; clones get independent rings | PASS | lines 410-413 |
| 14 | `Receiver::clone` doc comment updated to name fresh ring + `!Sync` rationale | PASS | lines 396-402 |
| 15 | `pair<T>()` factory constructs ring via `IoUring::new(4)?` (or .map_err) and stores in Receiver | PASS | lines 843-847: `.map_err(|e| ...)` wrapping |
| 16 | `Select::select`'s Read step at ~line 792 passes `&rx.ring` as third arg | PASS | line 792: `uring_read_into_acc(rx.read_fd.as_raw_fd(), &rx.accumulator, &rx.ring)` |
| 17 | Select's per-call POLL_ADD ring (~line 702) UNCHANGED — rune:temperare(no-reactor) PRESERVED | PASS | line 702 has rune:temperare preserved; `let mut ring = match IoUring::new(ring_capacity)` unchanged |
| 18 | `cargo build --release` clean | PASS | 0 errors; 5 pre-existing dead_code warnings |
| 19 | `cargo test --release --test comms` 34/34 PASS | PASS | 34 passed; 0 failed |
| 20 | `cargo test --release --test probe_channel_primitive` 3/3 PASS | PASS | 3 passed; 0 failed |
| 21 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS | PASS | 2 passed; 0 failed |
| 22 | NO new probe tests added | PASS | test count unchanged: 34 in comms, 3 in probe_channel_primitive, 2 in probe_pidfd_primitive |
| 23 | Stone A helpers (take_frame) UNCHANGED | PASS | take_frame untouched; same body and signature |
| 24 | Stone B PollOutcome enum UNCHANGED | PASS | enum variants DataReady / Shutdown unchanged |
| 25 | Stone C `decode_frame` / `Sender::send` UNCHANGED | PASS | both functions unchanged |
| 26 | Stone D1 methods (close/len) + trait impls UNCHANGED; try_recv body only gains `&self.ring` arg | PASS | len / close / CommReceiver impl unchanged; try_recv change is the single `&self.ring` arg |
| 27 | Stone D2 `Select::new` / `Select::recv` UNCHANGED; `Select::select` only the Read-step call site changes | PASS | Select::new and Select::recv untouched; only line 792 (Read step) changed |
| 28 | Sender<T> struct UNCHANGED | PASS | Sender struct and all its impls untouched |
| 29 | NO config tunable code added | PASS | grep for "set-process-tier-uring-depth" → 0 hits; no atomics added |
| 30 | NO `wat_arc170_program_contracts` re-run | PASS | not run |
| 31 | Dirty tree (src/fork.rs + src/spawn_process.rs) UNTOUCHED | PASS | git diff shows only src/comms/process.rs changed |
| 32 | `src/typed_channel.rs`, `src/edn_shim.rs`, `src/comms/mod.rs`, `src/comms/thread.rs`, `Cargo.toml` UNTOUCHED | PASS | git diff confirms no changes to these files |
| 33 | Zero modifications outside `src/comms/process.rs` + SCORE doc | PASS | git diff --stat shows only process.rs modified |
| 34 | SAFETY comments on all unsafe blocks PRESERVED + still honest | PASS | both SAFETY comments in helpers retained verbatim; lifetimes still hold (ring borrow_mut held for submit_and_wait duration) |
| 35 | Every modified item has updated doc comment | PASS | Receiver struct, ring field, uring_read_into_acc, wait_for_data_or_cascade, Receiver::clone, Receiver::recv read-step comment, Receiver::try_recv read-step comment all updated |
| 36 | NO commit (orchestrator commits after verify + ward pass) | PASS | no commit made |

## Honest deltas

### Surprise 1 — IoUring does not implement Debug (Risk 2 variant)

EXPECTATIONS Risk 2 predicted "IoUring import missing" as the compile risk. The actual compile error was different: `IoUring` does not implement the `Debug` trait, so `#[derive(Debug)]` on `Receiver<T>` failed.

**Resolution:** Removed `#[derive(Debug)]` from `Receiver<T>`. Added a manual `impl<T: HolonRepresentable> std::fmt::Debug for Receiver<T>` that renders the ring field as the opaque string `"IoUring"` (all other fields use their own Debug impls). This is honest: it surfaces what the Receiver contains without fabricating a ring representation.

**LOC impact:** +13 lines (manual Debug impl). Total net delta is +37 lines vs predicted 30-60 net. Within range.

**Ward note for gaze:** the manual Debug impl is a new structural item on Receiver not mentioned in the BRIEF. It is mechanical, honest, and unavoidable — IoUring's upstream crate does not expose Debug. The alternative (dropping Debug from Receiver entirely) would break any caller using `{:?}` on a Receiver; preserving Debug is the correct choice.

### No other surprises

- RefCell borrow ergonomics (Risk 1): clean — `let mut ring = ring.borrow_mut();` at top of each helper; released at function return.
- pair() error handling (Risk 3): `.map_err` pattern worked as specified; `?` propagates correctly into `std::io::Result`.
- Clone panic path (Risk 4): `.expect(...)` pattern matched BRIEF exactly.
- Select Read-step delegation (Risk 5): `&rx.ring` correctly used; Select's per-call POLL_ADD ring untouched.
- Rune deletion vs preservation (Risk 6): 2 rune:temperare comments deleted (helpers); 1 preserved (Select POLL_ADD at line 702).
- Test count (Risk 7): 34/34 unchanged; no new tests added.
- Doc comment scope (Risk 8): only the named items updated; Sender and all Stone A-C items untouched.
- Stones A-D2 preservation (Risk 9): all behavior-preserving edits confirmed via 34/34 test pass.
