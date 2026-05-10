# Arc 170 slice 1f-0b — SCORE

**Result:** Mode A clean.
**Runtime:** ~8 min sonnet (way under predicted 60-90 band; well under 180 hard cap).
**Files:** 3 modified (`src/thread_io.rs`, `src/lib.rs`, `tests/wat_arc170_slice_1f_alpha_helpers.rs`).

## Calibration

- **Predicted runtime band:** 60-90 min (sonnet runs faster than the opus-tier prediction)
- **Actual:** ~8 min — far under band
- **Why faster than predicted:** the BRIEF gave verbatim before/after Rust for every edit; pattern was fully specified; no judgment required beyond the bounded honest-delta categories (which all resolved to BRIEF defaults without friction)
- **Model decision validated:** Sonnet was the right call for this slice. The work was mechanical pattern-application — sonnet's wheelhouse. Opus would have spent equivalent time on the same edits; the design-call surface was bounded by explicit BRIEF prescriptions.

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A — Three Event enums minted | ✓ `pub enum StdInServiceEvent`, `StdOutServiceEvent`, `StdErrServiceEvent` at `src/thread_io.rs:43`, `:60`, `:73` |
| B — ThreadId typealias | ✓ `pub type ThreadId = i64;` at `src/thread_io.rs:38` |
| C — ThreadIO fields use Event-typed senders | ✓ `stdout_tx: Sender<StdOutServiceEvent>`, `stderr_tx: Sender<StdErrServiceEvent>`, `stdin_tx: Sender<StdInServiceEvent>` at lines 98/102/106 |
| D — `eval_kernel_println` constructs Write variant | ✓ `send(StdOutServiceEvent::Write { line })` at line 193 |
| E — `eval_kernel_eprintln` constructs Write variant | ✓ `send(StdErrServiceEvent::Write { line })` at line 222 |
| F — `eval_kernel_readln` constructs Read variant | ✓ `send(StdInServiceEvent::Read)` at line 256 |
| G — `src/lib.rs` re-exports Event enums | ✓ `pub use thread_io::{install_thread_io, uninstall_thread_io, StdInServiceEvent, StdOutServiceEvent, StdErrServiceEvent, ThreadId, ThreadIO}` at `src/lib.rs:95-100` |
| H — All 10 test rows pass | ✓ 10/10 in `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` |
| I — `cargo check --release` green | ✓ clean compile (1 pre-existing warning unrelated) |
| J — Workspace within ±5 band | ✓ exactly 1328/854 (no delta from post-1f-0a baseline; perfect) |
| K — Zero new dependencies | ✓ Cargo.toml unchanged |
| L — Zero new Mutex / RwLock / CondVar | ✓ grep returns 0 hits in modified files |
| M — Type-check arms unchanged | ✓ TypeScheme for println/eprintln (`∀T. T → nil`) + readln (`() → HolonAST`) at `src/check.rs:12819/12831` untouched |
| N — Honest deltas surfaced | ✓ 7 categories surfaced (all bounded; all resolved to BRIEF defaults without friction) |

**14/14 rows pass.** Mode A clean.

## Honest deltas surfaced

1. **Field name `stdout_tx` vs `stdout_req_tx`** — BRIEF preference adopted without friction. Elected: `stdout_tx` / `stderr_tx` / `stdin_tx`.

2. **ThreadId representation** — typealias `pub type ThreadId = i64` per BRIEF. No consumer requires newtype guarantees; slice 1f-γ will populate from a monotonic counter. Elected: typealias.

3. **`#[derive(Clone)]` on Event enums** — all three carry `#[derive(Debug, Clone)]` per BRIEF. Clone on Add variant requires `Receiver<T>` and `Sender<T>` to be Clone — crossbeam's channel ends ARE Clone; compiles without issue. Needed for slice 1f-β-i's service implementations that pattern-match on Add/Remove.

4. **`#[derive(Debug)]`** — present on all three; required for clean test-assertion diagnostics. No friction.

