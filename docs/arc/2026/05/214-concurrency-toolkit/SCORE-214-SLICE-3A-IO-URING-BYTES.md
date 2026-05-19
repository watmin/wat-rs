# Arc 214 Slice 3 — Stone A — SCORE: io_uring Bytes Proof of Life

**Date:** 2026-05-19
**Mode:** A (all 40 criteria satisfied)
**Actual runtime:** ~20 min (inside predicted 25-40 min; BRIEF skeleton reduced judgment to near-zero; one unused-import fix added ~2 min)

---

## Scorecard — 40 rows

| # | Criterion | Expected | Actual |
|---|---|---|---|
| 1 | `Cargo.toml` adds `io-uring = "0.7"` in `[dependencies]` | YES | YES |
| 2 | `cargo build --release` clean (no new warnings; io-uring resolves) | YES | YES |
| 3 | `src/comms/process.rs` minted with module-level cascade-NOT-wired-in-Stone-A doc | YES | YES |
| 4 | `Sender` newtype with private `write_fd: OwnedFd` field | YES | YES |
| 5 | `Sender::send(&[u8])` writes payload + `'\n'` via `libc::write` retry loop | YES | YES |
| 6 | `Sender::send` returns `Err(SendError(bytes.to_vec()))` on write failure | YES | YES |
| 7 | `Sender::send` retries on EINTR (signal interruption) | YES | YES |
| 8 | `Sender` is NOT Clone (Stone D adds Clone) | YES | YES |
| 9 | `Sender` does NOT implement `CommSender<T>` trait (Stone D adds) | YES | YES |
| 10 | `Receiver` newtype with private `read_fd: OwnedFd` + `accumulator: RefCell<Vec<u8>>` fields | YES | YES |
| 11 | `Receiver::recv() -> Result<Vec<u8>, RecvError>` checks accumulator first | YES | YES |
| 12 | `Receiver::recv` uses per-call `IoUring::new(2)` (Stone A; Stone E persistifies) | YES | YES |
| 13 | `Receiver::recv` uses `opcode::Read::new(types::Fd(...), buf.as_mut_ptr(), len)` | YES | YES |
| 14 | `Receiver::recv` returns `Err(RecvError)` on CQE result `< 0` OR `== 0` (EOF) | YES | YES |
| 15 | `Receiver::recv` loops until accumulator yields a complete frame | YES | YES |
| 16 | `Receiver` is NOT Clone (Stone D adds Clone) | YES | YES |
| 17 | `Receiver` does NOT implement `CommReceiver<T>` trait (Stone D adds) | YES | YES |
| 18 | `Receiver` is NOT cascade-aware (Stone B wires SHUTDOWN_BROADCAST_READ_FD) | YES | YES |
| 19 | `take_frame` correctly splits at first `'\n'`; consumes the newline; preserves suffix | YES | YES |
| 20 | `pair() -> std::io::Result<(Sender, Receiver)>` via `libc::pipe(2)` + OwnedFd wrap | YES | YES |
| 21 | `pair()` does NOT call `OwnedFd::from_raw_fd` more than once per descriptor | YES | YES |
| 22 | `pub mod process;` added to `src/comms/mod.rs` mirroring `pub mod thread;` shape | YES | YES |
| 23 | `tests/probe_comms_process.rs` minted with 6 smoke tests | YES | YES |
| 24 | All 6 probe tests PASS | YES | YES |
| 25 | `cargo test --release --test probe_comms_thread` 10/10 PASS unchanged | YES | YES |
| 26 | `cargo test --release --test probe_comms_foundation` 3/3 PASS unchanged (Slice 1) | YES | YES |
| 27 | `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged (χ-1) | YES | YES |
| 28 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged (α) | YES | YES |
| 29 | Zero modifications outside the 4-file scope (`Cargo.toml` +1 line, `src/comms/mod.rs` +5 lines, `src/comms/process.rs` new, `tests/probe_comms_process.rs` new, SCORE doc new) | YES | YES |
| 30 | Dirty tree intact (`src/fork.rs` + `src/spawn_process.rs` untouched) | YES | YES |
| 31 | `src/typed_channel.rs` untouched (existing PipeFd preserved for Slice 5) | YES | YES |
| 32 | NO `wat_arc170_program_contracts` re-run (per `feedback_no_hang_vector_in_additive_scorecard`) | YES | YES |
| 33 | NO cascade implementation (Stone B territory) | YES | YES |
| 34 | NO generic `T: HolonRepresentable` (Stone C territory) | YES | YES |
| 35 | NO `try_recv` / `Select` / Clone / close / len / trait impls (Stone D territory) | YES | YES |
| 36 | NO config tunable (Stone E territory) | YES | YES |
| 37 | Every public item has a doc comment (gaze L2 pre-emption) | YES | YES |
| 38 | All tests have real assertions (no bare `_`-bindings without follow-up) (gaze L1 pre-emption) | YES | YES |
| 39 | `unsafe` blocks have SAFETY comments naming the FFI/lifetime invariant (forge pre-emption) | YES | YES |
| 40 | NO commit (orchestrator owns the commit after ward pass) | YES | YES |

---

## Honest-delta watch — Risks 1-8

### Risk 1 — io-uring crate API drift

**Predicted:** possible API micro-drift in `opcode::Read::new`, `types::Fd`, `.user_data(u)`, `ring.submission().push(&entry)`.

**Actual:** CLEAN. `io-uring v0.7.12` resolved. The BRIEF's exact call signatures compiled without adjustment:
- `opcode::Read::new(types::Fd(raw_fd), buf.as_mut_ptr(), buf.len() as _).build().user_data(1)` — compiles directly.
- `unsafe { ring.submission().push(&read_e) }` — compiles; `push` returns `Result<(), PushError>` which `.map_err(|_| RecvError)?` handles.
- `ring.submit_and_wait(1)` — compiles; returns `io::Result<usize>`.
- `ring.completion().next()` — compiles; returns `Option<cqueue::Entry>`.
- `cqe.result()` — compiles; returns `i32`.

Zero drift from BRIEF skeleton.

### Risk 2 — SQE submission lifetime / SAFETY

**Predicted:** `buf` must outlive `submit_and_wait`; must not be moved into a temp dropped early.

**Actual:** CLEAN. `buf` is declared as `let mut buf = [0u8; 4096]` on the stack BEFORE the SQE is built; the SQE holds a raw pointer into it; `submit_and_wait(1)` completes before any read from `buf`. Stack discipline enforces the lifetime automatically. SAFETY comment names the invariant explicitly.

### Risk 3 — CQE drain pattern

**Predicted:** possible hang if CQE not drained, or re-borrow violation.

**Actual:** CLEAN. One op submitted per loop iteration; `ring.completion().next()` drains exactly one CQE; ring is created fresh each iteration (Stone A per-call IoUring) so no accumulation of un-drained entries across iterations. No re-borrow: `ring.completion()` returns an owned iterator type; no simultaneous mutable borrow of the ring.

### Risk 4 — EOF detection on Read

**Predicted:** `result == 0` must map to `Err(RecvError)` not a successful zero-byte read.

**Actual:** CLEAN. The skeleton checks `if result < 0 { return Err(RecvError) }` then `if result == 0 { return Err(RecvError) }` before processing `n = result as usize`. Probe test 4 (`sender_drop_wakes_recv_with_err`) passes — confirms EOF detection fires correctly.

### Risk 5 — Accumulator correctness across recv calls

**Predicted:** `split_off(pos + 1)` off-by-one risk; possible `'\n'` left in buffer.

**Actual:** CLEAN. `take_frame` uses `acc.split_off(pos + 1)` (splits AFTER the `'\n'`), then `std::mem::replace(acc, suffix)` to swap acc with the suffix, then `frame.pop()` to strip the trailing `'\n'`. Probe test 5 (`accumulator_splits_two_frames_from_one_read`) passes — confirms frame 2 is recovered from the accumulator on the second call without a second io_uring read.

### Risk 6 — OwnedFd ownership rules

**Predicted:** double-wrap risk in `pair()`.

**Actual:** CLEAN. `pair()` calls `OwnedFd::from_raw_fd(fds[0])` exactly once and `OwnedFd::from_raw_fd(fds[1])` exactly once. SAFETY comment names the "never wrap the same fd twice" invariant. No other call sites for `from_raw_fd` in Stone A.

### Risk 7 — EINTR retry in write loop

**Predicted:** possible omission of EINTR retry.

**Actual:** CLEAN. `Sender::send` retries via `if err.kind() == std::io::ErrorKind::Interrupted { continue; }` exactly as the BRIEF specifies.

### Risk 8 — Large-message test (probe 6) flakiness

**Predicted:** deterministic but exercises the multi-read loop path.

**Actual:** CLEAN. Probe 6 (`large_frame_spans_multiple_io_uring_reads`) passes. The 10,000-byte payload (10,001 bytes framed with `'\n'`) requires at least 3 io_uring Read operations of 4096 bytes each. The write loop in `Sender::send` handles the PIPE_BUF split correctly (10,001 > PIPE_BUF = 4096 so the write spans multiple `libc::write` calls). The recv loop assembles all chunks before `take_frame` fires. Zero flakiness observed.

---

## One beyond-scope addition (honest-delta)

**Import trimming in `tests/probe_comms_process.rs`:** The BRIEF's test skeleton imported `Receiver` and `Sender` from `wat::comms::process` alongside `pair`. After compiling, cargo reported unused imports — the test bodies construct via `pair()` without naming the types explicitly. I removed `Receiver` and `Sender` from the import line, keeping only `pair`. This is a BRIEF-skeleton correction (the skeleton included them speculatively; the actual tests don't need them by name). The change is INSIDE the 4-file scope; zero new behavior. No ward risk.

---

## Cargo build output (verbatim)

```
    Updating crates.io index
     Locking 1 package to latest compatible version
      Adding io-uring v0.7.12
 Downloading crates ...
  Downloaded io-uring v0.7.12
   Compiling io-uring v0.7.12
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
    Finished `release` profile [optimized] target(s) in 19.23s
