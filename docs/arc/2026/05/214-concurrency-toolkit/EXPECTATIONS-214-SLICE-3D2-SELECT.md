# Arc 214 Slice 3 — Stone D2 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 20-30 min Mode A. Smaller than D1 because Stone D2 is focused on ONE new struct (Select<'a, T>) + 2 new tests. The implementation is novel substrate work (N+1-arm POLL_ADD) but the pattern generalizes Stone B's 2-arm case, which is well-established.
- **LOC changed:** ~220-270 total (~150-180 in `src/comms/process.rs`; ~70-90 in `tests/comms/process.rs`).
- **New files:** 1 (SCORE doc only).
- **Surprises expected:** MEDIUM. The io_uring N+1-arm orchestration has multiple potential micro-bugs (ring sizing, user_data token scheme, CQE drain at scale, bail-out path semantics).

## Honest-delta watch

### Risk 1 — Ring capacity sizing

**What:** Select's per-call `IoUring::new(ring_capacity)` needs enough SQE slots for N data arm POLL_ADDs + 1 broadcast (when initialized). The BRIEF computes `((arm_count.max(1)).next_power_of_two() as u32).max(2)`. If sonnet drops the `.max(1)` (when arm_count is 0, next_power_of_two on 0 is undefined/panics), or drops the `.max(2)` floor (io-uring crate may reject 1-entry rings), submission fails.

**Mitigation:** BRIEF spells out the exact formula. Sonnet copies.

### Risk 2 — user_data token scheme

**What:** Tokens: 0 = BROADCAST_TOKEN; 1..=N = data arms. Arm index = token - 1. If sonnet uses 0-indexed tokens for data arms (collides with broadcast), the dispatch logic breaks. If sonnet uses 1-indexed user_arm tokens but doesn't subtract 1 when computing arm_idx, the index is off by one.

**Mitigation:** BRIEF skeleton spells out `user_data((i + 1) as u64)` for push and `arm = (token - 1) as usize` for dispatch.

### Risk 3 — Defensive empty-CQE drain return

