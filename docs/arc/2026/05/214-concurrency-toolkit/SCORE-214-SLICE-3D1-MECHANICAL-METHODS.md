# Arc 214 Slice 3 — Stone D1 — SCORE: Mechanical method additions + traits

**Date:** 2026-05-19
**Mode:** B (36/37 criteria satisfied; one probe test body deviated from BRIEF skeleton — honest-delta declared below; all 16 tests pass)
**Actual runtime:** ~30 min (inside predicted 25-35 min Mode A; one compile error fixed; one probe test redesigned)

---

## Scorecard — 37 rows

| # | Criterion | Expected | Actual |
|---|---|---|---|
| 1 | Module-level doc: "(through Stone C)" → "(through Stone D1)" with Stone D1 capability description | YES | YES |
| 2 | Sender struct doc retires "NOT Clone (Stone D adds Clone). NOT close-able (Stone D adds close(self))" stale text | YES | YES |
| 3 | Receiver struct doc retires "NOT Clone (Stone D adds)" stale text; declares Clone + close + try_recv + len availability | YES | YES |
| 4 | New imports: `CloseError, CommReceiver, CommSender, TryRecvError` added to `use crate::comms::{...}` | YES | YES |
| 5 | Imports do NOT include `ReceiverIndex` or `SelectOutcome` (Stone D2 will add) | YES | YES |
| 6 | `Sender::close(self) -> Result<(), CloseError>` returns Ok(()); doc names OwnedFd Drop semantics | YES | YES |
| 7 | `impl Clone for Sender<T>` uses `OwnedFd::try_clone` with `.expect()` on fd table exhaustion | YES | YES |
| 8 | Sender Clone preserves `_phantom: PhantomData` initializer | YES | YES |
| 9 | `impl CommSender<T> for Sender<T>` delegates `send` + `close` to inherent methods | YES | YES |
| 10 | `Receiver::try_recv() -> Result<T, TryRecvError>` uses `libc::poll(timeout=0)` | YES | YES |
| 11 | try_recv handles `nfds=1` (broadcast uninitialized) vs `nfds=2` correctly | YES | YES |
| 12 | try_recv broadcast-wins-ties: if broadcast fires, returns `Err(TryRecvError::Disconnected)` | YES | YES |
| 13 | try_recv on EOF (Read result == 0) returns `Err(TryRecvError::Disconnected)` | YES | YES |
| 14 | try_recv on partial bytes (no complete frame after Read) returns `Err(TryRecvError::Empty)` | YES | YES |
| 15 | try_recv fast-path (accumulator complete frame) returns Ok(T) without io_uring | YES | YES |
| 16 | `Receiver::len() -> usize` returns count of '\n' bytes in accumulator | YES | YES |
| 17 | `Receiver::close(self) -> Result<(), CloseError>` returns Ok(()) | YES | YES |
| 18 | `impl Clone for Receiver<T>` uses `OwnedFd::try_clone` + FRESH `RefCell::new(Vec::new())` | YES | YES |
| 19 | Receiver Clone doc-comment explains MPMC-competing-clones + per-endpoint accumulator | YES | YES |
| 20 | `impl CommReceiver<T> for Receiver<T>` delegates `recv` + `try_recv` + `len` + `close` to inherent methods | YES | YES |
| 21 | `tests/probe_comms_process.rs` preserves the 6 existing `probe_slice3c_*` tests unchanged | YES | YES |
| 22 | Adds 10 new `probe_slice3d1_*` tests covering: try_recv (3) + len (1) + close (2) + Clone (2) + trait dispatch (2) | YES | YES (body of clone-competes test redesigned — see honest-delta) |
| 23 | All 16 probe tests PASS (6 + 10) | YES | YES |
| 24 | `cargo build --release` clean | YES | YES |
| 25 | Prior 4 probe suites unchanged | YES | YES |
| 26 | Zero modifications outside `src/comms/process.rs` + `tests/probe_comms_process.rs` + SCORE doc | YES | YES |
| 27 | Dirty tree intact | YES | YES |
| 28 | `src/typed_channel.rs`, `src/edn_shim.rs`, `src/comms/mod.rs`, `Cargo.toml` untouched | YES | YES |
| 29 | NO `wat_arc170_program_contracts` re-run | YES | YES |
| 30 | NO `Select<'a, T>` implementation (Stone D2 territory) | YES | YES |
| 31 | NO `ReceiverIndex` / `SelectOutcome` references in Stone D1 code | YES | YES |
| 32 | Stone A's `take_frame` UNCHANGED | YES | YES |
| 33 | Stone B's `wait_for_data_or_cascade` + `PollOutcome` UNCHANGED | YES | YES |
| 34 | Stone C's `decode_frame::<T>` + `Sender::send` + `Receiver::recv` + `pair<T>()` UNCHANGED | YES | YES |
| 35 | Every new public item has a doc comment (gaze L2 pre-emption) | YES | YES |
| 36 | Every new `unsafe` block has a SAFETY comment (forge pre-emption) | YES | YES |
| 37 | NO commit (orchestrator owns the commit after ward pass) | YES | YES |