5. **Module location** — kept in `src/thread_io.rs` alongside `ThreadIO`. Natural home; no friction requiring extraction.

6. **`Arc<HolonAST>` ownership on stdin Add variant** — `reply_tx: Sender<Arc<HolonAST>>` in `StdInServiceEvent::Add` composes correctly with crossbeam + Arc. Verified clean.

7. **No other consumers of old field names** — grep across all of `src/` and `tests/` (excluding the two slice files) returned zero hits. No unilateral migration needed beyond stated scope.

## Calibration row

- **Actual runtime:** ~8 min (Mode A clean — well under predicted band)
- **Workspace post-1f-0b:** 1328 passed / 854 failed
- **Fail-count delta from post-1f-0a baseline:** 0 (855 baseline rot stays put; slice 1f-0b is parallel infrastructure)
- **Pass-count delta:** 0 (the 10 1f-α tests migrated but stayed green)
- **Honest deltas surfaced:** 7 (all resolved cleanly to BRIEF defaults)
- **Model decision:** Sonnet validated — work was mechanical; design surface bounded by BRIEF prescriptions

## Implementation choices (locked)

- **ThreadId rep:** typealias (`pub type ThreadId = i64`)
- **Field-name choice:** `stdout_tx` / `stderr_tx` / `stdin_tx`
- **Derives on Event enums:** `#[derive(Debug, Clone)]` on all three (crossbeam channel ends are Clone; compiles clean)
- **Module location:** `src/thread_io.rs` (no extraction)
- **Event enum shapes:** verbatim per BRIEF + pass 18

## Files modified

- `src/thread_io.rs` — 3 new pub enums + ThreadId typealias + ThreadIO struct field-type changes + 3 eval-arm body edits
- `src/lib.rs` — re-exports for Event enums + ThreadId
- `tests/wat_arc170_slice_1f_alpha_helpers.rs` — 10 test rows migrated to construct/match Event variants

## Lessons captured

1. **BRIEF with verbatim before/after = sonnet at 10x speed.** The 60-90 min opus prediction was over-budgeted because the BRIEF supplied the exact Rust code. Sonnet executed in ~8 min. Future mechanical-pattern slices should be sonnet-default when the BRIEF provides verbatim code.

2. **Honest-delta categories resolved to BRIEF defaults across the board.** No friction; the bounded design-surface in the BRIEF was tight enough that sonnet didn't have to reach. Future BRIEFs of this shape are well-calibrated.

3. **Workspace baseline unchanged** — the slice is parallel infrastructure; the 854-failure rot stays put. Slice 1f-0a-ii / iii / iv (or whatever rot-fix slices land) reduce that baseline.

4. **Pass 18's Event-protocol concretization is complete on the Rust side.** Slice 1f-β-i-redux now has concrete Rust Event types to mirror in the wat-side StdInService implementation.

## What's next

1. **Commit slice 1f-0b atomically** (this turn) — bundle the 3 modified files + this SCORE doc
2. **Author slice 1f-β-i-redux BRIEF + EXPECTATIONS** — `wat/kernel/services/stdin.wat` wat-side StdInService with unified Event protocol; mirrors the Rust Event types shipped here
3. **Spawn slice 1f-β-i-redux** (opus + wat-author for pattern-minting; predicted 60-90 min)

## Cross-references

- BRIEF: [`BRIEF-SLICE-1F-0B.md`](./BRIEF-SLICE-1F-0B.md)
- EXPECTATIONS: [`EXPECTATIONS-SLICE-1F-0B.md`](./EXPECTATIONS-SLICE-1F-0B.md)
- BUILD-PLAN ref: §3 slice 1f-0b
- REALIZATIONS pass 18 (the locked Event protocol this slice instantiates on Rust side)
- Predecessor: slice 1f-α (`fcaf600`) — this slice MODIFIES what 1f-α shipped per pass 18
- Successor: slice 1f-β-i-redux (wat-side StdInService) + slices 1f-β-ii / iii / γ / δ / ε
