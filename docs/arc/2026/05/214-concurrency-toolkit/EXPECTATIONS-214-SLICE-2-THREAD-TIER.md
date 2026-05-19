# Arc 214 Slice 2 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 45-60 min Mode A. Bigger than Slice 1 (more code; non-trivial Select<'a, T> implementation with internal index tracking). Heavy reference to χ-1's existing wrapper at typed_channel.rs:541+ makes the Sender/Receiver implementations near-mechanical; Select<'a, T> is the new structural work.
- **LOC changed:** ~280-350 (~180-220 in src/comms/thread.rs; ~100-130 in tests/probe_comms_thread.rs; +1 line in src/comms/mod.rs)
- **New files:** 3 (`src/comms/thread.rs`, `tests/probe_comms_thread.rs`, SCORE doc)
- **Surprises expected:** LOW-MEDIUM. The Sender/Receiver shape is established (χ-1 reference). The Select<'a, T> internal index tracking is the only judgment call — sonnet must correctly map crossbeam-internal indices to user-facing ReceiverIndex.

## Honest-delta watch

### Risk 1 — Select<'a, T> index mapping

`crossbeam_channel::Select` returns the crossbeam-arm index when `select()` fires. The user-facing API exposes a separate `ReceiverIndex` that reflects registration order (independent of internal arm indices). Sonnet must track the mapping in `user_arms: Vec<(crossbeam_idx, &Receiver<T>)>` and `find()` the user position by crossbeam index when select fires.

If sonnet conflates the two (returns crossbeam-internal index as ReceiverIndex), tests will FAIL on `probe_slice2_select_indices_match_registration_order`. The BRIEF spells out the mapping explicitly; sonnet's job is to implement it faithfully.

### Risk 2 — SendError variant mapping

`crossbeam_channel::SendError<T>` and `crate::comms::SendError<T>` have the same shape but distinct types. Sonnet maps via:

```rust
self.inner.send(value).map_err(|crossbeam_channel::SendError(v)| SendError(v))
```

If sonnet forgets the destructure and writes `.map_err(|e| SendError(e))`, the compile will fail (or worse, type-erase). The pattern is spelled out in the BRIEF; sonnet copies.

### Risk 3 — TryRecvError variant mapping

`crossbeam_channel::TryRecvError` has variants `Empty` / `Disconnected`. `crate::comms::TryRecvError` has the same variant names. Match-arm mapping is mechanical; spelled out in the BRIEF.

### Risk 4 — Cascade-aware recv lifetime

`crossbeam_channel::select! { recv(&self.inner) -> msg => ... }` needs `&self.inner` to outlive the select! invocation. In a `&self`-method context this is fine. If sonnet accidentally `let inner_ref = &self.inner;` outside the macro and then references it inside, no lifetime issue but unnecessary indirection. The BRIEF spells out the inline form.

### Risk 5 — Bootstrap fallback path

When `SHUTDOWN_RX.get()` returns `None` (pre-init), recv falls back to bare `self.inner.recv()`. This is the χ-1 pattern and the typed_recv pattern. If sonnet OMITS the fallback (no `match shutdown_rx { Some(srx) => ..., None => ... }`), recv will panic in bootstrap. The BRIEF spells out both arms.

### Risk 6 — Default impl for Select<'a, T>

