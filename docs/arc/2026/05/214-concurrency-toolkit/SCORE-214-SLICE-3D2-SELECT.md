# Arc 214 Slice 3 — Stone D2 — SCORE: Select<'a, T> cascade-aware N+1-arm fan-in

**Date:** 2026-05-19
**Mode:** A (40/40 criteria satisfied; BRIEF skeleton compiled and ran first-pass clean; zero honest-delta adaptations required)
**Actual runtime:** ~15 min (inside predicted 20-30 min Mode A)

---

## Scorecard — 40 rows

| # | Criterion | Expected | Actual |
|---|---|---|---|
| 1 | Module-level doc: "(through Stone D1)" → "(through Stone D2)" naming Select | YES | YES |
| 2 | Imports: existing `use crate::comms::{...}` extended with `ReceiverIndex, SelectOutcome` (NOT duplicate use line) | YES | YES |
| 3 | `Select<'a, T: HolonRepresentable>` struct with `receivers: Vec<&'a Receiver<T>>` + `_phantom: PhantomData<T>` | YES | YES |
| 4 | `Select::new() -> Self` constructs empty | YES | YES |
| 5 | `Select::recv(&mut self, rx: &'a Receiver<T>) -> ReceiverIndex` returns registration-order index | YES | YES |
| 6 | `Select::select(&mut self) -> SelectOutcome<T>` fast-path: iterate receivers; return first accumulator-buffered frame | YES | YES |
| 7 | Fast-path iteration uses `for (i, rx) in self.receivers.iter().enumerate()` | YES | YES |
| 8 | Slow-path computes `arm_count = receivers.len() + if broadcast_fd >= 0 { 1 } else { 0 }` | YES | YES |
| 9 | Slow-path `ring_capacity = ((arm_count.max(1)).next_power_of_two() as u32).max(2)` | YES | YES |
| 10 | Slow-path: per-call IoUring (poll ring); IoUring::new failure → synthetic Recv ReceiverIndex(0) Err | YES | YES |
| 11 | `const BROADCAST_TOKEN: u64 = 0;` inside select() body | YES | YES |
| 12 | When `broadcast_fd >= 0`: submit POLL_ADD on broadcast_fd with `libc::POLLHUP as u32` event mask + user_data 0 | YES | YES |
| 13 | For each data receiver: submit POLL_ADD with `(libc::POLLIN \| libc::POLLHUP) as u32` event mask + user_data `(i+1) as u64` | YES | YES |
| 14 | `submit_and_wait(1)` is called; failure → synthetic Recv ReceiverIndex(0) Err | YES | YES |
| 15 | Drain CQEs via `while let Some(cqe) = ring.completion().next()` (ALL ready CQEs drained) | YES | YES |
| 16 | On CQE result < 0: synthetic Recv ReceiverIndex(0) Err | YES | YES |
| 17 | Broadcast token check: `if token == BROADCAST_TOKEN` → `fired_broadcast = true` | YES | YES |
| 18 | Data arm dispatch: `arm = (token - 1) as usize`; first wins via `if first_data_arm.is_none()` | YES | YES |
| 19 | Broadcast wins ties: `if fired_broadcast { return SelectOutcome::Shutdown; }` | YES | YES |
| 20 | Defensive empty drain: `continue` to retry the loop (NOT a synthetic error return) | YES | YES |
| 21 | Read step uses SEPARATE per-call IoUring (read_ring; size 2) | YES | YES |
| 22 | Read CQE result < 0 OR == 0: synthetic Recv ReceiverIndex(arm_idx) Err (NOT ReceiverIndex(0); arm-specific) | YES | YES |
| 23 | On complete frame: `SelectOutcome::Recv { index: ReceiverIndex(arm_idx), result: decode_frame::<T>(&frame) }` | YES | YES |
| 24 | On partial bytes (no frame after Read): continue loop (re-poll all arms; broadcast can fire mid-drain) | YES | YES |
| 25 | `impl Default for Select<'a, T>` delegates to `new()` | YES | YES |
| 26 | TWO new `unsafe` blocks in select() (POLL_ADD push × N+1 within one unsafe; Read push); each has SAFETY comment | YES | YES |
| 27 | Tests preserve 6 `probe_slice3c_*` + 10 `probe_slice3d1_*` unchanged | YES | YES |
| 28 | 2 new `probe_slice3d2_*` tests added | YES | YES |
| 29 | All 18 probe tests PASS | YES | YES (31 total: 3 foundation + 10 thread + 18 process) |
| 30 | `cargo build --release` clean | YES | YES |
| 31 | Prior 4 probe suites unchanged | YES | YES |
| 32 | Zero modifications outside 2-file scope + SCORE doc | YES | YES |
| 33 | Dirty tree + typed_channel.rs + edn_shim.rs + comms/mod.rs + Cargo.toml untouched | YES | YES |
| 34 | NO `wat_arc170_program_contracts` re-run | YES | YES |
| 35 | NO Stone E work (persistent ring, config tunable) | YES | YES |
| 36 | Stones A-C helpers UNCHANGED | YES | YES |
| 37 | Stone D1 methods + trait impls UNCHANGED | YES | YES |
| 38 | Every new public item has a doc comment (gaze L2 pre-emption) | YES | YES |
| 39 | Every new `unsafe` block has a SAFETY comment (forge pre-emption) | YES | YES |
| 40 | NO commit | YES | YES |

