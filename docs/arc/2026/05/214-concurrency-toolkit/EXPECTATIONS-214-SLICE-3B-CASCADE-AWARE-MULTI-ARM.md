# Arc 214 Slice 3 — Stone B — EXPECTATIONS

## Independent prediction

- **Runtime band:** 15-25 min Mode A. Smaller than Stone A — Stone B adds one helper (`wait_for_data_or_cascade`) + one enum (`PollOutcome`) + refactors `Receiver::recv` to prepend a cascade-aware pre-poll. The BRIEF spells out the full code skeleton + the substrate event-mask convention. No new tests. Per-stone IoUring pattern from Stone A carries over.
- **LOC changed:** ~80-110 total (~70-90 added in `src/comms/process.rs`; ~10-15 modified in `Receiver::recv` + module doc).
- **New files:** 1 (SCORE doc only — no new code or test files; Stone B is a Receiver::recv refactor + 2 new private items in the same file).
- **Surprises expected:** LOW. The io_uring opcode pattern is identical to Stone A (just `PollAdd` instead of `Read`). The substrate event-mask convention is established in typed_channel.rs:329-368. The bootstrap fallback path is a simple `if broadcast_fd >= 0` branch.

## Honest-delta watch

### Risk 1 — io-uring PollAdd opcode API

**What:** The BRIEF uses `opcode::PollAdd::new(types::Fd(fd), poll_mask_u32).build().user_data(u64)`. io-uring 0.7's actual API may have slight signature drift on the poll mask type (could be `u32` directly or `PollFlags`). The BRIEF assumes `(libc::POLLIN | libc::POLLHUP) as u32` which matches the most common io-uring 0.7 shape.

**Mitigation:** Sonnet compiles each call; cargo error messages identify drift; adjust in place. If unresolved in ≤5 min, STOP.

### Risk 2 — CQE drain after multi-arm submission

**What:** After `submit_and_wait(1)`, both POLL_ADD CQEs may already be ready (both fds happen to be ready simultaneously). The `while let Some(cqe) = ring.completion().next()` loop drains ALL ready CQEs; this is correct. If sonnet writes `if let Some(cqe) = ring.completion().next()` (single CQE) instead, the unconsumed CQE stays in the ring — but the ring drops at end of function, so no leak; just a missed broadcast detection.

**Mitigation:** The BRIEF spells out the `while let Some(cqe)` pattern explicitly + names "both arms may fire simultaneously" in the doc comment.

### Risk 3 — Broadcast-wins-ties enforcement

**What:** The substrate discipline (typed_channel.rs:360-364) says "Shutdown wins ties." The BRIEF's `wait_for_data_or_cascade` body checks `got_broadcast` BEFORE `got_data`. If sonnet flips the order (checks `got_data` first), recv may continue reading data when it should report shutdown — a correctness bug under simultaneous-fire conditions.

**Mitigation:** The BRIEF skeleton spells out the exact `if got_broadcast { ... } else if got_data { ... }` order + the doc comment names the substrate-invariant rationale.

### Risk 4 — Defensive return on empty CQE drain