**What:** `submit_and_wait(1)` waits for ≥1 CQE. If somehow no CQE drains (shouldn't happen with min_complete=1), the loop body's `first_data_arm` is None and `fired_broadcast` is false. The BRIEF's `continue` retries the loop. If sonnet returns a synthetic Err here instead, callers see spurious failures.

**Mitigation:** BRIEF uses `continue` in the defensive branch.

### Risk 4 — Read step after data arm fires

**What:** When data arm fires, Select does a SECOND io_uring (per-call IoUring::new(2)) for the Read SQE. This is the same pattern as Stone B/C's recv slow-path. If sonnet inlines without the separate Read ring, or merges into the first ring (which already has POLL_ADD SQEs), the submission queue may overflow or behave unexpectedly.

**Mitigation:** BRIEF skeleton uses a SEPARATE `read_ring` for the Read step. Explicit.

### Risk 5 — Lifetime correctness for Select<'a, T>

**What:** Select holds `Vec<&'a Receiver<T>>`. The lifetime 'a tracks the registered receivers' borrow. If sonnet's signatures drop 'a or use 'static, callers fail to compile or the borrow checker rejects.

**Mitigation:** BRIEF signatures spell out 'a explicitly throughout.

### Risk 6 — Bail-out synthetic ReceiverIndex(0)

**What:** On io_uring substrate failures (ring creation, submission, wait), Select returns a synthetic `SelectOutcome::Recv { index: ReceiverIndex(0), result: Err(RecvError) }`. The index 0 is arbitrary — no arm actually fired. If sonnet picks a different arbitrary index (say, an unbounded `last_known_arm`), it's still honest but inconsistent with the BRIEF's convention.

**Mitigation:** BRIEF spells out the convention (ReceiverIndex(0) for substrate failures; ReceiverIndex(arm_idx) for arm-specific failures).

### Risk 7 — Fast-path accumulator check ordering

**What:** Before io_uring, Select iterates receivers and checks each accumulator. Lower-indexed receivers checked first. If receiver 0 has a frame AND receiver 1 has a frame, Select returns receiver 0's frame. This matches "registration-order priority" — consistent with crossbeam's default.

**Mitigation:** BRIEF uses `for (i, rx) in self.receivers.iter().enumerate()` — natural ordering.

### Risk 8 — Imports addition

**What:** Stone D1's imports added `CloseError, CommReceiver, CommSender, TryRecvError`. Stone D2 adds `ReceiverIndex, SelectOutcome` to the same `use crate::comms::{...}` line. If sonnet adds in a separate `use` statement, the file gets two duplicate import lines — gaze may flag.

**Mitigation:** BRIEF explicit on the merged import line.

### Risk 9 — Preserving Stones A-D1 unchanged

**What:** Stone D2 ADDS Select around D1's methods + Stones A-C's helpers. All preserved unchanged.

**Mitigation:** BRIEF's STOP triggers list every helper as UNCHANGED.

### Risk 10 — Test name prefix

**What:** New tests use `probe_slice3d2_*` prefix; existing tests keep their `probe_slice3c_*` and `probe_slice3d1_*` prefixes. If sonnet renames or uses a different prefix, gaze flags inconsistency.

**Mitigation:** BRIEF explicit.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | Module-level doc: "(through Stone D1)" → "(through Stone D2)" naming Select | YES |
| 2 | Imports: existing `use crate::comms::{...}` extended with `ReceiverIndex, SelectOutcome` (NOT duplicate use line) | YES |
| 3 | `Select<'a, T: HolonRepresentable>` struct with `receivers: Vec<&'a Receiver<T>>` + `_phantom: PhantomData<T>` | YES |
| 4 | `Select::new() -> Self` constructs empty | YES |
| 5 | `Select::recv(&mut self, rx: &'a Receiver<T>) -> ReceiverIndex` returns registration-order index | YES |
| 6 | `Select::select(&mut self) -> SelectOutcome<T>` fast-path: iterate receivers; return first accumulator-buffered frame | YES |
| 7 | Fast-path iteration uses `for (i, rx) in self.receivers.iter().enumerate()` | YES |
| 8 | Slow-path computes `arm_count = receivers.len() + if broadcast_fd >= 0 { 1 } else { 0 }` | YES |
| 9 | Slow-path `ring_capacity = ((arm_count.max(1)).next_power_of_two() as u32).max(2)` | YES |
| 10 | Slow-path: per-call IoUring (poll ring); IoUring::new failure → synthetic Recv ReceiverIndex(0) Err | YES |
| 11 | `const BROADCAST_TOKEN: u64 = 0;` inside select() body | YES |
| 12 | When `broadcast_fd >= 0`: submit POLL_ADD on broadcast_fd with `libc::POLLHUP as u32` event mask + user_data 0 | YES |
| 13 | For each data receiver: submit POLL_ADD with `(libc::POLLIN \| libc::POLLHUP) as u32` event mask + user_data `(i+1) as u64` | YES |
| 14 | `submit_and_wait(1)` is called; failure → synthetic Recv ReceiverIndex(0) Err | YES |
| 15 | Drain CQEs via `while let Some(cqe) = ring.completion().next()` (ALL ready CQEs drained) | YES |
| 16 | On CQE result < 0: synthetic Recv ReceiverIndex(0) Err | YES |
| 17 | Broadcast token check: `if token == BROADCAST_TOKEN` → `fired_broadcast = true` | YES |
| 18 | Data arm dispatch: `arm = (token - 1) as usize`; first wins via `if first_data_arm.is_none()` | YES |
| 19 | Broadcast wins ties: `if fired_broadcast { return SelectOutcome::Shutdown; }` | YES |
| 20 | Defensive empty drain: `continue` to retry the loop (NOT a synthetic error return) | YES |
| 21 | Read step uses SEPARATE per-call IoUring (read_ring; size 2) | YES |
| 22 | Read CQE result < 0 OR == 0: synthetic Recv ReceiverIndex(arm_idx) Err (NOT ReceiverIndex(0); arm-specific) | YES |
| 23 | On complete frame: `SelectOutcome::Recv { index: ReceiverIndex(arm_idx), result: decode_frame::<T>(&frame) }` | YES |
| 24 | On partial bytes (no frame after Read): continue loop (re-poll all arms; broadcast can fire mid-drain) | YES |
| 25 | `impl Default for Select<'a, T>` delegates to `new()` | YES |
| 26 | TWO new `unsafe` blocks in select() (POLL_ADD push × N+1 within one unsafe; Read push); each has SAFETY comment | YES |
| 27 | Tests preserve 6 `probe_slice3c_*` + 10 `probe_slice3d1_*` unchanged | YES |
| 28 | 2 new `probe_slice3d2_*` tests added | YES |
| 29 | All 18 probe tests PASS | YES |
| 30 | `cargo build --release` clean | YES |
| 31 | Prior 4 probe suites unchanged | YES |
| 32 | Zero modifications outside 2-file scope + SCORE doc | YES |
| 33 | Dirty tree + typed_channel.rs + edn_shim.rs + comms/mod.rs + Cargo.toml untouched | YES |
| 34 | NO `wat_arc170_program_contracts` re-run | YES |
| 35 | NO Stone E work (persistent ring, config tunable) | YES |
| 36 | Stones A-C helpers UNCHANGED | YES |
| 37 | Stone D1 methods + trait impls UNCHANGED | YES |
| 38 | Every new public item has a doc comment (gaze L2 pre-emption) | YES |
| 39 | Every new `unsafe` block has a SAFETY comment (forge pre-emption) | YES |
| 40 | NO commit | YES |

## Mode classification

- **Mode A:** all 40 criteria satisfied
- **Mode B (acceptable):**
  - Risk 1 fires (ring sizing): probe test 9 (`select_picks_fired_receiver`) catches
  - Risk 2 fires (token scheme off-by-one): probe test 10 (`indices_match_registration_order`) catches
  - Risk 4 fires (Read step merged with poll ring): submission queue overflow; cargo error
  - One probe test fails: sonnet investigates
- **Mode C (failure):**
  - Touched any file outside 2-file scope + SCORE doc
  - Modified Stones A-D1 helpers/methods
  - Implemented Stone E territory

## Calibration metadata

- **Orchestrator confidence:** MEDIUM-HIGH. Stone D2 is novel substrate work (N+1-arm orchestration) but the pattern generalizes Stone B's 2-arm case. The BRIEF skeleton is exhaustive — every io_uring submission + drain path is spelled out.
- **Risk factors:** Ring sizing micro-formula; token off-by-one; bail-out semantic consistency.

## Ward pass prediction

- gaze: 0-1 (smaller surface than original Stone D; possible mumble on bail-out arbitrary index 0)
- forge: 1-2 (Select's bail-out pattern; PhantomData invariance; possible candidate-rune on Select<'a, T> being a side-effecting fan-in struct)
- reap: 0
- sever: 0-1 (Select::select bundles "submit POLL_ADDs" + "drain CQEs" + "Read from fired arm" + "decode frame" — sequence, not braid; but ward may flag length)
- temper: 1-2 (two per-call IoUring per select() iteration — DOUBLE the per-call overhead vs recv; Stone E persistifies; flag as known-deferred)

Total: 2-6 findings. Round 2 likely.

## Cross-references

- BRIEF-214-SLICE-3D2-SELECT.md — this stone's work order
- BRIEF-214-SLICE-3D1-MECHANICAL-METHODS.md — D1 (just-shipped pre-D2)
- BRIEF-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md — Stone B 2-arm pattern (D2 generalizes)
- `src/comms/thread.rs` — Slice 2 thread tier Select<'a, T> MIRROR REFERENCE
- `src/comms/mod.rs` — Slice 1 `ReceiverIndex` + `SelectOutcome<T>`
- `feedback_iterative_complexity` — Stone D split into D1 + D2 per four-questions