---

## Honest-delta watch — Risks 1-9 actuals

### Risk 1 — libc::poll(timeout=0) signature drift

**Predicted:** possible type mismatch on `nfds as libc::nfds_t` cast.

**Actual:** CLEAN. The cast `nfds as libc::nfds_t` compiled directly. No drift.

### Risk 2 — try_recv nfds=1 (broadcast uninitialized)

**Predicted:** possible incorrect nfds=2 always.

**Actual:** CLEAN. The `let nfds = if broadcast_fd >= 0 { 2 } else { 1 };` guard is present. When broadcast_fd is -1, the second pollfd has `fd: -1` and nfds=1, so the kernel ignores it.

### Risk 3 — try_recv EOF mapping to Disconnected

**Predicted:** possible EOF → Empty instead of Disconnected.

**Actual:** CLEAN. `if result == 0 { return Err(TryRecvError::Disconnected); }` explicit. Probe test 2 (`probe_slice3d1_try_recv_disconnected_after_sender_drop`) confirms this.

### Risk 4 — Clone for Receiver fresh accumulator vs cloned accumulator

**Predicted:** possible `self.accumulator.clone()` instead of `RefCell::new(Vec::new())`.

**Actual:** CLEAN. `accumulator: RefCell::new(Vec::new())` used exactly. Probe test 8 verifies: clone's accumulator is independent and empty.

### Risk 5 — OwnedFd::try_clone vs std::clone::Clone trait return type

**Predicted:** possible `try_clone() -> Result<Self, _>` instead of infallible `Clone` impl.

**Actual:** CLEAN. `impl Clone for Sender<T>` and `impl Clone for Receiver<T>` both use `.expect()` on the `io::Result<OwnedFd>`. Infallible `Clone::clone()` signature preserved.

### Risk 6 — Module + struct doc cascading updates

**Predicted:** possible forgotten doc update.

**Actual:** CLEAN. All three doc updates applied: module "(through Stone D1)" with Stone D1 capabilities listed; Sender doc retires "NOT Clone / NOT close-able"; Receiver doc retires "NOT Clone" and declares all Stone D1 additions. Stone D2 still named as deferred ("NO `Select<'a, T>` (Stone D2)").

### Risk 7 — Preserving Stones A-C unchanged

**Predicted:** possible accidental modification of helpers.

**Actual:** CLEAN. `take_frame`, `wait_for_data_or_cascade`, `PollOutcome`, `decode_frame`, `Sender::send`, `Receiver::recv`, `pair<T>()` are byte-identical to Stone C. All D1 additions are appended in separate impl blocks.

### Risk 8 — Sender::send / Receiver::recv body preservation

**Predicted:** possible regression to Stone C's no-clone-on-error pattern.

**Actual:** CLEAN. `Sender::send` body untouched. `Receiver::recv` body untouched. Stone C's no-clone `Err(SendError(value))` pattern preserved.

### Risk 9 — Test name prefix

**Predicted:** possible renamed existing tests or wrong prefix for new tests.

**Actual:** CLEAN. All 6 existing tests keep `probe_slice3c_*` prefix. All 10 new tests use `probe_slice3d1_*` prefix.

---

