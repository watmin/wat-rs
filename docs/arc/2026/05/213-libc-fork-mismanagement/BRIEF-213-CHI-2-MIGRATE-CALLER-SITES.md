# Arc 213 stone χ-2 — Migrate caller sites to `wat::typed_channel` wrapper

## Substrate gap (recap)

χ-1 shipped (`0097ee3`) the substrate-owned chokepoint primitive:
- `wat::typed_channel::{Sender<T>, Receiver<T>}` newtypes
- `Receiver<T>::recv()` routes through `SHUTDOWN_RX` cascade-aware `select!`
- `unbounded<T>()` / `bounded<T>(n)` factories
- 3-test smoke probe (`tests/probe_channel_primitive.rs`) verified

Now the wrapper exists, but **35 caller sites still use bare `crossbeam_channel::{Sender, Receiver}` directly** — bypassing the cascade. They live in 4 files:

| File | `.recv()` | `.send()` | `.try_recv()` | Total |
|---|---|---|---|---|
| `src/thread_io.rs` | 9 | 15 | 0 | 24 |
| `src/runtime.rs` | 5 | 3 | 1 | 9 |
| `src/freeze.rs` | 1 | 0 | 0 | 1 |
| `src/spawn.rs` | 0 | 1 | 0 | 1 |
| **Total** | **15** | **19** | **1** | **35** |

These bypass `typed_recv`'s cascade-aware `select!`; when the substrate's shutdown cascade fires, the parked recvs don't wake → parent hangs → orphan accumulates. THIS is the cascade-completeness gap the χ doctrine exists to close (INTERSTITIAL § 2026-05-18 "Channel-cascade-completeness wall").

## Mission

**Migrate the 4 caller files to `wat::typed_channel::*`.** The wrapper's methods have identical names to crossbeam's, so `.recv()` / `.send()` / `.try_recv()` call sites need no edits — only the **field types**, **factory calls**, and **imports** change. Once those rename, the call sites automatically inherit cascade-aware semantics.

## Mechanical edits per file

In each of `src/thread_io.rs`, `src/runtime.rs`, `src/freeze.rs`, `src/spawn.rs`:

1. **Imports** — replace
   ```rust
   use crossbeam_channel::{Receiver, Sender};
   ```
   with
   ```rust
   use crate::typed_channel::{Receiver, Sender};
   ```
   (only `src/thread_io.rs:28` currently has this exact form; other files reference types via `crossbeam_channel::Sender` / `crossbeam_channel::Receiver` directly without an import — adjust per local style)

2. **Field types** — in struct/enum definitions:
   ```rust
   crossbeam_channel::Sender<T>  →  crate::typed_channel::Sender<T>
   crossbeam_channel::Receiver<T>  →  crate::typed_channel::Receiver<T>
   ```

3. **Factory calls** — in function bodies:
   ```rust
   crossbeam_channel::unbounded::<T>()  →  crate::typed_channel::unbounded::<T>()
   crossbeam_channel::bounded::<T>(n)  →  crate::typed_channel::bounded::<T>(n)
   ```

4. **Method calls** — `.recv()` / `.send(...)` / `.try_recv()` need NO edits (wrapper has identical method names; field type change does the work).

## CRITICAL exclusions — DO NOT touch the cascade primitives in `src/runtime.rs`

The substrate's shutdown cascade infrastructure ITSELF uses bare `crossbeam_channel`. These lines MUST stay bare — they're the foundation the wrapper builds on:

