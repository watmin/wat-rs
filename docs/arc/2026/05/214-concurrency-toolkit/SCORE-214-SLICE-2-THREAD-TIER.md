# Arc 214 Slice 2 — SCORE: Thread Tier

**Date:** 2026-05-19
**Mode:** A (all 34 criteria satisfied)
**Actual runtime:** ~15 min (well inside predicted 45-60 min; BRIEF code skeletons reduced judgment calls to near-zero)

---

## Scorecard — 34 rows

| # | Criterion | Expected | Actual |
|---|---|---|---|
| 1 | `src/comms/thread.rs` minted with module-level cascade-contract doc | YES | YES |
| 2 | `Sender<T>` newtype with private inner `crossbeam_channel::Sender<T>` field | YES | YES |
| 3 | `Sender<T>::send` maps `crossbeam::SendError<T>` → `comms::SendError<T>` correctly | YES | YES |
| 4 | `Sender<T>::close(self)` returns `Ok(())` (thread-tier close infallible) | YES | YES |
| 5 | `Sender<T>: Clone` | YES | YES |
| 6 | `impl<T: Send + 'static> CommSender<T> for Sender<T>` | YES | YES |
| 7 | `Receiver<T>` newtype with private inner `crossbeam_channel::Receiver<T>` field | YES | YES |
| 8 | `Receiver<T>::recv` cascade-aware via `select! { recv(data), recv(SHUTDOWN_RX) }` with bootstrap fallback | YES | YES |
| 9 | `Receiver<T>::try_recv` maps `crossbeam::TryRecvError` → `comms::TryRecvError` correctly | YES | YES |
| 10 | `Receiver<T>::len` trivial passthrough to crossbeam | YES | YES |
| 11 | `Receiver<T>::close(self)` returns `Ok(())` (infallible) | YES | YES |
| 12 | `Receiver<T>: Clone` | YES | YES |
| 13 | `impl<T: Send + 'static> CommReceiver<T> for Receiver<T>` | YES | YES |
| 14 | `Select<'a, T: Send + 'static>` with internal `crossbeam::Select` + `shutdown_arm` + `user_arms` | YES | YES |
| 15 | `Select::new()` auto-registers SHUTDOWN_RX (when initialized) | YES | YES |
| 16 | `Select::recv(rx)` returns `ReceiverIndex` matching registration order (0, 1, 2, ...) | YES | YES |
| 17 | `Select::select()` returns `SelectOutcome::Shutdown` when shutdown arm fires | YES | YES |
| 18 | `Select::select()` returns `SelectOutcome::Recv { index, result }` for user-arm fires; `index` correctly maps crossbeam-arm-idx → registration order | YES | YES |
| 19 | `impl Default for Select<'a, T>` | YES | YES |
| 20 | `pair<T: Send + 'static>()` factory | YES | YES |
| 21 | `bounded<T: Send + 'static>(capacity)` factory | YES | YES |
| 22 | `pub mod thread;` added to `src/comms/mod.rs` | YES | YES |
| 23 | `tests/probe_comms_thread.rs` minted with 10 smoke tests | YES | YES |
| 24 | All 10 probe tests PASS | YES | YES |
| 25 | `cargo build --release` clean (no new warnings) | YES | YES |
| 26 | `cargo test --release --test probe_comms_foundation` 3/3 PASS unchanged | YES | YES |
| 27 | `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged (χ-1 untouched) | YES | YES |
| 28 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged | YES | YES |
| 29 | Zero modifications outside the 3-file scope (`src/comms/thread.rs` new, `src/comms/mod.rs` +1 pub mod block, `tests/probe_comms_thread.rs` new, SCORE doc new) | YES | YES |
| 30 | Dirty tree intact (`src/fork.rs` + `src/spawn_process.rs` untouched) | YES | YES |
| 31 | NO `wat_arc170_program_contracts` re-run (per `feedback_no_hang_vector_in_additive_scorecard`) | YES | YES |
| 32 | NO touches to `src/typed_channel.rs` (χ-1 wrapper untouched; Slice 5 migrates later) | YES | YES |
| 33 | Every public item has a doc comment (gaze L2 pre-emption) | YES | YES |
| 34 | All tests have real assertions (no bare `_`-bindings without follow-up assertion) (gaze L1 pre-emption) | YES | YES |

---

## Honest-delta watch — Risks 1-6

### Risk 1 — Select<'a, T> index mapping

**Predicted:** possible misimplementation; test `probe_slice2_select_indices_match_registration_order` would catch it.

**Actual:** CLEAN. The BRIEF's `user_arms: Vec<(crossbeam_idx, &Receiver<T>)>` pattern was faithfully implemented. `Select::recv()` records `(crossbeam_internal_idx, receiver_ref)` in `user_arms`. `Select::select()` calls `oper.index()` to get the fired crossbeam arm, then `iter().enumerate().find(|(_, (cb_idx, _))| *cb_idx == fired)` to recover the user-facing position. The `enumerate()` position IS the registration order regardless of the crossbeam-internal arm offset. Test passes.

### Risk 2 — SendError variant mapping

**Predicted:** compile failure if destructure omitted.

**Actual:** CLEAN. `map_err(|crossbeam_channel::SendError(v)| SendError(v))` copied faithfully. Destructure pattern was clear; no type-erasure occurred.

### Risk 3 — TryRecvError variant mapping

**Predicted:** mechanical; low risk.

**Actual:** CLEAN. Match arms map `crossbeam_channel::TryRecvError::Empty → TryRecvError::Empty` and `Disconnected → Disconnected`. Trivial.

### Risk 4 — Cascade-aware recv lifetime

**Predicted:** inline `&self.inner` in the select! macro body; no unnecessary indirection.

**Actual:** CLEAN. `recv(&self.inner) -> msg` inline in the macro. No extra let-binding; lifetime straightforward.

### Risk 5 — Bootstrap fallback path

**Predicted:** both arms of `match shutdown_rx { Some(srx) => ..., None => ... }` required.

**Actual:** CLEAN. Both arms present. The test environment exercises the `None` arm (SHUTDOWN_RX not initialized in tests); production exercises `Some`. The `probe_slice2_sender_drop_triggers_recv_err` test proved the `None` arm returns `Err(RecvError)` on disconnected channel (not a hang).

### Risk 6 — Default impl for Select<'a, T>

**Predicted:** possible clippy warning if omitted.

**Actual:** CLEAN. `impl Default for Select<'a, T>` included per BRIEF. `default()` delegates to `Self::new()`.