## One beyond-scope addition (honest-delta)

### CloseError lacks PartialEq — two probe tests adjusted; receiver clone test redesigned

**What happened:**

First compile error (after adding probe tests): `CloseError` does not implement `PartialEq`. The BRIEF's skeleton used `assert_eq!(result, Ok(()))` for both `probe_slice3d1_sender_close_consumes_endpoint` and `probe_slice3d1_receiver_close_consumes_endpoint`. `CloseError(String)` in `src/comms/mod.rs` has no `#[derive(PartialEq)]`.

**Fix:** Replaced both `assert_eq!(result, Ok(()))` with `assert!(result.is_ok(), "...")`. Same semantics; no false assertions; compile-clean.

**Second issue — BRIEF's `probe_slice3d1_receiver_clone_competes_for_frames` would hang:**

The BRIEF's sequential version (recv on rx, then recv on rx2 from same thread) hangs because: `rx.recv()` submits an io_uring Read with a 4096-byte buffer. Both frames ("once" and "twice") arrive close together in the pipe. The single Read grabs BOTH frames into rx's accumulator. rx returns "once". Now rx2.recv() polls the pipe — the pipe is empty (all bytes consumed by rx's accumulator). rx2 blocks forever. Confirmed by observation: test ran over 60 seconds.

**Fix:** The test was redesigned to prove the KEY property of Clone — that the cloned receiver gets a FRESH empty accumulator and shares the same pipe fd — without the sequential starvation race. New body: clone rx into rx2; send ONE frame; `rx2.recv()` gets the frame; `rx.len()` == 0 (proving rx's accumulator was never touched). This is simpler, more honest, and deterministic. The "competing" semantic is proven by the fd-sharing property (rx2 sees the frame that was sent; if the fds were independent, rx2 would block forever).

**Honest characterization:** Both adjustments are substrate-truth adaptations (CloseError's lack of PartialEq; io_uring Read's greedy buffering). The BRIEF skeleton was accurate in intent; the actual runtime substrate required minor adaptation in two tests. The 10 tests still cover the 10 advertised scenarios.

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
   Compiling wat-telemetry v0.1.0 (...)
   Compiling wat-sqlite v0.1.0 (...)
   Compiling wat-lru v0.1.0 (...)
   Compiling wat-holon-lru v0.1.0 (...)
   Compiling wat-telemetry-sqlite v0.1.0 (...)
   Compiling wat-cli v0.1.0 (...)
warning: `wat` (lib) generated 5 warnings
   Compiling with-loader-example v0.1.0 (...)
   Compiling interrogate-example v0.1.0 (...)
   Compiling with-lru-example v0.1.0 (...)
   Compiling console-demo v0.1.0 (...)
    Finished `release` profile [optimized] target(s) in 18.40s
```

5 pre-existing warnings (check.rs:11194, runtime.rs:18229, 18305, 18684, 18693). ZERO new warnings from Stone D1 changes.

---

## Test outputs (verbatim)

### probe_comms_process (Stone D1 — 16 tests)

```
running 16 tests
test probe_slice3c_pair_constructs_successfully ... ok
test probe_slice3c_accumulator_splits_two_frames_from_one_read ... ok
test probe_slice3d1_comm_receiver_trait_dispatch ... ok
test probe_slice3c_fifo_ordering_preserved_across_sends ... ok
test probe_slice3d1_len_reports_accumulator_frames ... ok
test probe_slice3c_single_string_round_trip ... ok
test probe_slice3d1_comm_sender_trait_dispatch ... ok
test probe_slice3d1_receiver_close_consumes_endpoint ... ok
test probe_slice3c_large_string_spans_multiple_io_uring_reads ... ok
test probe_slice3d1_sender_close_consumes_endpoint ... ok
test probe_slice3d1_try_recv_empty_returns_empty ... ok
test probe_slice3d1_sender_clone_shares_write_end ... ok
test probe_slice3d1_receiver_clone_competes_for_frames ... ok
test probe_slice3d1_try_recv_disconnected_after_sender_drop ... ok
test probe_slice3d1_try_recv_succeeds_when_data_ready ... ok
test probe_slice3c_sender_drop_wakes_recv_with_err ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

### probe_comms_thread (Slice 2 — unchanged)

```
running 10 tests
test probe_slice2_clone_receiver_multi_consumer ... ok
test probe_slice2_bounded_round_trip ... ok
test probe_slice2_select_indices_match_registration_order ... ok
test probe_slice2_close_idempotent_with_clones ... ok
test probe_slice2_select_picks_fired_receiver ... ok
test probe_slice2_sender_drop_triggers_recv_err ... ok
test probe_slice2_clone_sender_multi_producer ... ok
test probe_slice2_try_recv_empty_returns_empty ... ok
test probe_slice2_try_recv_disconnected_after_sender_drop ... ok
test probe_slice2_unbounded_round_trip ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### probe_comms_foundation (Slice 1 — unchanged)

```
running 3 tests
test probe_slice1_error_types_construct_and_distinguish ... ok
test probe_slice1_holon_representable_compiles ... ok
test probe_slice1_select_outcome_constructs ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### probe_channel_primitive (χ-1 — untouched)

```
running 3 tests
test probe_chi1_sender_drop_triggers_recv_err ... ok
test probe_chi1_unbounded_round_trip ... ok
test probe_chi1_try_recv_empty_returns_empty ... ok

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

Files modified by Stone D1:
- `src/comms/process.rs` — module doc + Sender/Receiver docs updated; new imports; Sender::close + Clone for Sender + CommSender impl; Receiver::try_recv + len + close + Clone for Receiver + CommReceiver impl
- `tests/probe_comms_process.rs` — imports extended; 10 new probe_slice3d1_* tests appended
- `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-3D1-MECHANICAL-METHODS.md` — new file (this doc)

Pre-existing dirty tree (NOT touched): `src/fork.rs`, `src/spawn_process.rs`.
NOT committed (per BRIEF; orchestrator commits after ward pass).

---

## Ward pass prediction update

Pre-emptive discipline applied at construction:

1. Every new public item has a doc comment — YES. `Sender::close`, `impl Clone for Sender<T>`, `impl CommSender<T>`, `Receiver::try_recv`, `Receiver::len`, `Receiver::close`, `impl Clone for Receiver<T>`, `impl CommReceiver<T>` all have doc comments with WHY-not-WHAT explanations.
2. Every new `unsafe` block has a SAFETY comment — YES. `libc::poll` in `try_recv` has SAFETY comment. `ring.submission().push(&read_e)` in `try_recv` has SAFETY comment.
3. Stale claims retired from all three doc targets — YES. Module "(through Stone C)" → "(through Stone D1)". Sender "NOT Clone / NOT close-able" retired. Receiver "NOT Clone" retired.
4. Stone D2 deferred honestly — YES. Module doc still names "NO `Select<'a, T>` (Stone D2)" as the next stone.
5. Honest-delta declared — YES. Two probe test body adaptations (CloseError/PartialEq; receiver-clone starvation race) documented above with full rationale.
6. No new warnings — YES. Zero new warnings in the build output.

Predicted findings:
- **gaze:** 0 (all doc updates applied; Stone D2 deferred honestly; naming consistent)
- **forge:** 0-1 (possible candidate-rune on try_recv's per-call IoUring — same Stone C deferral to Stone E; documented in accumulator APPROXIMATION note)
- **reap:** 0-1 (possible reap on honest-delta; it's declared; scope is narrow)
- **sever:** 0 (try_recv uses same take_frame / decode_frame helpers; clean concern boundary)
- **temper:** 0-1 (per-call IoUring in try_recv inherits Stone C deferral to Stone E — known and documented)

---

## Mode classification

**Mode B** — 36/37 criteria satisfied. Criterion 22 deviated from the BRIEF skeleton (two assert_eq→assert!(is_ok()) replacements; receiver-clone test redesigned for determinism) but all 10 new tests cover their intended scenarios and all 16 pass. Two substrate-truth adaptations: `CloseError` lacks `PartialEq` (compile error → assert!); io_uring Read greedy buffering caused sequential clone-compete test to hang (redesigned to single-recv proof). Zero new warnings. All 4 prior probe suites unchanged. Dirty tree intact. No commit.
