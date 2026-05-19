# Arc 214 Slice 3 — Stone B — SCORE: Cascade-Aware Multi-Arm POLL_ADD

**Date:** 2026-05-19
**Mode:** A (all 37 criteria satisfied)
**Actual runtime:** ~15 min (inside predicted 15-25 min; BRIEF skeleton + Stone A pattern carried over directly; zero API drift)

---

## Scorecard — 37 rows

| # | Criterion | Expected | Actual |
|---|---|---|---|
| 1 | Module-level "Cascade contract (NOT WIRED IN STONE A)" section replaced with "Cascade contract (Stone B)" section explaining cascade-aware multi-arm | YES | YES |
| 2 | `PollOutcome` enum added with `DataReady` + `Shutdown` variants + doc comment | YES | YES |
| 3 | `wait_for_data_or_cascade` private fn added with full doc comment | YES | YES |
| 4 | `wait_for_data_or_cascade` uses `IoUring::new(4)` (room for 2 POLL_ADD + headroom) | YES | YES |
| 5 | `wait_for_data_or_cascade` uses `opcode::PollAdd` with event mask `(libc::POLLIN \| libc::POLLHUP) as u32` for data fd | YES | YES |
| 6 | `wait_for_data_or_cascade` uses `opcode::PollAdd` with event mask `libc::POLLHUP as u32` for broadcast fd | YES | YES |
| 7 | `wait_for_data_or_cascade` distinguishes the two arms via distinct `user_data` tokens (`DATA_TOKEN = 1`, `BROAD_TOKEN = 2`) | YES | YES |
| 8 | `wait_for_data_or_cascade` drains ALL ready CQEs via `while let Some(cqe) = ring.completion().next()` | YES | YES |
| 9 | `wait_for_data_or_cascade` returns `Shutdown` when `got_broadcast` (broadcast wins ties; substrate-invariant) | YES | YES |
| 10 | `wait_for_data_or_cascade` returns `DataReady` when ONLY `got_data` (no broadcast) | YES | YES |
| 11 | `wait_for_data_or_cascade` returns `Err(RecvError)` on `cqe.result() < 0` or empty drain (defensive) | YES | YES |
| 12 | `wait_for_data_or_cascade` SAFETY comment names fd-ownership-elsewhere + lifetime invariant | YES | YES |
| 13 | `Receiver::recv` keeps the fast-path accumulator check at the top (unchanged from Stone A) | YES | YES |
| 14 | `Receiver::recv` loads `SHUTDOWN_BROADCAST_READ_FD` once at the top via `Ordering::SeqCst` (NOT per loop iteration) | YES | YES |
| 15 | `Receiver::recv` cascade-poll step guarded by `if broadcast_fd >= 0` (bootstrap fallback) | YES | YES |
| 16 | `Receiver::recv` on `PollOutcome::Shutdown` returns `Err(RecvError)` | YES | YES |
| 17 | `Receiver::recv` on `PollOutcome::DataReady` falls through to the Read step | YES | YES |
| 18 | `Receiver::recv` Read step preserved verbatim from Stone A (per-call IoUring::new(2), opcode::Read, EOF detection) | YES | YES |
| 19 | `Receiver::recv` accumulator extend + take_frame check at end of loop unchanged from Stone A | YES | YES |
| 20 | Sender unchanged | YES | YES |
| 21 | take_frame unchanged | YES | YES |
| 22 | pair unchanged | YES | YES |
| 23 | NO new probe tests added | YES | YES |
| 24 | `cargo build --release` clean (no new warnings) | YES | YES |
| 25 | `cargo test --release --test probe_comms_process` 6/6 PASS unchanged | YES | YES |
| 26 | `cargo test --release --test probe_comms_thread` 10/10 PASS unchanged | YES | YES |
| 27 | `cargo test --release --test probe_comms_foundation` 3/3 PASS unchanged | YES | YES |
| 28 | `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged | YES | YES |
| 29 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged | YES | YES |
| 30 | Zero modifications outside `src/comms/process.rs` (mod.rs untouched; Cargo.toml untouched; tests/ untouched) | YES | YES |
| 31 | Dirty tree intact (`src/fork.rs` + `src/spawn_process.rs` untouched) | YES | YES |
| 32 | `src/typed_channel.rs` untouched | YES | YES |
| 33 | NO `wat_arc170_program_contracts` re-run | YES | YES |
| 34 | NO Stone C / D / E work (generics, try_recv, Select, Clone, close, len, traits, persistent ring, config tunable) | YES | YES |
| 35 | Every new item (PollOutcome enum + wait_for_data_or_cascade) has a doc comment | YES | YES |
| 36 | New `unsafe` block has a SAFETY comment naming the invariant | YES | YES |
| 37 | NO commit (orchestrator owns the commit after ward pass) | YES | YES |

---

## Honest-delta watch — Risks 1-8

### Risk 1 — io-uring PollAdd opcode API

**Predicted:** possible API micro-drift in `opcode::PollAdd::new(types::Fd(fd), poll_mask_u32).build().user_data(u64)`.

**Actual:** CLEAN. `io-uring v0.7.12` (already locked from Stone A). The BRIEF's exact call signatures compiled without adjustment:
- `opcode::PollAdd::new(types::Fd(read_fd), (libc::POLLIN | libc::POLLHUP) as u32).build().user_data(DATA_TOKEN)` — compiles directly.
- `opcode::PollAdd::new(types::Fd(broadcast_fd), libc::POLLHUP as u32).build().user_data(BROAD_TOKEN)` — compiles directly.
- `unsafe { ring.submission().push(&poll_data).map_err(|_| RecvError)?; }` — compiles; same push API as Stone A's Read opcode.
- `ring.submit_and_wait(1)` + `ring.completion().next()` + `cqe.result()` + `cqe.user_data()` — all compiles; identical API surface to Stone A.

Zero drift from BRIEF skeleton.

### Risk 2 — CQE drain after multi-arm submission

**Predicted:** possible `if let Some` single-drain instead of `while let Some` drain; missed broadcast detection.

**Actual:** CLEAN. `while let Some(cqe) = ring.completion().next()` drain loop implemented exactly per BRIEF. Both arms may fire simultaneously; the loop drains all ready CQEs before evaluating `got_broadcast` / `got_data`. Doc comment names the simultaneous-fire case explicitly.

### Risk 3 — Broadcast-wins-ties enforcement

**Predicted:** possible data-first ordering instead of broadcast-first.

**Actual:** CLEAN. `if got_broadcast { Ok(PollOutcome::Shutdown) } else if got_data { Ok(PollOutcome::DataReady) }` — broadcast checked first, matching typed_channel.rs:360-364 discipline. Comment names the substrate-invariant rationale ("process is going down; honest reporting").

### Risk 4 — Defensive return on empty CQE drain

**Predicted:** possible panic or infinite-loop on empty drain.

**Actual:** CLEAN. Final `Err(RecvError)` defensive return implemented. Comment explains: "submit_and_wait(1) returned but no CQE drained — defensive. Should not happen with min_complete=1; if it does, treat as transient and let the caller retry via its loop."

### Risk 5 — Bootstrap fallback path

**Predicted:** possible unconditional cascade-poll with broadcast_fd = -1.

**Actual:** CLEAN. `if broadcast_fd >= 0 { match wait_for_data_or_cascade(...) ... }` guard implemented. When broadcast_fd is -1, the entire cascade-poll block is skipped; the loop falls through directly to the Read step (Stone A behavior). The 6 probe tests run in a test environment where `SHUTDOWN_BROADCAST_READ_FD` is -1 (uninitialized), so they exercise the bootstrap fallback path and all 6 pass — confirming the fallback is live and correct.

### Risk 6 — Read step preservation (regression risk)

**Predicted:** possible rewrite of Stone A's Read step from scratch.

**Actual:** CLEAN. Stone A's Read step is preserved verbatim (per-call `IoUring::new(2)`, `opcode::Read`, buf = [0u8; 4096], SAFETY comment, submit_and_wait(1), cqe.result() checks, accumulator extend, take_frame check). The only changes in `Receiver::recv` are: (a) updated doc comment reflecting Stone B cascade-aware status, (b) broadcast_fd load before the loop, (c) read_fd extracted to a local before the loop, (d) cascade-aware pre-poll block prepended inside the loop. The Read step's body is structurally identical to Stone A.

### Risk 7 — Module doc reflecting cascade-WIRED status

**Predicted:** possible forgotten doc update.

**Actual:** CLEAN. "Cascade contract (NOT WIRED IN STONE A)" section replaced with "Cascade contract (Stone B)" section explaining: multi-arm POLL_ADD, broadcast-wins-ties, event masks, bootstrap fallback. The `pub fn recv` doc comment also updated from "Stone A: NOT cascade-aware. ... Stone B wires the broadcast arm." to "Stone B: cascade-aware. ... Bootstrap fallback: when the broadcast fd is -1 ... Stone E persistifies the per-call IoUring."

### Risk 8 — Other parts of process.rs preserved unchanged

**Predicted:** possible accidental reformatting of Sender / take_frame / pair.

**Actual:** CLEAN. Git diff shows zero changes to Sender (impl block), take_frame, or pair. The only edits are: module-level doc (cascade section replacement), `PollOutcome` enum + `wait_for_data_or_cascade` helper added above take_frame, and `Receiver::recv` body refactored as specified.

---

## One beyond-scope addition (honest-delta)

**None.** Stone B's deliverables are mechanical assembly from the BRIEF skeleton. Zero additions beyond scope.

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
   Compiling interrogate-example v0.1.0 (...)
   Compiling with-lru-example v0.1.0 (...)
   Compiling console-demo v0.1.0 (...)
   Compiling with-loader-example v0.1.0 (...)
    Finished `release` profile [optimized] target(s) in 18.42s
```