**What:** `submit_and_wait(1)` should always return with ≥1 CQE ready (min_complete=1 is the wait condition). If somehow neither arm fired (which shouldn't happen), the function returns `Err(RecvError)` defensively. If sonnet panics or infinite-loops here instead, a hypothetical kernel oddity becomes a hang.

**Mitigation:** The BRIEF's final `Err(RecvError)` defensive return is the correct pattern. Sonnet copies.

### Risk 5 — Bootstrap fallback path

**What:** When `broadcast_fd == -1` (atomic load returned the uninitialized sentinel), the BRIEF skips the cascade-poll step and falls through to bare Read (same as Stone A). If sonnet wires the cascade-poll unconditionally, recv would call `wait_for_data_or_cascade` with `broadcast_fd = -1`, which would submit POLL_ADD on fd -1 (invalid) → kernel returns -EBADF → CQE result < 0 → recv returns `Err(RecvError)` spuriously.

**Mitigation:** The BRIEF's `if broadcast_fd >= 0 { ... }` guard is the correct bootstrap fallback. Sonnet copies.

### Risk 6 — Read step preservation (regression risk)

**What:** Stone A's recv body has a working Read step (per-call IoUring + opcode::Read + EOF detection). Stone B should PRESERVE this verbatim and only PREPEND the cascade-aware pre-poll step. If sonnet rewrites the Read step from scratch (or moves it into a helper, or changes the buf size), regressions could surface.

**Mitigation:** The BRIEF says "Stone-A `Receiver::recv` code body is partially preserved, NOT rewritten from scratch — the Read step is verbatim." Sonnet's job is to add a cascade pre-poll, not to refactor Stone A's code.

### Risk 7 — Module doc reflecting cascade-WIRED status

**What:** Stone A's module doc has a "Cascade contract (NOT WIRED IN STONE A)" section. Stone B replaces this with a "Cascade contract (Stone B)" section explaining the new behavior. If sonnet forgets to update the doc, the file's contract claims diverge from the code.

**Mitigation:** The BRIEF explicitly lists module-doc-replacement as deliverable #1 + provides the exact replacement text.

### Risk 8 — Other parts of process.rs preserved unchanged

**What:** Sender + take_frame + pair are unchanged in Stone B. If sonnet accidentally touches any of these (e.g., reformats the Sender::send retry loop), the diff bloats and ward findings multiply.

**Mitigation:** The BRIEF's "ZERO modifications outside the listed scope" constraint + the deliverables list naming ONLY (a) module doc, (b) new PollOutcome/wait_for_data_or_cascade items, (c) Receiver::recv body. Sender/take_frame/pair untouched.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | Module-level "Cascade contract (NOT WIRED IN STONE A)" section replaced with "Cascade contract (Stone B)" section explaining cascade-aware multi-arm | YES |
| 2 | `PollOutcome` enum added with `DataReady` + `Shutdown` variants + doc comment | YES |
| 3 | `wait_for_data_or_cascade` private fn added with full doc comment | YES |
| 4 | `wait_for_data_or_cascade` uses `IoUring::new(4)` (room for 2 POLL_ADD + headroom) | YES |
| 5 | `wait_for_data_or_cascade` uses `opcode::PollAdd` with event mask `(libc::POLLIN \| libc::POLLHUP) as u32` for data fd | YES |
| 6 | `wait_for_data_or_cascade` uses `opcode::PollAdd` with event mask `libc::POLLHUP as u32` for broadcast fd | YES |
| 7 | `wait_for_data_or_cascade` distinguishes the two arms via distinct `user_data` tokens (`DATA_TOKEN = 1`, `BROAD_TOKEN = 2`) | YES |
| 8 | `wait_for_data_or_cascade` drains ALL ready CQEs via `while let Some(cqe) = ring.completion().next()` | YES |
| 9 | `wait_for_data_or_cascade` returns `Shutdown` when `got_broadcast` (broadcast wins ties; substrate-invariant) | YES |
| 10 | `wait_for_data_or_cascade` returns `DataReady` when ONLY `got_data` (no broadcast) | YES |
| 11 | `wait_for_data_or_cascade` returns `Err(RecvError)` on `cqe.result() < 0` or empty drain (defensive) | YES |
| 12 | `wait_for_data_or_cascade` SAFETY comment names fd-ownership-elsewhere + lifetime invariant | YES |
| 13 | `Receiver::recv` keeps the fast-path accumulator check at the top (unchanged from Stone A) | YES |
| 14 | `Receiver::recv` loads `SHUTDOWN_BROADCAST_READ_FD` once at the top via `Ordering::SeqCst` (NOT per loop iteration) | YES |
| 15 | `Receiver::recv` cascade-poll step guarded by `if broadcast_fd >= 0` (bootstrap fallback) | YES |
| 16 | `Receiver::recv` on `PollOutcome::Shutdown` returns `Err(RecvError)` | YES |
| 17 | `Receiver::recv` on `PollOutcome::DataReady` falls through to the Read step | YES |
| 18 | `Receiver::recv` Read step preserved verbatim from Stone A (per-call IoUring::new(2), opcode::Read, EOF detection) | YES |
| 19 | `Receiver::recv` accumulator extend + take_frame check at end of loop unchanged from Stone A | YES |
| 20 | Sender unchanged | YES |
| 21 | take_frame unchanged | YES |
| 22 | pair unchanged | YES |
| 23 | NO new probe tests added | YES |
| 24 | `cargo build --release` clean (no new warnings) | YES |
| 25 | `cargo test --release --test probe_comms_process` 6/6 PASS unchanged | YES |
| 26 | `cargo test --release --test probe_comms_thread` 10/10 PASS unchanged | YES |
| 27 | `cargo test --release --test probe_comms_foundation` 3/3 PASS unchanged | YES |
| 28 | `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged | YES |
| 29 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged | YES |
| 30 | Zero modifications outside `src/comms/process.rs` (mod.rs untouched; Cargo.toml untouched; tests/ untouched) | YES |
| 31 | Dirty tree intact (`src/fork.rs` + `src/spawn_process.rs` untouched) | YES |
| 32 | `src/typed_channel.rs` untouched | YES |
| 33 | NO `wat_arc170_program_contracts` re-run | YES |
| 34 | NO Stone C / D / E work (generics, try_recv, Select, Clone, close, len, traits, persistent ring, config tunable) | YES |
| 35 | Every new item (PollOutcome enum + wait_for_data_or_cascade) has a doc comment | YES |
| 36 | New `unsafe` block has a SAFETY comment naming the invariant | YES |
| 37 | NO commit (orchestrator owns the commit after ward pass) | YES |

## Mode classification

- **Mode A:** all 37 criteria satisfied; Stone B shipped clean
- **Mode B (acceptable; honest surface):**
  - Risk 1 fires (io-uring PollAdd API drift): sonnet adjusts ≤5 min and continues
  - Risk 2 fires (single-CQE drain): probe tests still pass (data path works); ward catches the missed-broadcast issue
  - Risk 4 fires (empty CQE drain → Err): unusual but defensive; SCORE notes
  - One probe test fails: sonnet investigates per failure mode; reports honestly
- **Mode C (failure):**
  - Touched any file outside `src/comms/process.rs` + SCORE doc
  - Touched `src/typed_channel.rs` or the dirty tree
  - Ran `wat_arc170_program_contracts`
  - Committed the work
  - Implemented Stone C / D / E territory
  - Added Mutex / RwLock / CondVar (ZERO-MUTEX violation)
  - Rewrote Sender or take_frame or pair from scratch

## Calibration metadata

- **Orchestrator confidence:** HIGH on first-attempt Mode A. Stone B is a surgical insertion of a cascade-aware pre-poll step ahead of an unchanged Read step. The BRIEF skeleton + the substrate event-mask convention + Stone A's per-call IoUring pattern all carry over directly. The risk surface is small (8 risks; mostly micro-API + correctness-discipline reminders).
- **Risk factors:**
  - io-uring PollAdd API micro-drift (Risk 1) — mitigated by sonnet adjusting on cargo errors
  - Broadcast-wins-ties ordering (Risk 3) — mitigated by explicit BRIEF + doc comment naming the substrate invariant
  - Bootstrap fallback guard (Risk 5) — mitigated by explicit `if broadcast_fd >= 0` in the BRIEF
- **Why this matters:** Stone B is the LOAD-BEARING cascade-aware step. Without it, the process tier hangs past substrate shutdown — a deadlock-class defect per `feedback_never_deadlock` doctrine. Stone B closes the contract the module-level doc has been promising since Stone A. After Stone B, the process tier matches Slice 2's thread-tier cascade discipline.

## Ward pass prediction

Per the kernel-impeccability protocol: after SCORE verification, 5 wards spawn in parallel.

Predicted findings:
- **gaze:** 0-1 (possible mumble on `DATA_TOKEN`/`BROAD_TOKEN` const naming; possible L2 on doc comment WHY-vs-WHAT)
- **forge:** 0-1 (SAFETY comment quality; possible candidate-rune on `wait_for_data_or_cascade` being a side-effecting fn that could be `&self` method on Receiver)
- **reap:** 0 (Stone B scope is tightly bounded; no honest-delta expected)
- **sever:** 0 (wait_for_data_or_cascade is a separate concern from Receiver::recv; PollOutcome is its own concern; clean separation)
- **temper:** 0-1 (per-call IoUring::new(4) explicitly known-deferred to Stone E; ward acknowledges; possible flag on the `as u32` cast cost — but it's a free compile-time cast)

Total predicted: 0-3 findings; most are L2. Round 2 should be CLEAN.

## Tractability tiebreaker rationale

Stone B is gated on Stone A. No alternative ordering within Slice 3 — Stones C/D/E all depend on Stone B's cascade-aware shape (Stone D's Select needs cascade-arm registration; Stone C's HolonRepresentable serialization composes over cascade-aware recv).

Within Stone B: ONE coherent concern (cascade-aware multi-arm wait). Decomposed into PollOutcome enum + wait_for_data_or_cascade helper + Receiver::recv refactor at the IMPLEMENTATION level but logically ONE concern at the stepping-stone level.

## Cross-references

- BRIEF-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md — this stone's work order
- BRIEF-214-SLICE-3A-IO-URING-BYTES.md — Stone A foundation
- WARD-PASS-3A-IO-URING-BYTES.md — Stone A ward round-trip
- `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` — Slice 3 cascade contract
- `src/typed_channel.rs:329-368` — substrate's existing libc::poll cascade pattern (mirror its event-mask + tiebreak discipline)
- `src/runtime.rs:201` — `SHUTDOWN_BROADCAST_READ_FD` definition
- `src/comms/process.rs` — Stone A's current state
- `src/comms/thread.rs` — Slice 2 cascade-aware reference (different mechanism, same contract)
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol
- `feedback_never_deadlock` — cascade-aware recv is load-bearing for "deadlocks are illegal"
