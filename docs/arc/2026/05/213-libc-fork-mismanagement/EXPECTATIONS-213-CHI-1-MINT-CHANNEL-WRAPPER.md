# Arc 213 stone χ-1 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 30-45 min Mode A. Additive mint + 3-test smoke probe; pattern mirrors α (Pidfd primitive mint); typed_recv's existing cascade-aware select is the canonical reference shape — sonnet copies the pattern.
- **LOC changed:** ~80-110 (60-80 in src/typed_channel.rs for the wrapper + 25-35 in tests/probe_channel_primitive.rs)
- **New files:** 2 (tests/probe_channel_primitive.rs + SCORE doc)
- **Surprises expected:** LOW. The crossbeam_channel API is stable; the select! macro is in scope already in typed_channel.rs; the SHUTDOWN_RX runtime accessor is the existing chokepoint.

## Honest-delta watch

### Risk 1 — SHUTDOWN_RX accessor signature

The wrapper's `recv()` calls `crate::runtime::SHUTDOWN_RX.get()`. Verify that:
- It exists (typed_recv at src/typed_channel.rs:303 uses it — confirmed exists)
- It returns `Option<&crossbeam_channel::Receiver<()>>` or `Option<crossbeam_channel::Receiver<()>>` (pattern in typed_recv suggests Option<&Receiver>)

If the type returned by `.get()` doesn't compose cleanly with `crossbeam_channel::select!` recv arm, sonnet uses the same dereference dance typed_recv does. Pattern parity with typed_recv's Crossbeam arm (line 304-313).

### Risk 2 — `crossbeam_channel::select!` macro in generic context

`select!` is a procedural macro; generics may or may not compose. typed_recv's usage operates on `&ReceiverInner` (no generic over T at the select! call site — the inner Crossbeam is `crossbeam_channel::Receiver<Value>`). The χ-1 wrapper IS generic over T:

```rust
impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, RecvError> {
        ...
        crossbeam_channel::select! {
            recv(&self.inner) -> msg => msg,    // msg: Result<T, RecvError>
            recv(srx) -> _ => Err(RecvError),
        }
    }
}
```

The select! macro is generic-tolerant — generates code that resolves at each instantiation. Should work; if it doesn't, sonnet documents the failure exactly + STOPs.

### Risk 3 — Send needs a `Sender<T>::clone()`

Many crossbeam Sender callers clone to share across threads. The wrapper's `Sender<T>: Clone` impl handles this. Verify `crossbeam_channel::Sender<T>: Clone` (it is — `Sender<T>` is `Clone` regardless of T). The wrapper's manual `impl<T> Clone for Sender<T>` mirrors this.

Same for Receiver clone — crossbeam_channel::Receiver<T> implements Clone for shared-receiver scenarios; the wrapper's `impl<T> Clone for Receiver<T>` mirrors.

### Risk 4 — Module path in the probe

The probe imports `use wat::typed_channel::{unbounded, RecvError, TryRecvError};`. Verify the crate is published with this path. Check `src/lib.rs` for `pub mod typed_channel;` (it almost certainly is — typed_channel.rs is a substrate-internal module). If not pub, sonnet uses `wat::typed_channel::...` if available OR `wat::typed_channel::...` via the crate's public re-exports.

### Risk 5 — derived traits

Wrapper types may need `Debug` derived for use in panic messages / log output. Add `#[derive(Debug)]` if crossbeam's types implement Debug (they do, when T: Debug). Sonnet may need to add `where T: Debug` bounds OR omit Debug to keep T unconstrained.