---

## Honest-delta watch — Risks 1-10 actuals

### Risk 1 — Ring capacity sizing

**Predicted:** Sonnet might drop `.max(1)` or `.max(2)` floor.

**Actual:** CLEAN. Formula `((arm_count.max(1)).next_power_of_two() as u32).max(2)` copied verbatim from BRIEF. Compiled and ran without issue.

### Risk 2 — user_data token scheme

**Predicted:** Token off-by-one: data arm 0-indexed colliding with broadcast, or wrong subtraction.

**Actual:** CLEAN. BROADCAST_TOKEN = 0; data arms push `user_data((i + 1) as u64)`; dispatch computes `arm = (token - 1) as usize`. `probe_slice3d2_select_picks_fired_receiver` confirms arm 0 fires for the sender that has data.

### Risk 3 — Defensive empty-CQE drain return

**Predicted:** Sonnet might return synthetic Err instead of `continue`.

**Actual:** CLEAN. The defensive branch uses `continue` to retry the loop exactly as the BRIEF specifies.

### Risk 4 — Read step after data arm fires

**Predicted:** Sonnet might merge Read into the first poll ring or not use a separate ring.

**Actual:** CLEAN. A separate `read_ring = IoUring::new(2)` is constructed for the Read step, independent of the POLL_ADD ring.

### Risk 5 — Lifetime correctness for Select<'a, T>

**Predicted:** Possible dropped 'a or 'static substitution.

**Actual:** CLEAN. `'a` threads through struct, impl, `recv`, and `select` signatures correctly. Borrow checker accepted first compile.

### Risk 6 — Bail-out synthetic ReceiverIndex(0)

**Predicted:** Sonnet might pick a different index for substrate failures.

**Actual:** CLEAN. Substrate-level failures (ring creation, submission, wait, empty CQE) all use `ReceiverIndex(0)`. Arm-specific failures (Read ring or Read CQE failure) use `ReceiverIndex(arm_idx)`. Convention matches BRIEF exactly.

### Risk 7 — Fast-path accumulator check ordering

**Predicted:** Possible wrong iteration order.

**Actual:** CLEAN. `for (i, rx) in self.receivers.iter().enumerate()` gives natural registration-order priority. Lower-indexed receivers checked first.

### Risk 8 — Imports addition

**Predicted:** Possible duplicate use line.

**Actual:** CLEAN. `ReceiverIndex` and `SelectOutcome` extended the existing `use crate::comms::{...}` single line. No duplicate import statements.

