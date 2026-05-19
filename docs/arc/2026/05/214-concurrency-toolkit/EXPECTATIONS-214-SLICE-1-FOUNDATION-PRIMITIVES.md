# Arc 214 Slice 1 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 20-30 min Mode A. Atomic stepping stone; pure type/trait/error definitions; no runtime behavior; one example HolonRepresentable impl in the smoke probe. Smallest slice in arc 214.
- **LOC changed:** ~120-180 (80-120 in `src/comms/mod.rs` for traits + errors + docs; 40-60 in `tests/probe_comms_foundation.rs` for 3 smoke tests; 1 line in `src/lib.rs`)
- **New files:** 3 (`src/comms/mod.rs`, `tests/probe_comms_foundation.rs`, SCORE doc)
- **Surprises expected:** LOW. Type definitions + trait definitions are mechanical Rust; the only judgment call is the blanket impl decision for HolonRepresentable.

## Honest-delta watch

### Risk 1 — HolonRepresentable blanket impl complexity

The blanket impl `impl<T> HolonRepresentable for T where T: Into<HolonAST> + ...` has subtle trait-bound issues:
- `Into<HolonAST>` consumes self; `HolonRepresentable::to_holon_ast` takes &self — requires `T: Clone` for the consuming form, OR the wrapper trait needs `for<'a> HolonAST: From<&'a T>` reference-style conversion.

If sonnet finds a clean blanket form, include it. If not, OMIT the blanket and document with a comment: *"Manual `impl HolonRepresentable for T` per substrate-internal type; blanket impl deferred because Into<HolonAST> consumes self and would require T: Clone overhead at every send. Future arc may revisit if a clean blanket pattern surfaces."* This is HONEST Mode A — sonnet picks pragmatically.

### Risk 2 — HolonAST sub-variant for ToyType smoke probe

The smoke probe's `ToyType(i64)` needs `to_holon_ast` / `from_holon_ast` impls. Sonnet investigates the `holon` crate's HolonAST API briefly (looks at how `src/lower.rs:161` or `src/edn_shim.rs:1997` construct HolonAST values) + picks the simplest variant that carries an i64. The test verifies SHAPE not VSA semantics.

If sonnet can't find a clean simplest variant: STOP, report what HolonAST API they found, do NOT invent one. Orchestrator picks if surfaces.

### Risk 3 — `holon` crate import discovery

`use holon::HolonAST` is the precedent (see src/runtime.rs:45). Sonnet verifies the crate name is `holon` (not `holon-rs` or other) before importing. If the import fails, STOP + report.

### Risk 4 — Alphabetical insertion in lib.rs

`pub mod comms;` goes between `pub mod closure_extract;` (line 61) and `pub mod compose;` (line 62). Sonnet inserts exactly there; verifies cargo build catches any typo.

### Risk 5 — Cascade contract docs