Conservative: skip Debug derive on Sender/Receiver wrappers. Existing 35 sites likely don't depend on Debug derive of the channel handle.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `wat::typed_channel::Sender<T>` newtype minted with `send(t) -> Result<(), SendError<T>>` | YES |
| 2 | `wat::typed_channel::Receiver<T>` newtype minted with `recv() -> Result<T, RecvError>` cascade-aware | YES |
| 3 | `Receiver<T>::try_recv() -> Result<T, TryRecvError>` | YES |
| 4 | `unbounded<T>()` + `bounded<T>(n)` factory functions | YES |
| 5 | `Sender<T>` + `Receiver<T>` both implement `Clone` | YES |
| 6 | `RecvError`, `SendError`, `TryRecvError` re-exported from crossbeam_channel | YES |
| 7 | recv() routes through `SHUTDOWN_RX` cascade-aware `select!` (pattern parity with typed_recv) | YES |
| 8 | recv() bootstrap fallback to bare `.inner.recv()` when SHUTDOWN_RX uninitialized | YES |
| 9 | `tests/probe_channel_primitive.rs` minted with 3 tests | YES |
| 10 | Probe test `probe_chi1_unbounded_round_trip` PASS | YES |
| 11 | Probe test `probe_chi1_sender_drop_triggers_recv_err` PASS | YES |
| 12 | Probe test `probe_chi1_try_recv_empty_returns_empty` PASS | YES |
| 13 | cargo build --release clean | YES |
| 14 | No modifications to existing typed_send/typed_recv/SenderInner/ReceiverInner | YES |
| 15 | Zero modifications outside src/typed_channel.rs + tests/probe_channel_primitive.rs + SCORE doc | YES |
| 16 | Dirty tree intact (src/fork.rs + src/spawn_process.rs untouched) | YES |
| 17 | cargo test --release --test wat_arc170_program_contracts result unchanged from pre-χ-1 baseline | YES |

## Mode classification

- **Mode A:** all 17 criteria satisfied; cascade-aware wrapper minted; smoke probe green; dirty tree preserved
- **Mode B (acceptable):**
  - `select!` macro doesn't compose in generic context — sonnet documents + STOPs (orchestrator picks fallback design)
  - `SHUTDOWN_RX.get()` returns a type that doesn't compose with select! recv arm — sonnet documents + STOPs
  - Either Risk 1 or Risk 2 surfaces honestly (Mode B is HONEST design surface, not workaround)
- **Mode C (failure):**
  - Workaround attempt: shipping non-cascade-aware recv "because cascade is hard"
  - Touched files outside the 3-file scope
  - Touched dirty tree (src/fork.rs / src/spawn_process.rs)
  - Made probe tests pass by weakening their assertions
  - Migrated any of the 35 bare-recv/send/try_recv sites (that's χ-2)
  - Committed the work (orchestrator commits)

## Calibration metadata

- **Orchestrator confidence:** HIGH on the design (pattern mirrors typed_recv's existing Crossbeam arm; substrate truth confirmed pre-spawn via grep; no new substrate concepts). MEDIUM-HIGH on first-attempt Mode A (only real risk is select! macro behavior in generic context — should work but unverified empirically).
- **Risk factors:**
  - `crossbeam_channel::select!` generic-context behavior (Risk 2)
  - SHUTDOWN_RX accessor signature subtlety (Risk 1)
- **Why this matters:** χ-1 is the chokepoint primitive. χ-2's substrate-as-teacher cascade depends on the wrapper existing + the type signature being mechanically substitutable for `crossbeam_channel::Sender<T>` / `Receiver<T>` at the call site. The wrapper's API surface is the load-bearing contract; the cascade-awareness inside `recv()` is the structural fix.

## Tractability tiebreaker rationale (per `feedback_tractability_tiebreaker`)

χ-1 minted → χ-2 migration becomes mechanical (rename type imports; cargo build cascade guides sonnet site-by-site). Without χ-1's wrapper existing, χ-2 has nothing to migrate to. χ-1 before χ-2 is forced.

## Cross-references

- INTERSTITIAL § 2026-05-18 (post-δ-1 investigation) "Channel-cascade-completeness wall (arc 213 χ) + 'we are our own users' doctrine" — load-bearing doctrine
- `src/typed_channel.rs:295-340` typed_recv — the canonical cascade-aware select pattern χ-1 mirrors
- `src/typed_channel.rs:203` typed_send — Value-layer sibling
- `src/runtime.rs` SHUTDOWN_RX — the OnceLock the wrapper queries
- Arc 198 `restricted_to` precedent — sets up χ-3's import restriction
- Arc 203 struct-restricted + Arc 212 ζ `WatAST::children()` — same shape at different layers
- `feedback_defect_fix_or_panic_never_revert` — the dirty tree is precious; do not touch
