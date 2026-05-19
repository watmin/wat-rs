# Arc 214 Slice 3 — Stone A — EXPECTATIONS

## Independent prediction

- **Runtime band:** 25-40 min Mode A. Smaller than Slice 2 (Slice 2 had Sender + Receiver + Select + Clone + try_recv + len + close + trait impls = 8 concerns; Stone A has Sender + Receiver + pair = 3 concerns, no traits, no cascade). io-uring API is new but the BRIEF spells out the exact opcode + user_data pattern; sonnet copies skeleton.
- **LOC changed:** ~250-330 total (~150-180 in `src/comms/process.rs`; ~120-150 in `tests/probe_comms_process.rs`; +5 lines in `src/comms/mod.rs`; +1 line in `Cargo.toml`)
- **New files:** 3 (`src/comms/process.rs`, `tests/probe_comms_process.rs`, SCORE doc)
- **Surprises expected:** MEDIUM. The io-uring crate is new substrate; possible API misuse around SQE submission lifetime, CQE drain semantics, or OwnedFd ownership rules. The BRIEF skeleton pre-empts the obvious traps; residual risk is in details sonnet must compile-check against the actual io-uring 0.7 API.

## Honest-delta watch

### Risk 1 — io-uring crate API drift

**What:** The BRIEF's code uses `opcode::Read::new(types::Fd(fd), ptr, len).build().user_data(u)` and `ring.submission().push(&entry)`. io-uring 0.7's actual API may differ slightly (e.g., the `types::Fd` newtype shape, the `.user_data(u)` vs `.user_data(u as u64)` cast, the `unsafe { ... .push(...) }` exact placement).

**Mitigation:** Sonnet compiles each call site; cargo error messages will identify any drift; sonnet adjusts in place. If sonnet can't resolve a compile error in ≤5 min, STOP and report.

### Risk 2 — SQE submission lifetime / SAFETY

**What:** `opcode::Read::new` takes a raw pointer to `buf`. The pointer must be valid until `submit_and_wait` returns (the kernel writes through it). If `buf` is dropped before the wait, it's UB. The BRIEF places `buf` on the function stack ABOVE the submission + wait; Rust's stack discipline keeps it alive — but sonnet must NOT move `buf` into a temp that's dropped early.

**Mitigation:** The BRIEF SAFETY comment names this constraint explicitly. Sonnet should read the SAFETY comment as load-bearing, not decorative.

### Risk 3 — CQE drain pattern

**What:** After `submit_and_wait(1)`, `ring.completion().next()` returns ONE CQE. Stone A only submits ONE op per loop iteration, so one CQE is correct. If sonnet writes the loop wrong (e.g., submits one op but drains zero, or drains in a place that re-borrows the ring incorrectly), it'll either hang (un-drained CQE accumulates) or panic (re-borrow violation).

**Mitigation:** The BRIEF's `loop { ... }` body shows the exact one-shot pattern. Sonnet copies.

### Risk 4 — EOF detection on Read

**What:** `Read` opcode returns the number of bytes read in the CQE's `result()` field. `0` means EOF (peer closed write-end); negative means I/O error. If sonnet treats `0` as a successful read of 0 bytes (looping back to read more), the recv hangs forever (the pipe is dead).

**Mitigation:** The BRIEF skeleton explicitly checks `if result == 0 { return Err(RecvError); }` after the negative-check.

### Risk 5 — Accumulator correctness across recv calls

**What:** The `take_frame` helper consumes `[0..=pos]` from the accumulator (including the `'\n'`), leaving `[pos+1..]` for the next call. If sonnet writes the split off-by-one (consumes `[0..pos]` without the `'\n'`, leaves the `'\n'` in the buffer; next recv() returns empty bytes), the FIFO test will fail.

**Mitigation:** The BRIEF spells out `split_off(pos + 1)` and the `.pop()` to remove the trailing `'\n'`. Probe test 5 (`accumulator_splits_two_frames_from_one_read`) catches this case explicitly.

### Risk 6 — OwnedFd ownership rules

**What:** `OwnedFd::from_raw_fd` takes ownership of the raw fd. Calling it twice on the same fd is UB (double-close on Drop). The BRIEF skeleton wraps `fds[0]` and `fds[1]` each ONCE in `pair()`. Sonnet must not call `OwnedFd::from_raw_fd` elsewhere on the same descriptors.

**Mitigation:** The BRIEF SAFETY comment in `pair()` names this constraint. Stone A has no clone path (Stone D) so the double-wrap risk is bounded to `pair()` itself.

### Risk 7 — EINTR retry in write loop

**What:** `libc::write` can return `-1` with `errno == EINTR` (signal interruption). The BRIEF skeleton retries on EINTR via `if err.kind() == std::io::ErrorKind::Interrupted { continue; }`. If sonnet omits this, signal-interrupted writes return spurious SendError.