- **`src/runtime.rs:179`** — `pub static SHUTDOWN_RX: OnceLock<crossbeam_channel::Receiver<()>>` (the broadcast cascade receiver every wrapper's `recv()` queries via `SHUTDOWN_RX.get()`)
- **`src/runtime.rs:185`** — `static SHUTDOWN_TX_PTR: AtomicPtr<crossbeam_channel::Sender<()>>` (the cascade trigger)
- **`src/runtime.rs:233`** — `let (tx, rx) = crossbeam_channel::unbounded::<()>();` (the factory that creates the cascade pair inside `init_shutdown_signal`)

These three lines are LOAD-BEARING and must not migrate. They define the cascade itself; migrating them would create a circular dependency (the wrapper queries SHUTDOWN_RX; SHUTDOWN_RX would query SHUTDOWN_RX).

If you find any other `crossbeam_channel::` references in `runtime.rs` that involve `SHUTDOWN_RX`, `SHUTDOWN_TX_PTR`, `SHUTDOWN_TX`, `SHUTDOWN_BROADCAST_READ_FD`, or related shutdown infrastructure — those also stay bare. When in doubt: if the reference is in code that DEFINES the cascade primitive, leave it; if it's in code that USES some other channel (e.g., a Process<I,O> mailbox, a thread-handle pair, etc.), migrate.

Doc-comments referencing `:rust::crossbeam_channel::Sender<T>` (the wat-visible type path, e.g., line 359, 417) — leave unchanged. Those are user-facing naming; not Rust callers.

## DO NOT touch these files (out of scope)

- `src/typed_channel.rs` — wrapper home; already has the chokepoint
- `src/check.rs` — type-checker references to `:rust::crossbeam_channel::*` are wat-visible TYPE NAMES in error messages + doc comments; not Rust callers
- `src/lexer.rs` / `src/parser.rs` — test fixtures tokenizing the wat-visible keyword form; not Rust callers
- `src/types.rs` — type registry entries (lines 922, 929) reference wat-visible type names; not Rust callers
- `src/fork.rs` / `src/spawn_process.rs` — DIRTY TREE δ-1 replication artifacts; per `feedback_defect_fix_or_panic_never_revert` they are precious; STOP if you find yourself about to edit them

## Verification

```
cargo build --release                                  # must be clean
cargo test --release --test probe_channel_primitive    # 3/3 PASS (unchanged)
cargo test --release --test probe_pidfd_primitive      # 2/2 PASS (unchanged)
```

**That is the FULL verification. STOP here.** Per `feedback_no_hang_vector_in_additive_scorecard` — do NOT run `wat_arc170_program_contracts` or any other workspace test. The actual hang-elimination proof belongs to χ-4 (50-trial gate); χ-2 is the migration that ENABLES that proof. Running workspace tests here produces orphan-accumulation noise + risks misdiagnosis.

cargo build clean + both probes still passing is sufficient evidence that:
- The wrapper's signature is signature-identical for caller-site method calls (else build fails)
- The cascade primitives are still wired correctly (else probe_channel_primitive sender-drop-Err would fail)
- The pidfd primitive's existing infrastructure still works (else probe_pidfd_primitive would fail)

## Out of scope (STOP triggers)

- DO NOT add `#[restricted_to(...)]` on `crossbeam_channel` imports. That's χ-3 — the wall enforcement comes after the migration.
- DO NOT run wat_arc170_program_contracts, wat_arc170_*, or full workspace tests. The hang-elimination proof belongs in χ-4.
- DO NOT touch the dirty tree (src/fork.rs / src/spawn_process.rs).
- DO NOT touch the cascade primitives (SHUTDOWN_RX / SHUTDOWN_TX_PTR / init_shutdown_signal in runtime.rs).
- DO NOT migrate type-name strings in check.rs / lexer.rs / parser.rs / types.rs (those are wat-visible type paths, not Rust callers).
- If cargo build fails after your edits and you can't resolve via mechanical substrate-as-teacher cascade (e.g., you hit a method on crossbeam that the wrapper doesn't expose): STOP, write the SCORE with the honest report (what method, which site, what cargo says), do NOT add the method to the wrapper (that's a separate stone if needed).
- If the wrapper turns out to need additional methods (`len`, `is_empty`, `iter`, etc.) to support the existing call sites: STOP and report. Do NOT extend the wrapper yourself; that's a follow-up decision.

## Concrete deliverables

1. Edits to `src/thread_io.rs`, `src/runtime.rs`, `src/freeze.rs`, `src/spawn.rs` per the per-file migration rules above
2. SCORE doc `docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-CHI-2-MIGRATE-CALLER-SITES.md` per EXPECTATIONS scorecard

## Critical constraints

- DO NOT commit. Orchestrator commits after independent SCORE verification.
- DO NOT touch any file outside the 4 caller files + the SCORE doc.
- DO NOT touch the dirty tree files (src/fork.rs / src/spawn_process.rs).
- DO NOT touch the cascade primitives in runtime.rs (named lines above).
- Use the existing CWD (`/home/watmin/work/holon/wat-rs/`); do not cd elsewhere.

## Cross-references

- `src/typed_channel.rs` — wrapper home; χ-1's mint
- `tests/probe_channel_primitive.rs` — χ-1 smoke probe (must still pass)
- INTERSTITIAL § 2026-05-18 (post-δ-1 investigation) "Channel-cascade-completeness wall" — load-bearing doctrine
- SCORE-213-CHI-1-MINT-CHANNEL-WRAPPER.md — χ-1's verification record
- `feedback_no_hang_vector_in_additive_scorecard` — why χ-2 does NOT verify via wat_arc170
- Arc 198 `restricted_to` pattern — sets up χ-3's enforcement