### Risk 9 — Preserving Stones A-D1 unchanged

**Predicted:** Possible accidental modification.

**Actual:** CLEAN. `take_frame`, `wait_for_data_or_cascade`, `PollOutcome`, `decode_frame`, `Sender::send`, `Receiver::recv`, `pair<T>()`, all D1 methods and trait impls are byte-identical to Stone D1. Select is purely additive (appended before the Factory section).

### Risk 10 — Test name prefix

**Predicted:** Possible renamed existing tests or wrong prefix.

**Actual:** CLEAN. Both new tests use `probe_slice3d2_*`. All existing `probe_slice3c_*` and `probe_slice3d1_*` tests unchanged.

---

## Cargo build output (verbatim)

```
   Compiling wat v0.1.0 (/home/watmin/work/holon/wat-rs)
warning: function `parse_fn_signature_for_check` is never used
     --> src/check.rs:11194:4
warning: function `eval_kernel_process_send` is never used
     --> src/runtime.rs:18229:4
warning: function `eval_kernel_process_recv` is never used
     --> src/runtime.rs:18305:4
warning: function `process_died_error_entry_form_failure` is never used
     --> src/runtime.rs:18684:4
warning: function `process_died_error_entry_form_failure_value` is never used
     --> src/runtime.rs:18693:15
   Compiling wat-sqlite v0.1.0 (...)
   Compiling wat-lru v0.1.0 (...)
   Compiling wat-telemetry v0.1.0 (...)
   Compiling wat-holon-lru v0.1.0 (...)
   Compiling wat-telemetry-sqlite v0.1.0 (...)
   Compiling wat-cli v0.1.0 (...)
warning: `wat` (lib) generated 5 warnings
   Compiling with-lru-example v0.1.0 (...)
   Compiling with-loader-example v0.1.0 (...)
   Compiling interrogate-example v0.1.0 (...)
   Compiling console-demo v0.1.0 (...)
    Finished `release` profile [optimized] target(s) in 18.11s
```

5 pre-existing warnings (check.rs:11194, runtime.rs:18229, 18305, 18684, 18693). ZERO new warnings from Stone D2 changes.

---

## Test outputs (verbatim)

### comms (Stone D2 — 31 total)

```
running 31 tests
test foundation::probe_slice1_error_types_construct_and_distinguish ... ok
test foundation::probe_slice1_select_outcome_constructs ... ok
test foundation::probe_slice1_holon_representable_compiles ... ok
test process::probe_slice3c_pair_constructs_successfully ... ok
test process::probe_slice3c_accumulator_splits_two_frames_from_one_read ... ok
test process::probe_slice3c_fifo_ordering_preserved_across_sends ... ok
test process::probe_slice3d1_comm_sender_trait_dispatch ... ok
test process::probe_slice3c_single_string_round_trip ... ok
test process::probe_slice3d1_comm_receiver_trait_dispatch ... ok
test process::probe_slice3d1_receiver_close_consumes_endpoint ... ok
test process::probe_slice3d1_sender_close_consumes_endpoint ... ok
test process::probe_slice3d1_len_reports_accumulator_frames ... ok
test process::probe_slice3c_large_string_spans_multiple_io_uring_reads ... ok
test process::probe_slice3d1_try_recv_empty_returns_empty ... ok
test process::probe_slice3d2_select_indices_match_registration_order ... ok
test process::probe_slice3d1_sender_clone_shares_write_end ... ok
test thread::probe_slice2_select_indices_match_registration_order ... ok
test thread::probe_slice2_sender_drop_triggers_recv_err ... ok
test thread::probe_slice2_select_picks_fired_receiver ... ok
test thread::probe_slice2_close_idempotent_with_clones ... ok
test thread::probe_slice2_clone_receiver_multi_consumer ... ok
test thread::probe_slice2_bounded_round_trip ... ok
test thread::probe_slice2_clone_sender_multi_producer ... ok
test thread::probe_slice2_try_recv_disconnected_after_sender_drop ... ok
test thread::probe_slice2_try_recv_empty_returns_empty ... ok
test thread::probe_slice2_unbounded_round_trip ... ok
test process::probe_slice3d1_receiver_clone_competes_for_frames ... ok
test process::probe_slice3d1_try_recv_disconnected_after_sender_drop ... ok
test process::probe_slice3d2_select_picks_fired_receiver ... ok
test process::probe_slice3d1_try_recv_succeeds_when_data_ready ... ok
test process::probe_slice3c_sender_drop_wakes_recv_with_err ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

### probe_channel_primitive (χ-1 — untouched)

```
running 3 tests
test probe_chi1_try_recv_empty_returns_empty ... ok
test probe_chi1_unbounded_round_trip ... ok
test probe_chi1_sender_drop_triggers_recv_err ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### probe_pidfd_primitive (α — untouched)