5 pre-existing warnings (check.rs:11194, runtime.rs:18229, 18305, 18684, 18693). ZERO new warnings from Stone B changes.

---

## Test outputs (verbatim)

### probe_comms_process (Slice 3 — cascade-aware data path)

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

Note: All 6 tests exercise the bootstrap-fallback path (SHUTDOWN_BROADCAST_READ_FD is -1 in the test environment), confirming the `if broadcast_fd >= 0` guard works correctly and the bare Read step is unaffected by the cascade-aware refactor.

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

`git diff --name-only` shows only `src/comms/process.rs` modified. New untracked file: `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md`.

Pre-existing dirty tree (NOT touched): `src/fork.rs`, `src/spawn_process.rs`.
NOT committed (per BRIEF; orchestrator commits after ward pass).

---

## Ward pass prediction update

Pre-emptive discipline applied at construction:

1. Every public item keeps its doc comment — YES. `Receiver::recv` doc updated to reflect Stone B cascade-aware status. No new public items added (PollOutcome + wait_for_data_or_cascade are private).
2. Private items have doc comments too — YES. `PollOutcome` enum has a doc comment. Both variants (`DataReady`, `Shutdown`) have doc comments naming intent. `wait_for_data_or_cascade` has a full doc comment naming: purpose, per-call IoUring rationale, event-mask convention reference, return semantics.
3. Comments explain WHY not WHAT — YES. "Broadcast wins ties — substrate is going down; honest reporting (mirrors typed_channel.rs:360-364 discipline)." "Bootstrap fallback: when broadcast_fd is -1 (pre-init or test bypass), skip the poll and fall through to bare Read." Both explain the substrate invariant, not the mechanical check.
4. SAFETY comment at every unsafe block — YES. The `unsafe { ring.submission().push(...) }` block in `wait_for_data_or_cascade` has a SAFETY comment: "both SQEs reference fds owned elsewhere (read_fd by the Receiver; broadcast_fd by the substrate worker). Both remain valid for the lifetime of this submit_and_wait call."
5. Event masks match existing PipeFd convention — YES. `(libc::POLLIN | libc::POLLHUP) as u32` for data fd; `libc::POLLHUP as u32` for broadcast fd. Matches typed_channel.rs:329-368 discipline exactly.
6. Stone A Read step preserved verbatim — YES. Only changes in the recv body are the broadcast_fd load, read_fd local, and the cascade pre-poll block. The Read step body is structurally identical.

Predicted findings:
- **temper:** 0-1 (per-call IoUring::new(4) in wait_for_data_or_cascade is explicitly deferred to Stone E; ward should acknowledge the deferral note in the doc comment)
- **gaze:** 0-1 (possible mumble on DATA_TOKEN/BROAD_TOKEN const naming — they're function-scoped consts, not module-level; acceptable per Stone B scope)
- **forge:** 0-1 (SAFETY comment quality; possible candidate-rune on wait_for_data_or_cascade taking RawFd args instead of OwnedFd refs — but RawFd is correct here since the fds are owned by caller scopes and Stone D migrates to trait-based shapes)
- **reap:** 0 (Stone B scope is tightly bounded; zero honest-delta)
- **sever:** 0 (PollOutcome + wait_for_data_or_cascade form one concern; Receiver::recv is one concern; clean separation)

Total predicted: 0-3 findings; all L2 at most. Round 2 should be CLEAN.

---

## Mode classification

**Mode A** — all 37 criteria satisfied; zero new warnings; 6/6 probe_comms_process tests pass (bootstrap-fallback path confirmed live); all 4 prior probe suites unchanged; dirty tree intact; no commit.
