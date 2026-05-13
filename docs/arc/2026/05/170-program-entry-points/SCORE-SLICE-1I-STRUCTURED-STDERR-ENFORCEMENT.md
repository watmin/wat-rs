# Arc 170 slice 1i — SCORE: substrate-wide structured-exit protocol

**Result:** 8/8 rows PASS.
**Runtime:** ~180 min sonnet (within hard cap; deadlock pivot required mid-session).
**Files changed:** `src/spawn_process.rs`, `src/fork.rs`, `src/runtime.rs`, `src/types.rs`, `wat/kernel/sandbox.wat`, `wat/kernel/hermetic.wat`, `wat/test.wat` + 3 new probe files.

**Workspace: 167 pass / 7 fail.** Delta from pre-slice: 0 net (same 7 pre-existing failures; the 5 svc failures now surface ACTUAL type-check diagnostics instead of "forked program exited 3").

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A | Custom panic hook installed in `spawn_process_child_branch` + `fork.rs::child_branch_from_source` (and `child_branch`); suppresses Rust default "thread '...' panicked" / "RUST_BACKTRACE" noise on fd 2 | ✓ `probe_no_default_rust_panic_noise_on_stderr` PASSES; no Rust-default handler lines in RunResult.stderr |
| B | `emit_structured_exit` is the SOLE source of `write_direct_to_stderr` for exit paths; no direct callers bypass it | ✓ Verified: `write_direct_to_stderr` is called only from `emit_structured_exit` + `emit_panics_to_stderr` helpers |
| C | `ProcessDiedError` enum has all variants: Panic / RuntimeError / ChannelDisconnected (existing) + StartupError / EntryFormFailure / MainSignature / BadReturn (minted); registered in both Rust runtime dispatch (`runtime.rs`) AND TypeEnv (`types.rs`) | ✓ All 7 variants present; EDN round-trip works via `extract-panics` |
| D | `probe_runtime_error_produces_structured_edn` PASSES — runtime-error path produces structured EDN; `failure.message` is actual runtime error text, NOT "forked program exited N" | ✓ PASSES |
| E | `probe_plain_panic_produces_structured_edn` PASSES — plain panic path (CapacityMode::Panic with dim_count=1) produces structured EDN | ✓ PASSES |
| F | `probe_no_default_rust_panic_noise_on_stderr` PASSES — Rust default panic handler output absent from `RunResult.stderr` lines; structured `#wat.kernel/ProcessPanics` line present | ✓ PASSES |
| G | Existing `probe_runtime_err_stderr_visibility` still PASSES | ✓ PASSES |
| H | Wat-side harness `(None chain)` fallback retired in `run-hermetic-driver` + `run-hermetic-with-io-driver` (test.wat); replaced with contract-violation `assertion-failed!`; svc workspace failures now show ACTUAL diagnostic (`4 type-check error(s)` about retired `()` unit spelling) instead of "forked program exited 3" | ✓ PASSES — verified with cargo test output |

## Honest deltas

### Pivot required mid-session: deadlock from over-aggressive None-arm retirement

Initial attempt retired the `(None chain)` fallback in ALL four locations:
- `test.wat::run-hermetic-driver`
- `test.wat::run-hermetic-with-io-driver`
- `wat/kernel/sandbox.wat::drive-sandbox`
- `wat/kernel/hermetic.wat::run-sandboxed-hermetic-ast`

`drive-sandbox` is called by `run-sandboxed` and `run-sandboxed-ast` which use `spawn-program` and `spawn-program-ast` — IN-PROCESS thread spawns, not forked processes. Thread-based spawns never write `#wat.kernel/ProcessPanics` to a subprocess stderr pipe; the error chain comes through the crossbeam channel directly. Retiring the None fallback for thread-based paths triggered the contract-violation assertion-failed for all thread-spawn-backed tests.

**Fix:** reverted the None arm in `sandbox.wat::drive-sandbox` to the original `(:wat::core::None err)` fallback with an explanatory comment. The retirement is correct only for the test.wat harness functions that exclusively drive forked children via `spawn-process`.

`hermetic.wat::run-sandboxed-hermetic-ast` uses `fork-program-ast` (a real fork). The retirement there is CORRECT and stayed — the forked child's panic hook IS installed; structured EDN IS emitted.

### TypeEnv gap: new ProcessDiedError variants not registered

Initial implementation added new variants to:
- `runtime.rs` Rust dispatch arms (ProcessDiedError/message + ProcessDiedError/to-failure)
- `runtime.rs` Value builder functions (process_died_error_startup_value etc.)

But omitted: the TypeEnv registration in `src/types.rs`. The `extract-panics` verb calls `edn_to_value` → `reconstruct_enum_tagged` → `types.get(":wat::kernel::ProcessDiedError")` → variant lookup. Without the new variants in the TypeEnv, `extract-panics` returned None for StartupError/EntryFormFailure/MainSignature/BadReturn payloads, triggering the contract-violation assertion for pre-existing svc failures.

**Fix:** added all 4 new variants to the `ProcessDiedError` EnumDef in `types.rs`. EDN round-trip now works for all 7 variants.

### wat syntax: comments inside match arm bodies are invalid

First attempt at restoring the `sandbox.wat` fallback embedded a multi-line comment between `(:wat::core::None` and `err)` inside the match arm form. The wat parser rejected this with "unexpected ')'" at the inner form's closing paren.

**Fix:** moved the comment block to the line before the inner `match` form. Wat comments (`;;`) are only valid between top-level forms or between list elements at the outer level — not inline within a form's argument sequence.

### Closing paren count error

After fixing the comment placement, had one extra `)` in the restored arm (6 instead of 5), causing another parse error. Fixed by matching the original `git show fd3318b:wat/kernel/sandbox.wat` paren count exactly.

## Mode B trigger: not fired

No path remained that violated the structured-stderr-only doctrine after both fixes:
1. TypeEnv registration → extract-panics round-trips all new variants
2. sandbox.wat fallback restored → in-thread spawns fall through to join-result chain

The 5 svc test failures still fail (pre-existing `()` unit type spelling defect) but now surface the actual type-check error in their RunResult failure message. This is the Row H expected outcome.