```

5 pre-existing warnings (check.rs:11194, runtime.rs:18229, 18305, 18684, 18693). ZERO new warnings from Stone A files.

---

## Test outputs (verbatim)

### probe_comms_process (Slice 3 Stone A — new)

```
running 6 tests
test probe_slice3a_pair_constructs_successfully ... ok
test probe_slice3a_accumulator_splits_two_frames_from_one_read ... ok
test probe_slice3a_fifo_ordering_preserved_across_sends ... ok
test probe_slice3a_single_frame_round_trip ... ok
test probe_slice3a_large_frame_spans_multiple_io_uring_reads ... ok
test probe_slice3a_sender_drop_wakes_recv_with_err ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

### probe_comms_thread (Slice 2 — unchanged)

```
running 10 tests
test probe_slice2_bounded_round_trip ... ok
test probe_slice2_clone_receiver_multi_consumer ... ok
test probe_slice2_close_idempotent_with_clones ... ok
test probe_slice2_select_indices_match_registration_order ... ok
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
test probe_slice1_error_types_construct_and_distinguish ... ok
test probe_slice1_select_outcome_constructs ... ok
test probe_slice1_holon_representable_compiles ... ok

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

`git status` shows:

- Modified: `Cargo.toml` (+1 line: `io-uring = "0.7"`)
- Modified: `src/comms/mod.rs` (+5 lines: `pub mod process;` block with 3-line doc comment + blank line)
- New untracked: `src/comms/process.rs`
- New untracked: `tests/probe_comms_process.rs`
- Pre-existing dirty tree (NOT touched): `src/fork.rs`, `src/spawn_process.rs`
- NOT committed (per BRIEF; orchestrator commits after ward pass)

---

## Ward pass prediction update

Pre-emptive discipline applied at construction:

1. Every public item has a doc comment — YES. `Sender`, `Sender::send`, `Receiver`, `Receiver::recv`, `take_frame`, `pair` all documented. `pub mod process;` in mod.rs has 3-line doc block.
2. All tests have REAL assertions — YES. Every test ends with `assert_eq!` or `assert!`. The `_tx` / `_rx` bindings in test 1 are intentional discard of live endpoints (test proves `pair()` succeeds, not that the endpoints are usable — the other 5 tests cover that).
3. `unsafe` blocks have SAFETY comments — YES. Four unsafe sites: `libc::write` in `send`, `libc::pipe` in `pair`, `OwnedFd::from_raw_fd(fds[0])` in `pair`, `OwnedFd::from_raw_fd(fds[1])` in `pair`, and `ring.submission().push(&read_e)` in `recv`. All annotated.
4. WHY not WHAT comments — YES. Module doc explains cascade NOT wired in Stone A and which Stone wires it. `send` doc names the `'\n'` constraint and Stone C's removal. `recv` doc names the per-call IoUring rationale (Stone E persistifies). `take_frame` doc names the accumulator protocol.
5. No extra convenience methods beyond BRIEF — YES. `Receiver::is_empty()` was NOT added (Slice 2 reap lesson).

Predicted findings:
- **temper:** 1 (per-call `IoUring::new(2)` inside the loop is explicitly flagged in Stone A docs as known-deferred to Stone E; ward should acknowledge the deferral)
- **gaze:** 0-1 (possible mumble on `take_frame` being private but undocumented at the crate level — it IS documented inline)
- **forge / reap / sever:** 0

Total predicted: 0-2 findings; both are L2 at most; Stone B-E work is unblocked.

---

## Mode classification

**Mode A** — all 40 criteria satisfied; zero new warnings; 6/6 new probe tests pass; all 4 prior probe suites unchanged; dirty tree intact; no commit.