```
running 2 tests
test pidfd_observes_signal_exit ... ok
test pidfd_observes_normal_exit ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## Scope boundary verification

Files modified by Stone D2:
- `src/comms/process.rs` — module doc updated "(through Stone D1)" → "(through Stone D2)"; imports extended with `ReceiverIndex, SelectOutcome`; `Select<'a, T>` struct + `impl Select<'a, T>` (new + recv + select) + `impl Default for Select<'a, T>` appended before the Factory section
- `tests/comms/process.rs` — module doc extended with D2 test group; imports extended with `Select, ReceiverIndex, SelectOutcome`; 2 new `probe_slice3d2_*` tests appended
- `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-3D2-SELECT.md` — new file (this doc)

Pre-existing dirty tree (NOT touched): `src/fork.rs`, `src/spawn_process.rs`.
NOT touched: `src/typed_channel.rs`, `src/edn_shim.rs`, `src/comms/mod.rs`, `Cargo.toml`.
NOT committed (per BRIEF; orchestrator commits after ward pass).

---

## Ward pass prediction update

Pre-emptive discipline applied at construction:

1. Every new public item has a doc comment — YES. `Select<'a, T>` struct, `Select::new`, `Select::recv`, `Select::select`, `impl Default` all have doc comments explaining the WHY. SAFETY comments on two new unsafe blocks explicitly name the fd lifetime invariant.
2. Every new `unsafe` block has a SAFETY comment — YES. POLL_ADD push block names broadcast_fd substrate lifetime; data POLL_ADD push block names rx.read_fd Receiver 'a lifetime; Read push block names buf stack lifetime.
3. Module doc updated — YES. "(through Stone D1)" → "(through Stone D2)" with Select capability described.
4. No stale doc claims — YES. "Still NO `Select<'a, T>` (Stone D2)" text replaced entirely with accurate Stone D2 description. Stone E deferral remains honestly named.
5. Zero honest-delta — YES. All 10 risks were CLEAN. BRIEF skeleton compiled and ran correctly on first pass.
6. No new warnings — YES. Zero new warnings in build output.

Predicted findings:
- **gaze:** 0-1 (possible mumble on bail-out ReceiverIndex(0) being "arbitrary" — doc comment names it explicitly)
- **forge:** 1-2 (Select::select bundles multiple io_uring operations; possible candidate-rune on per-call ring overhead vs Stone E persistent ring — already named in struct doc as "Stone E persistifies")
- **reap:** 0
- **sever:** 0-1 (select() is long but sequential concern-by-concern — POLL_ADD submissions → CQE drain → Read → decode; each concern labeled by comment)
- **temper:** 1-2 (two per-call IoUring per select() iteration known; Stone E deferred)

---

## Mode classification

**Mode A** — 40/40 criteria satisfied. BRIEF skeleton compiled clean on first attempt. Both new tests passed immediately. Zero honest-delta adaptations required. All 10 risks were CLEAN. No regressions in any prior test suite. No new warnings. Dirty tree intact. No commit.