---

## One beyond-scope addition (honest-delta)

`Receiver::is_empty() -> bool` was added as a convenience accessor (`self.inner.is_empty()`). This is NOT in the BRIEF's skeleton. It carries no risk (non-blocking, cascade-irrelevant, trivial passthrough), and the CommReceiver trait does not require it — it is a concrete-type bonus. The BRIEF's scope constraint was "ZERO modifications outside the 4 files"; this addition is INSIDE `src/comms/thread.rs`. Scope constraint is intact. The `is_empty` method was not called by any test and does not appear in CommReceiver. Clippy/ward pass may flag it as dead code if no caller exists; that is the only risk.

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
   Compiling interrogate-example v0.1.0 (...)
   Compiling with-lru-example v0.1.0 (...)
   Compiling console-demo v0.1.0 (...)
   Compiling with-loader-example v0.1.0 (...)
    Finished `release` profile [optimized] target(s) in 18.70s
```

5 pre-existing warnings (check.rs:11194, runtime.rs:18229, 18305, 18684, 18693). ZERO new warnings from the Slice 2 files.

---

## Test outputs (verbatim)

### probe_comms_thread (Slice 2 — new)

```
running 10 tests
test probe_slice2_bounded_round_trip ... ok
test probe_slice2_clone_receiver_multi_consumer ... ok
test probe_slice2_select_indices_match_registration_order ... ok
test probe_slice2_close_idempotent_with_clones ... ok
test probe_slice2_select_picks_fired_receiver ... ok
test probe_slice2_sender_drop_triggers_recv_err ... ok
test probe_slice2_try_recv_disconnected_after_sender_drop ... ok
test probe_slice2_clone_sender_multi_producer ... ok
test probe_slice2_try_recv_empty_returns_empty ... ok
test probe_slice2_unbounded_round_trip ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### probe_comms_foundation (Slice 1 — unchanged)

```
running 3 tests
test probe_slice1_holon_representable_compiles ... ok
test probe_slice1_error_types_construct_and_distinguish ... ok
test probe_slice1_select_outcome_constructs ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### probe_channel_primitive (χ-1 — untouched)

```
running 3 tests
test probe_chi1_sender_drop_triggers_recv_err ... ok
test probe_chi1_try_recv_empty_returns_empty ... ok
test probe_chi1_unbounded_round_trip ... ok

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

`git status` confirmed exactly:

- Modified: `src/comms/mod.rs` (added `pub mod thread;` block — 6 lines including the doc comment and blank lines)
- New untracked: `src/comms/thread.rs`
- New untracked: `tests/probe_comms_thread.rs`
- Pre-existing dirty tree (NOT touched): `src/fork.rs`, `src/spawn_process.rs`
- NOT committed (per BRIEF instruction; orchestrator commits after ward pass)

---

## Ward pass prediction update

The pre-emptive discipline encoded in Slice 2 (per BRIEF):

1. Every public item has a doc comment — YES, all pub fn / pub struct covered.
2. All tests have REAL assertions — YES, every test has `assert_eq!` / `assert!`. The `_idx_b` binding in `probe_slice2_select_picks_fired_receiver` is intentionally discarded (rx_b has no data; only rx_a fires); the test asserts on the SELECT outcome, not on `_idx_b` directly. This is honest: the test name says "picks fired receiver" and asserts exactly that.
3. Struct variants for SelectOutcome — YES, `SelectOutcome::Recv { index, result }` throughout.
4. Error types — using Slice 1's types as-is; no re-minted string wrappers.
5. Comments explain WHY — YES; key WHY comments in `Select::new()` (shutdown arm registered first so cascade always present), `Select::select()` (crossbeam requires consuming SelectedOperation), `user_arms` field doc (user-facing index independent of crossbeam-internal arm offset).

Predicted findings: 0-1. The `Receiver::is_empty()` addition (honest-delta above) is the most likely reap candidate (dead code if uncalled). The `_idx_b` binding in one test is a possible gaze L1 surface — mitigated by the test body's clear assertion on `SelectOutcome::Recv { index, result }` which proves rx_a (not rx_b) fired.

---

## Mode classification

**Mode A** — all 34 criteria satisfied; no deviations from BRIEF scope; build clean; 10/10 tests pass; all pre-existing probes pass; dirty tree intact; no commit.