**Mitigation:** The BRIEF skeleton includes the EINTR retry inline.

### Risk 8 — Large-message test (probe 6) flakiness

**What:** The 10_000-byte send may exceed the kernel's atomic-write threshold (PIPE_BUF = 4096); the write splits across libc::write calls (but the BRIEF's write loop handles this). The receiver's io_uring reads may yield 4096 bytes at a time; the BRIEF's recv loop handles this. The probe should be deterministic — but if sonnet's write or read loop has a bug, this is where it surfaces.

**Mitigation:** Probe 6 is the integration test for the loop logic. If it fails, the failure points at write or read loop correctness, not at flakiness.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `Cargo.toml` adds `io-uring = "0.7"` in `[dependencies]` | YES |
| 2 | `cargo build --release` clean (no new warnings; io-uring resolves) | YES |
| 3 | `src/comms/process.rs` minted with module-level cascade-NOT-wired-in-Stone-A doc | YES |
| 4 | `Sender` newtype with private `write_fd: OwnedFd` field | YES |
| 5 | `Sender::send(&[u8])` writes payload + `'\n'` via `libc::write` retry loop | YES |
| 6 | `Sender::send` returns `Err(SendError(bytes.to_vec()))` on write failure | YES |
| 7 | `Sender::send` retries on EINTR (signal interruption) | YES |
| 8 | `Sender` is NOT Clone (Stone D adds Clone) | YES |
| 9 | `Sender` does NOT implement `CommSender<T>` trait (Stone D adds) | YES |
| 10 | `Receiver` newtype with private `read_fd: OwnedFd` + `accumulator: RefCell<Vec<u8>>` fields | YES |
| 11 | `Receiver::recv() -> Result<Vec<u8>, RecvError>` checks accumulator first | YES |
| 12 | `Receiver::recv` uses per-call `IoUring::new(2)` (Stone A; Stone E persistifies) | YES |
| 13 | `Receiver::recv` uses `opcode::Read::new(types::Fd(...), buf.as_mut_ptr(), len)` | YES |
| 14 | `Receiver::recv` returns `Err(RecvError)` on CQE result `< 0` OR `== 0` (EOF) | YES |
| 15 | `Receiver::recv` loops until accumulator yields a complete frame | YES |
| 16 | `Receiver` is NOT Clone (Stone D adds Clone) | YES |
| 17 | `Receiver` does NOT implement `CommReceiver<T>` trait (Stone D adds) | YES |
| 18 | `Receiver` is NOT cascade-aware (Stone B wires SHUTDOWN_BROADCAST_READ_FD) | YES |
| 19 | `take_frame` correctly splits at first `'\n'`; consumes the newline; preserves suffix | YES |
| 20 | `pair() -> std::io::Result<(Sender, Receiver)>` via `libc::pipe(2)` + OwnedFd wrap | YES |
| 21 | `pair()` does NOT call `OwnedFd::from_raw_fd` more than once per descriptor | YES |
| 22 | `pub mod process;` added to `src/comms/mod.rs` mirroring `pub mod thread;` shape | YES |
| 23 | `tests/probe_comms_process.rs` minted with 6 smoke tests | YES |
| 24 | All 6 probe tests PASS | YES |
| 25 | `cargo test --release --test probe_comms_thread` 10/10 PASS unchanged | YES |
| 26 | `cargo test --release --test probe_comms_foundation` 3/3 PASS unchanged (Slice 1) | YES |
| 27 | `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged (χ-1) | YES |
| 28 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged (α) | YES |
| 29 | Zero modifications outside the 4-file scope (`Cargo.toml` +1 line, `src/comms/mod.rs` +5 lines, `src/comms/process.rs` new, `tests/probe_comms_process.rs` new, SCORE doc new) | YES |
| 30 | Dirty tree intact (`src/fork.rs` + `src/spawn_process.rs` untouched) | YES |
| 31 | `src/typed_channel.rs` untouched (existing PipeFd preserved for Slice 5) | YES |
| 32 | NO `wat_arc170_program_contracts` re-run (per `feedback_no_hang_vector_in_additive_scorecard`) | YES |
| 33 | NO cascade implementation (Stone B territory) | YES |
| 34 | NO generic `T: HolonRepresentable` (Stone C territory) | YES |
| 35 | NO `try_recv` / `Select` / Clone / close / len / trait impls (Stone D territory) | YES |
| 36 | NO config tunable (Stone E territory) | YES |
| 37 | Every public item has a doc comment (gaze L2 pre-emption) | YES |
| 38 | All tests have real assertions (no bare `_`-bindings without follow-up) (gaze L1 pre-emption) | YES |
| 39 | `unsafe` blocks have SAFETY comments naming the FFI/lifetime invariant (forge pre-emption) | YES |
| 40 | NO commit (orchestrator owns the commit after ward pass) | YES |

## Mode classification

- **Mode A:** all 40 criteria satisfied; Stone A shipped clean; ward pass discipline applied at construction
- **Mode B (acceptable; honest surface):**
  - Risk 1 fires (io-uring API drift): sonnet adjusts ≤5 min and continues; SCORE notes the adjustment
  - Risk 2 fires (SAFETY-lifetime confusion): sonnet stops + reports; orchestrator decides fix
  - Risk 4 fires (EOF handling): probe 4 catches it; sonnet fixes
  - One probe test fails: sonnet investigates per failure mode; reports honestly
- **Mode C (failure):**
  - Touched any file outside the 4-file scope
  - Touched `src/typed_channel.rs` or the dirty tree
  - Ran `wat_arc170_program_contracts`
  - Committed the work
  - Implemented cascade / generics / Select / try_recv / Clone / close / len / traits / config (Stones B-E territory)
  - Added Mutex / RwLock / CondVar (ZERO-MUTEX violation; RefCell is fine)

## Calibration metadata

- **Orchestrator confidence:** HIGH on first-attempt Mode A. The BRIEF spells out the exact io-uring opcode pattern; the SAFETY/EINTR/EOF traps are pre-empted via inline code; the test set is precise. Sonnet's main work is mechanical assembly + 6 test bodies.
- **Risk factors:**
  - io-uring API micro-drift (Risk 1) — mitigated by sonnet adjusting on cargo errors
  - SQE-buffer lifetime SAFETY (Risk 2) — mitigated by explicit SAFETY comment + stack-allocated buf
  - Accumulator off-by-one (Risk 5) — mitigated by explicit `split_off(pos + 1)` + `.pop()` + probe test 5
- **Why this matters:** Stone A is the FIRST contact with io-uring in our substrate. If Stone A lands clean, Stones B-E have a proven foundation. If Stone A misuses io-uring, the misuse compounds across Stones B-E and the per-stone trust gate's value compounds inversely (the gate catches the cascade early). Per kernel-impeccability protocol: a stone is impeccable or it doesn't ship.

## Ward pass prediction

Per the kernel-impeccability protocol established in INTERSTITIAL § 2026-05-19 + applied to Slices 1 + 2: after SCORE verification, 5 wards (gaze + forge + reap + sever + temper) spawn in parallel.

Predicted findings:
- **gaze:** 0-1 (mostly clean; possible mumble on `take_frame` helper name or a local variable like `framed`)
- **forge:** 0-2 (possible candidate-rune on SAFETY annotations being insufficiently honest about io-uring crate's internal guarantees; possible Hickey lens flag on Sender/Receiver not being typestate-encoded)
- **reap:** 0 (Stone A scope is tightly bounded; honest-delta would self-flag in SCORE)
- **sever:** 0 (Sender / Receiver / take_frame / pair are 4 cleanly separated concerns)
- **temper:** 0-2 (per-call IoUring::new is wasteful — explicitly declared known-deferred to Stone E in BRIEF; ward may flag with deferral acknowledged)

Total predicted: 0-5 findings; most are L2 or rare-path. Round 2 should be CLEAN.

## Tractability tiebreaker rationale

Stone A is gated on Slice 2 (no overlap; different file). No alternative ordering within Slice 3 — Stones B/C/D/E all depend on Stone A's `Sender`/`Receiver`/`pair` shape. Within Stone A: ONE coherent concern (io-uring + pipe + bytes); decomposed into Sender/Receiver/pair/take_frame/probe at the IMPLEMENTATION level but logically ONE concern at the stepping-stone level. No further splitting needed.

## Cross-references

- BRIEF-214-SLICE-3A-IO-URING-BYTES.md — this stone's work order
- `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` — full arc 214 design
- WARD-PASS-1-FOUNDATION-PRIMITIVES.md — Slice 1 ward round-trip; lessons pre-empted
- WARD-PASS-2-THREAD-TIER.md — Slice 2 ward round-trip; 5-ward protocol applied
- `src/comms/thread.rs` — Slice 2 thread tier reference shape (mirror at structural level; underlying mechanism differs)
- `src/typed_channel.rs:324-388` — existing PipeFd READ-ONLY reference
- `src/runtime.rs:201` — `SHUTDOWN_BROADCAST_READ_FD` (Stone B will use; Stone A ignores)
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol
- `feedback_no_hang_vector_in_additive_scorecard` — verification discipline
- `feedback_defect_fix_or_panic_never_revert` — dirty tree preservation
- `feedback_iterative_complexity` — why 5 stones in Slice 3 (defended 2026-05-19 in conversation)
