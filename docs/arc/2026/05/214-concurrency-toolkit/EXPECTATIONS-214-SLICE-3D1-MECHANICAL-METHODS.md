# Arc 214 Slice 3 — Stone D1 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 25-35 min Mode A. Smaller than the original Stone D (split into D1+D2 after four-questions test). All 5 methods + 2 trait impls are MECHANICAL extensions of patterns established in Stones A/B/C — sonnet's main work is mechanical assembly + careful preservation of prior implementations + doc-cascading.
- **LOC changed:** ~350-420 total (~150-180 added to `src/comms/process.rs`; ~200-240 added to `tests/probe_comms_process.rs`).
- **New files:** 1 (SCORE doc only).
- **Surprises expected:** LOW-MEDIUM. The BRIEF skeleton is exhaustive; risks are micro-API drifts on libc::poll, OwnedFd::try_clone return type, and the Clone-for-Receiver-fresh-accumulator semantic.

## Honest-delta watch

### Risk 1 — libc::poll(timeout=0) signature drift

**What:** `libc::poll(fds: *mut pollfd, nfds: nfds_t, timeout: c_int)`. The BRIEF uses `fds.as_mut_ptr()`, `nfds as libc::nfds_t`, `0`. If sonnet drops the `libc::nfds_t` cast on `nfds`, it'll be a type mismatch. If sonnet writes `0i32` instead of bare `0`, Rust coerces — fine. If sonnet uses libc::poll without unsafe, compile error.

**Mitigation:** BRIEF skeleton is explicit on cast + unsafe.

### Risk 2 — try_recv nfds=1 (broadcast uninitialized)

**What:** When `broadcast_fd < 0`, only the first pollfd should be polled. The BRIEF sets the second pollfd's `fd: -1` (ignored by kernel) and passes `nfds=1`. If sonnet always passes `2`, kernel may error on the -1 fd or behave unexpectedly.

**Mitigation:** BRIEF code skeleton has the `if broadcast_fd >= 0 { 2 } else { 1 }` calculation.

### Risk 3 — try_recv EOF mapping to Disconnected

**What:** When poll fires data arm AND the subsequent io_uring Read returns 0 (EOF), try_recv MUST return `Err(TryRecvError::Disconnected)`. Returning Empty would cause caller retry-loops to spin forever.

**Mitigation:** BRIEF skeleton: `if result == 0 { return Err(TryRecvError::Disconnected); }` explicit.

### Risk 4 — Clone for Receiver fresh accumulator vs cloned accumulator

**What:** The BRIEF specifies `accumulator: RefCell::new(Vec::new())` for cloned receivers — FRESH empty buffer, NOT a clone of the original. If sonnet writes `self.accumulator.clone()`, the clone gets the original's bytes-so-far → phantom-frame behavior.

**Mitigation:** BRIEF code + doc-comment both explicit. Probe test `probe_slice3d1_receiver_clone_competes_for_frames` catches if wrong.

### Risk 5 — OwnedFd::try_clone vs std::clone::Clone trait return type

**What:** `OwnedFd::try_clone()` returns `io::Result<OwnedFd>`. `Clone::clone()` returns `Self` (no Result). The BRIEF uses `.expect()` with diagnostic — fail-stop on fd table exhaustion. If sonnet provides `try_clone() -> Result<Self, _>` instead of `Clone` impl, generic code expecting `Sender<T>: Clone` won't compile.

**Mitigation:** BRIEF explicitly says "impl Clone for ..."; uses `.expect()`.

### Risk 6 — Module + struct doc cascading updates

**What:** Three doc updates needed: module-level "Current scope (through Stone C)" → "(through Stone D1)"; Sender doc retires "NOT Clone / NOT close-able"; Receiver doc retires "NOT Clone". Same Stone B gaze L1 lesson. Stone D1 still names "NO Select (Stone D2)" honestly as deferred.

**Mitigation:** BRIEF spells out exact replacement text for all three.

### Risk 7 — Preserving Stones A-C unchanged

**What:** Stone D1 ADDS around Stone C's send/recv/take_frame/decode_frame/wait_for_data_or_cascade/PollOutcome/pair — all of which should be UNCHANGED. If sonnet refactors any "while in the area", we break the per-stone-trust-gate invariant.

**Mitigation:** BRIEF's STOP triggers list every helper as UNCHANGED. Sonnet's job is additive only.

### Risk 8 — Sender::send / Receiver::recv body preservation

**What:** Stone C's `Sender::send` has a no-clone-on-error pattern (returns `SendError(value)` not `SendError(value.clone())`). Stone B's `Receiver::recv` has cascade-aware multi-arm POLL_ADD via `wait_for_data_or_cascade`. If sonnet "while in the file" cleans up either body, they regress.

**Mitigation:** BRIEF explicitly names both as UNCHANGED + lists them in STOP triggers.

### Risk 9 — Test name prefix

**What:** New tests use `probe_slice3d1_*` prefix; existing 6 tests keep `probe_slice3c_*`. If sonnet renames the existing tests or uses `probe_slice3d_*` (the deleted-stone prefix), gaze flags inconsistency.