The module-level doc references SHUTDOWN_RX, broadcast_fd, libc::pipe/read/write/poll/epoll/io_uring_*, and Slice 6's structural wall. These are FORWARD references to substrate primitives Slices 2/3/6 implement. Sonnet writes the docs as ASPIRATIONAL contract (what the cascade WILL mean once tiers ship); doesn't try to verify the substrate primitives at this slice.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `src/comms/mod.rs` minted (~80-120 LOC) | YES |
| 2 | `HolonRepresentable` trait defined with `to_holon_ast(&self) -> HolonAST` + `from_holon_ast(&HolonAST) -> Result<Self, WireError>` | YES |
| 3 | Blanket impl included OR omitted with documented reason | YES |
| 4 | `CommSender<T>` trait defined with `send(t)` + `close(self)` returning appropriate Results | YES |
| 5 | `CommReceiver<T>` trait defined with `recv` (cascade-aware in doc) + `try_recv` + `len` + `close` | YES |
| 6 | Error types defined: `SendError<T>(pub T)`, `RecvError`, `TryRecvError { Empty, Disconnected }`, `CloseError(String)`, `WireError(String)` | YES |
| 7 | `SelectOutcome<T>` enum with `Recv(usize, Result<T, RecvError>)` + `Shutdown` variants | YES |
| 8 | Module-level doc comment in `src/comms/mod.rs` covers cascade contract + audience separation | YES |
| 9 | `src/lib.rs` updated with `pub mod comms;` at alphabetically correct position (between closure_extract + compose) | YES |
| 10 | `tests/probe_comms_foundation.rs` minted with 3 smoke tests | YES |
| 11 | Probe `probe_slice1_holon_representable_compiles` PASS (ToyType roundtrip works) | YES |
| 12 | Probe `probe_slice1_error_types_construct` PASS | YES |
| 13 | Probe `probe_slice1_select_outcome_constructs` PASS | YES |
| 14 | `cargo build --release` clean (5 pre-existing dead_code warnings unchanged; no new warnings) | YES |
| 15 | Zero modifications outside `src/comms/mod.rs` + `src/lib.rs` (1 line) + `tests/probe_comms_foundation.rs` + SCORE doc | YES |
| 16 | Dirty tree intact (`src/fork.rs` + `src/spawn_process.rs` untouched) | YES |
| 17 | NO `wat_arc170_program_contracts` re-run (per `feedback_no_hang_vector_in_additive_scorecard`) | YES |

## Mode classification

- **Mode A:** all 17 criteria satisfied; foundation primitives shipped clean
- **Mode B (acceptable; honest surface):**
  - Risk 1 fires (clean blanket impl not possible): sonnet documents + omits; orchestrator accepts
  - Risk 2 fires (HolonAST sub-variant unclear): sonnet STOPs + reports; orchestrator picks
  - Risk 3 fires (holon crate import path differs): sonnet documents + uses correct path
- **Mode C (failure):**
  - Touched any file outside the 4-file scope
  - Touched the dirty tree (src/fork.rs / src/spawn_process.rs)
  - Implemented Sender/Receiver/Select types (that's Slice 2/3)
  - Added wat::comms::thread::* or wat::comms::process::* submodules
  - Ran wat_arc170_program_contracts
  - Committed the work

## Calibration metadata

- **Orchestrator confidence:** HIGH on Mode A first-attempt. The work is pure type/trait/error declarations + 3 smoke tests; no runtime behavior; no substrate edges. The blanket impl decision is the only judgment call, and either path (include or omit with reason) is honest Mode A.
- **Risk factors:**
  - HolonAST API discovery for the smoke probe (Risk 2) — bounded by existing src/lower.rs + src/edn_shim.rs patterns
  - Blanket impl Rust trait-bound subtlety (Risk 1) — mitigated by sonnet's "if not clean, omit with reason" path
- **Why this matters:** Slice 1 is the FOUNDATION the entire arc 214 builds on. Every subsequent slice (2/3/4/5/7/8) imports `crate::comms::*` and uses these traits. Getting the type-level shape right here means the implementation slices have clean ground to stand on.

## Tractability tiebreaker rationale (per `feedback_tractability_tiebreaker`)

Slice 1 is the FIRST stone; nothing precedes it. The arc 214 stone chain naturally starts here because traits + errors + cascade contract are the shapes Slices 2/3 implement. Within Slice 1: single coherent concern (foundation primitives); no further splitting needed at slice-decomposition level.

## Cross-references

- BRIEF-214-SLICE-1-FOUNDATION-PRIMITIVES.md — this stone's work order
- `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` — full arc 214 design; Slice 1 description
- `project_holon_universal_ast` — HolonAST as universal substrate form (the strange-loop closing for this slice)
- `feedback_no_hang_vector_in_additive_scorecard` — why no wat_arc170 verification
- `feedback_defect_fix_or_panic_never_revert` — dirty tree preservation
- `feedback_iterative_complexity` — single-coherent-concern stepping stone discipline
- `feedback_simple_forms_per_func` — keep helper functions small
- `feedback_test_first` — smoke probe tests verify shape before implementation lands