Rust convention: types constructible via `new()` with no args should impl `Default`. The BRIEF includes the `impl Default for Select<'a, T>` block. If sonnet omits it, clippy may warn (`new_without_default`). Minor; mechanically resolved.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `src/comms/thread.rs` minted with module-level cascade-contract doc | YES |
| 2 | `Sender<T>` newtype with private inner `crossbeam_channel::Sender<T>` field | YES |
| 3 | `Sender<T>::send` maps `crossbeam::SendError<T>` → `comms::SendError<T>` correctly | YES |
| 4 | `Sender<T>::close(self)` returns `Ok(())` (thread-tier close infallible) | YES |
| 5 | `Sender<T>: Clone` | YES |
| 6 | `impl<T: Send + 'static> CommSender<T> for Sender<T>` | YES |
| 7 | `Receiver<T>` newtype with private inner `crossbeam_channel::Receiver<T>` field | YES |
| 8 | `Receiver<T>::recv` cascade-aware via `select! { recv(data), recv(SHUTDOWN_RX) }` with bootstrap fallback | YES |
| 9 | `Receiver<T>::try_recv` maps `crossbeam::TryRecvError` → `comms::TryRecvError` correctly | YES |
| 10 | `Receiver<T>::len` trivial passthrough to crossbeam | YES |
| 11 | `Receiver<T>::close(self)` returns `Ok(())` (infallible) | YES |
| 12 | `Receiver<T>: Clone` | YES |
| 13 | `impl<T: Send + 'static> CommReceiver<T> for Receiver<T>` | YES |
| 14 | `Select<'a, T: Send + 'static>` with internal `crossbeam::Select` + `shutdown_arm` + `user_arms` | YES |
| 15 | `Select::new()` auto-registers SHUTDOWN_RX (when initialized) | YES |
| 16 | `Select::recv(rx)` returns `ReceiverIndex` matching registration order (0, 1, 2, ...) | YES |
| 17 | `Select::select()` returns `SelectOutcome::Shutdown` when shutdown arm fires | YES |
| 18 | `Select::select()` returns `SelectOutcome::Recv { index, result }` for user-arm fires; `index` correctly maps crossbeam-arm-idx → registration order | YES |
| 19 | `impl Default for Select<'a, T>` | YES |
| 20 | `pair<T: Send + 'static>()` factory | YES |
| 21 | `bounded<T: Send + 'static>(capacity)` factory | YES |
| 22 | `pub mod thread;` added to `src/comms/mod.rs` | YES |
| 23 | `tests/probe_comms_thread.rs` minted with 10 smoke tests | YES |
| 24 | All 10 probe tests PASS | YES |
| 25 | `cargo build --release` clean (no new warnings) | YES |
| 26 | `cargo test --release --test probe_comms_foundation` 3/3 PASS unchanged | YES |
| 27 | `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged (χ-1 untouched) | YES |
| 28 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged | YES |
| 29 | Zero modifications outside the 3-file scope (`src/comms/thread.rs` new, `src/comms/mod.rs` +1 line, `tests/probe_comms_thread.rs` new, SCORE doc new) | YES |
| 30 | Dirty tree intact (`src/fork.rs` + `src/spawn_process.rs` untouched) | YES |
| 31 | NO `wat_arc170_program_contracts` re-run (per `feedback_no_hang_vector_in_additive_scorecard`) | YES |
| 32 | NO touches to `src/typed_channel.rs` (χ-1 wrapper untouched; Slice 5 migrates later) | YES |
| 33 | Every public item has a doc comment (gaze L2 pre-emption) | YES |
| 34 | All tests have real assertions (no bare `_`-bindings without follow-up assertion) (gaze L1 pre-emption) | YES |

## Mode classification

- **Mode A:** all 34 criteria satisfied; thread tier shipped clean; wards-pre-emption discipline encoded
- **Mode B (acceptable; honest surface):**
  - Risk 1 fires (Select index-mapping misimplemented): tests will fail; sonnet STOPs + reports; orchestrator-direct fix
  - Risk 4/5 fires (lifetime or bootstrap issue): cargo build fails; sonnet documents + reports
  - One of the 10 probe tests fails: sonnet investigates per the failure mode; reports honestly
- **Mode C (failure):**
  - Touched any file outside the 3-file scope
  - Touched src/typed_channel.rs (χ-1 wrapper)
  - Touched the dirty tree
  - Ran wat_arc170_program_contracts
  - Committed the work
  - Implemented process tier or kernel layer (Slice 3/4 territory)

## Calibration metadata

- **Orchestrator confidence:** HIGH on first-attempt Mode A. The Sender/Receiver shape is established (χ-1 reference at typed_channel.rs); the cascade-aware pattern is established (typed_recv reference). Select<'a, T> is the only structurally new work, and the BRIEF spells out the index-mapping pattern explicitly.
- **Risk factors:**
  - Select index mapping (Risk 1) — mitigated by explicit BRIEF + dedicated probe test
  - Bootstrap fallback omission (Risk 5) — mitigated by explicit BRIEF + the χ-1 reference pattern
- **Why this matters:** Slice 2 is the FIRST concrete tier implementation. Slice 3 (process tier, io_uring) mirrors Slice 2's structure with different underlying mechanism. If Slice 2 lands clean + impeccable (ward pass green), Slice 3 has a proven template to follow. The kernel-impeccability protocol gets its second cycle here; we walk it with the confidence of the first cycle's success.

## Ward pass prediction

Per the new kernel-impeccability protocol: after SCORE verification, 4 wards spawn in parallel (gaze + forge + reap + sever). Slice 2 pre-empts Slice 1's findings via the BRIEF's explicit ward-discipline section, so the round 1 ward pass should be CLEAN or near-clean. **NEW for Slice 2 (runtime logic lands here): temper ward also applies** — check for redundant computation (e.g., recomputing SHUTDOWN_RX.get() per-call when it could cache; allocation in hot paths; redundant traversals in Select::select).

Predicted ward findings: 0-2 (mostly clean; possible temper findings on the `user_arms.iter().find()` traversal that runs per-select).

## Tractability tiebreaker rationale

Slice 2 follows Slice 1 in the dependency graph (uses Slice 1's traits + types). No alternative ordering. Within Slice 2: single coherent concern (thread tier); decomposed into sender/receiver/select/factories/probe at the IMPLEMENTATION level but logically ONE concern at the slice level. No further splitting needed at slice-decomposition layer.

## Cross-references

- BRIEF-214-SLICE-2-THREAD-TIER.md — this stone's work order
- `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` — full arc 214 design
- `docs/arc/2026/05/214-concurrency-toolkit/WARD-PASS-1-FOUNDATION-PRIMITIVES.md` — Slice 1 ward round-trip; lessons pre-empted
- `src/typed_channel.rs:541+` — χ-1 wrapper PATTERN REFERENCE
- `src/typed_channel.rs:295-345` — typed_recv cascade-aware select pattern
- `src/runtime.rs:179` — SHUTDOWN_RX
- `src/comms/mod.rs` — Slice 1 traits + types
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol
- `feedback_no_hang_vector_in_additive_scorecard` — verification discipline
- `feedback_defect_fix_or_panic_never_revert` — dirty tree preservation
- `feedback_iterative_complexity` — stepping stone discipline