**Mitigation:** BRIEF explicit on prefix + naming preservation.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | Module-level doc: "(through Stone C)" → "(through Stone D1)" with Stone D1 capability description | YES |
| 2 | Sender struct doc retires "NOT Clone (Stone D adds Clone). NOT close-able (Stone D adds close(self))" stale text | YES |
| 3 | Receiver struct doc retires "NOT Clone (Stone D adds)" stale text; declares Clone + close + try_recv + len availability | YES |
| 4 | New imports: `CloseError, CommReceiver, CommSender, TryRecvError` added to `use crate::comms::{...}` | YES |
| 5 | Imports do NOT include `ReceiverIndex` or `SelectOutcome` (Stone D2 will add) | YES |
| 6 | `Sender::close(self) -> Result<(), CloseError>` returns Ok(()); doc names OwnedFd Drop semantics | YES |
| 7 | `impl Clone for Sender<T>` uses `OwnedFd::try_clone` with `.expect()` on fd table exhaustion | YES |
| 8 | Sender Clone preserves `_phantom: PhantomData` initializer | YES |
| 9 | `impl CommSender<T> for Sender<T>` delegates `send` + `close` to inherent methods | YES |
| 10 | `Receiver::try_recv() -> Result<T, TryRecvError>` uses `libc::poll(timeout=0)` | YES |
| 11 | try_recv handles `nfds=1` (broadcast uninitialized) vs `nfds=2` correctly | YES |
| 12 | try_recv broadcast-wins-ties: if broadcast fires, returns `Err(TryRecvError::Disconnected)` | YES |
| 13 | try_recv on EOF (Read result == 0) returns `Err(TryRecvError::Disconnected)` | YES |
| 14 | try_recv on partial bytes (no complete frame after Read) returns `Err(TryRecvError::Empty)` | YES |
| 15 | try_recv fast-path (accumulator complete frame) returns Ok(T) without io_uring | YES |
| 16 | `Receiver::len() -> usize` returns count of '\n' bytes in accumulator | YES |
| 17 | `Receiver::close(self) -> Result<(), CloseError>` returns Ok(()) | YES |
| 18 | `impl Clone for Receiver<T>` uses `OwnedFd::try_clone` + FRESH `RefCell::new(Vec::new())` | YES |
| 19 | Receiver Clone doc-comment explains MPMC-competing-clones + per-endpoint accumulator | YES |
| 20 | `impl CommReceiver<T> for Receiver<T>` delegates `recv` + `try_recv` + `len` + `close` to inherent methods | YES |
| 21 | `tests/probe_comms_process.rs` preserves the 6 existing `probe_slice3c_*` tests unchanged | YES |
| 22 | Adds 10 new `probe_slice3d1_*` tests covering: try_recv (3) + len (1) + close (2) + Clone (2) + trait dispatch (2) | YES |
| 23 | All 16 probe tests PASS (6 + 10) | YES |
| 24 | `cargo build --release` clean | YES |
| 25 | Prior 4 probe suites unchanged | YES |
| 26 | Zero modifications outside `src/comms/process.rs` + `tests/probe_comms_process.rs` + SCORE doc | YES |
| 27 | Dirty tree intact | YES |
| 28 | `src/typed_channel.rs`, `src/edn_shim.rs`, `src/comms/mod.rs`, `Cargo.toml` untouched | YES |
| 29 | NO `wat_arc170_program_contracts` re-run | YES |
| 30 | NO `Select<'a, T>` implementation (Stone D2 territory) | YES |
| 31 | NO `ReceiverIndex` / `SelectOutcome` references in Stone D1 code | YES |
| 32 | Stone A's `take_frame` UNCHANGED | YES |
| 33 | Stone B's `wait_for_data_or_cascade` + `PollOutcome` UNCHANGED | YES |
| 34 | Stone C's `decode_frame::<T>` + `Sender::send` + `Receiver::recv` + `pair<T>()` UNCHANGED | YES |
| 35 | Every new public item has a doc comment (gaze L2 pre-emption) | YES |
| 36 | Every new `unsafe` block has a SAFETY comment (forge pre-emption) | YES |
| 37 | NO commit (orchestrator owns the commit after ward pass) | YES |

## Mode classification

- **Mode A:** all 37 criteria satisfied
- **Mode B (acceptable):**
  - Risk 1/2 fires (libc::poll micro-drift): sonnet adjusts on cargo error
  - Risk 3 fires (try_recv EOF → Empty): probe test 2 catches
  - Risk 4 fires (Receiver Clone cloned accumulator): probe test 8 catches
  - One probe test fails: sonnet investigates per failure mode
- **Mode C (failure):**
  - Touched any file outside 2-file scope + SCORE doc
  - Implemented `Select<'a, T>` (Stone D2 territory)
  - Modified Stones A-C's preserved helpers
  - Cloned accumulator in Receiver::clone

## Calibration metadata

- **Orchestrator confidence:** HIGH on first-attempt Mode A. D1 is mechanical extensions of established patterns; the genuine novelty (Select<'a, T>) is deferred to D2.
- **Risk factors:** libc::poll micro-API (mitigated by BRIEF + cargo errors); Receiver Clone semantics (mitigated by BRIEF + probe test 8); module/struct doc cascading (mitigated by explicit BRIEF text).

## Ward pass prediction

- gaze: 0-1 (smaller doc-cascade footprint than original Stone D; risk is residual stale-claim)
- forge: 0-1 (possible candidate-rune on Clone::clone panic-via-expect vs Result return shape)
- reap: 0
- sever: 0
- temper: 0-1 (per-call IoUring in try_recv inherits Stone C deferral)

Round-1-clean is possible (precedent: Stone C). Otherwise minor findings.

## Cross-references

- BRIEF-214-SLICE-3D1-MECHANICAL-METHODS.md — this stone's work order
- Stones A-C BRIEFs + WARD-PASS docs
- `src/comms/thread.rs` — Slice 2 thread tier mirror
- `src/typed_channel.rs:407-470` — existing PipeFd try_recv pattern
- `feedback_iterative_complexity` — Stone D split into D1 + D2 per four-questions
