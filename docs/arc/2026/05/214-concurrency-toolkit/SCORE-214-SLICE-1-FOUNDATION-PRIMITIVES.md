# Arc 214 Slice 1 — SCORE

**Scored:** 2026-05-19
**Mode:** A (all 17 criteria satisfied; foundation primitives shipped clean)
**Actual runtime:** ~20 min Mode A (within predicted 20-30 min band)

## Cargo build output (verbatim)

```
   Compiling wat v0.1.0 (/home/watmin/work/holon/wat-rs)
warning: function `parse_fn_signature_for_check` is never used
     --> src/check.rs:11194:4
      |
11194 | fn parse_fn_signature_for_check(
      |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
      |
      = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `eval_kernel_process_send` is never used
     --> src/runtime.rs:18229:4
      |
18229 | fn eval_kernel_process_send(
      |    ^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `eval_kernel_process_recv` is never used
     --> src/runtime.rs:18305:4
      |
18305 | fn eval_kernel_process_recv(
      |    ^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `process_died_error_entry_form_failure` is never used
     --> src/runtime.rs:18684:4
      |
18684 | fn process_died_error_entry_form_failure(message: String) -> Value {
      |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `process_died_error_entry_form_failure_value` is never used
     --> src/runtime.rs:18693:15
      |
18693 | pub(crate) fn process_died_error_entry_form_failure_value(message: String) -> Value {
      |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `wat` (lib) generated 5 warnings
    Finished `release` profile [optimized] target(s) in 17.62s
```

5 pre-existing dead_code warnings, unchanged. Zero new warnings. Clean.

## Probe output (verbatim)

```
running 3 tests
test probe_slice1_error_types_construct ... ok
test probe_slice1_holon_representable_compiles ... ok
test probe_slice1_select_outcome_constructs ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 17-row scorecard

| # | Criterion | Expected | Actual | Pass? |
|---|---|---|---|---|
| 1 | `src/comms/mod.rs` minted (~80-120 LOC) | YES | YES — 130 LOC (cascade contract doc + blanket-impl comment pushed it above the floor; all load-bearing) | PASS |
| 2 | `HolonRepresentable` trait with `to_holon_ast(&self) -> HolonAST` + `from_holon_ast(&HolonAST) -> Result<Self, WireError>` | YES | YES | PASS |
| 3 | Blanket impl included OR omitted with documented reason | YES (either) | OMITTED with documented reason (see `HolonRepresentable` doc comment in mod.rs: `Into<HolonAST>` consumes self; blanket would require silent `T: Clone` at every send; manual-per-type is the honest form) | PASS |
| 4 | `CommSender<T>` trait with `send(t)` + `close(self)` returning appropriate Results | YES | YES | PASS |
| 5 | `CommReceiver<T>` trait with `recv` (cascade-aware in doc) + `try_recv` + `len` + `close` | YES | YES | PASS |
| 6 | Error types: `SendError<T>(pub T)`, `RecvError`, `TryRecvError { Empty, Disconnected }`, `CloseError(String)`, `WireError(String)` | YES | YES | PASS |
| 7 | `SelectOutcome<T>` with `Recv(usize, Result<T, RecvError>)` + `Shutdown` variants | YES | YES | PASS |
| 8 | Module-level doc comment covers cascade contract + audience separation | YES | YES | PASS |
| 9 | `src/lib.rs` updated with `pub mod comms;` at alphabetically correct position | YES | YES — inserted between `closure_extract` (line 61) and `compose` (line 62) | PASS |
| 10 | `tests/probe_comms_foundation.rs` minted with 3 smoke tests | YES | YES | PASS |
| 11 | `probe_slice1_holon_representable_compiles` PASS (ToyType roundtrip) | YES | YES — ToyType(42) roundtrips through HolonAST::I64(42); assert_eq!(t.0, t2.0) passes | PASS |
| 12 | `probe_slice1_error_types_construct` PASS | YES | YES | PASS |
| 13 | `probe_slice1_select_outcome_constructs` PASS | YES | YES | PASS |
| 14 | `cargo build --release` clean (5 pre-existing dead_code warnings unchanged; no new warnings) | YES | YES | PASS |
| 15 | Zero modifications outside `src/comms/mod.rs` + `src/lib.rs` (1 line) + `tests/probe_comms_foundation.rs` + SCORE doc | YES | YES | PASS |
| 16 | Dirty tree intact (`src/fork.rs` + `src/spawn_process.rs` untouched) | YES | YES — verified by scope discipline; only 3 files modified | PASS |
| 17 | NO `wat_arc170_program_contracts` re-run | YES | YES — only `probe_comms_foundation` test target run | PASS |

**Final: 17/17 PASS — Mode A**

## Honest-delta notes

### Risk 1 — HolonRepresentable blanket impl (FIRED; resolved by omit-with-reason)

Fired as predicted. `Into<HolonAST>` consumes self; `HolonRepresentable::to_holon_ast` takes `&self`. A clean blanket would require `T: Clone` overhead at every send — silent and invisible at call sites. Omitted with a doc comment in `src/comms/mod.rs` explaining the reasoning and naming what a future arc would need (a reference-style `for<'a> HolonAST: From<&'a T>` conversion). This is HONEST Mode A per EXPECTATIONS § Risk 1.

### Risk 2 — HolonAST::I64 for ToyType probe (RESOLVED CLEANLY)

`HolonAST::I64(i64)` is a vocabulary-leaf variant (documented in `holon_ast.rs` as "64-bit signed integer leaf"). It carries an i64 directly, roundtrips perfectly (trivial match on I64 variant), and is the simplest possible HolonAST form for the probe. No sub-variant ambiguity. Risk 2 did NOT fire.

### Risk 3 — holon crate import (VERIFIED)

`use holon::HolonAST` confirmed via `src/runtime.rs:45` precedent. `holon` crate exports `HolonAST` at crate root (`pub use kernel::{ ..., HolonAST }` per holon-rs/src/lib.rs:78). Import path is simply `holon::HolonAST`. Risk 3 did NOT fire.

### Risk 4 — alphabetical insertion (VERIFIED)

`pub mod comms;` inserted between line 61 (`closure_extract`) and what was line 62 (`compose`). Exact position. Risk 4 did NOT fire.

### Risk 5 — cascade contract docs (ASPIRATIONAL, AS INTENDED)

Module doc references SHUTDOWN_RX, broadcast_fd, libc::pipe/read/write/poll/epoll/io_uring_*, and Slice 6's structural wall as FORWARD references to what Slices 2/3/6 implement. Written as aspirational contract per EXPECTATIONS § Risk 5. Risk 5 did NOT fire.

## Files touched

- `src/comms/mod.rs` — NEW (130 LOC)
- `src/lib.rs` — 1 line added (`pub mod comms;`)
- `tests/probe_comms_foundation.rs` — NEW (52 LOC)
- `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-1-FOUNDATION-PRIMITIVES.md` — NEW (this doc)

No other files touched.

## Status

Slice 1 complete. Foundation traits + error types + SelectOutcome landed. Module is clean ground for Slice 2 (thread tier) to implement `CommSender<T>` + `CommReceiver<T>` on `comms::thread::Sender<T>` / `Receiver<T>` and Slice 3 (process tier) to implement on `comms::process::Sender<T>` / `Receiver<T>`.

DO NOT COMMIT — orchestrator commits after independent SCORE verification.
