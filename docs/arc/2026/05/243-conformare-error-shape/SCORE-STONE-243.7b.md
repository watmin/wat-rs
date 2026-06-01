# SCORE — Stone 243.7b — eval-loop control signal channel split (FINISH)

## Phase A — substrate refactor verified

**Mode:** A (finish/recovery — prior agent dropped mid-cascade)

**Scope:** Channel split ONLY. `RuntimeError` → diagnostic-only. `EvalSignal` (the trio) + `EvalBreak { Diagnostic, Signal }` minted. No Pattern A shape change to diagnostics (that is 243.7c). Behavior-preserving.

### Per-step audit

| Step | Status | Notes |
|---|---|---|
| S1 — Mint `EvalSignal` + `EvalBreak` + `From` | COMPLETE (prior agent) | `EvalSignal { TailCall, TryPropagate, OptionPropagate }` + `EvalBreak { Diagnostic(RuntimeError), Signal(EvalSignal) }` + `impl From<RuntimeError> for EvalBreak` minted in runtime.rs |
| S2 — Move signal Display + EDN arms | COMPLETE (prior agent) | 3 signal arms moved from `impl Display for RuntimeError` → `impl Display for EvalSignal`; 3 EDN arms moved to `EvalSignal` serializer in runtime_error_edn.rs |
| S3 — Remove trio from RuntimeError | COMPLETE (prior agent) | `TailCall`, `TryPropagate`, `OptionPropagate` variants deleted from RuntimeError |
| S4 — 5 construction sites | COMPLETE (prior agent) | `TailCall` @ 4695; `TryPropagate` @ 14358/14361; `OptionPropagate` @ 14408/14411 → `Err(EvalBreak::Signal(EvalSignal::…))` |
| S5 — 2 catch boundaries | COMPLETE (prior agent) | `apply_function` trampoline (@ 21881/21894/21897/21900) + propagation handler (@ 25036/25064) → match `EvalBreak::Signal(EvalSignal::…)` / `EvalBreak::Diagnostic(re)` |
| S6 — Cascade (subgraph flip to EvalBreak) | COMPLETE (prior agent + finish) | Eval subgraph functions flipped to `Result<_, EvalBreak>`; leaf callers lift via `From` at `?` |
| **Fix A — Pattern-position `.into()` (prior agent tool corruption)** | **COMPLETE (finish)** | 10 sites total: 4 in test match arms (32861/32891/32894/32916 → `EvalBreak::Diagnostic(RuntimeError::…)`); 6 in lib tests (`matches!` + match arm bodies in test code) wrapped in `EvalBreak::Diagnostic(…)` |
| **Fix B — EvalBreak containment at freeze boundary** | **COMPLETE (finish)** | `register_defines`, `register_stdlib_defines`, `register_defalias`, `preregister_*`, `register_struct/enum/newtype/type_predicates_methods`, `parse_defclause_form`, `parse_defclause_clause`, `parse_type_keyword` — all pure registration/parse functions wrongly flipped to `EvalBreak` by prior agent; flipped back to `RuntimeError`. `register_runtime_defs` (legitimately on signal path via `eval_inner`) stays `EvalBreak`; freeze.rs collapses at boundary via match `{ Diagnostic(re) => Runtime(re), Signal(_) => unreachable!("interpreter bug") }` |
| **Fix C — dispatch match arm type mismatches** | **COMPLETE (finish)** | `dispatch_keyword_head_value` match block: all leaf calls returning `RuntimeError` (string_ops, io, time, edn_shim, thread_io) annotated with `.map_err(Into::into)` to lift to `EvalBreak`; `eval_apply`'s `apply_function` calls lifted similarly |
| **Fix D — unused import EvalSignal** | **COMPLETE (finish)** | `EvalSignal` removed from runtime_error_edn.rs import (EDN serializer for signals exists but import wasn't used at call sites) |
| **Fix E — wat-telemetry-sqlite boxing** | **COMPLETE (finish)** | 3 `TypeMismatch.got` fields in `crates/wat-telemetry-sqlite/` missed the 243.7a boxing (needed `Box::new(ValueSnapshot::of(…))`); fixed for `cargo build --release --tests` clean |
| S7 — check.rs doc-comment updates | COMPLETE | Lines 8371, 8488, 14487: `RuntimeError::TryPropagate`/`OptionPropagate` → `EvalSignal::…` in prose; also updated inline doc-comments at 4602, 4691, 14362, 14415, 21763 in runtime.rs |
| S8 — `tools/transform-evalbreak/` deleted | COMPLETE | `rm -rf tools/transform-evalbreak/`; `tools/` itself (empty) removed. Tool never lands. |

### Cascade audit table

| File | Sites / type | Category |
|---|---|---|
| `src/runtime.rs` | ~240+ signature flips (subgraph) + 10 pattern fixes + 37 leaf arm `.map_err` + 26 test `matches!`/match fixes + doc updates | Core subgraph + boundary + test parity |
| `src/runtime_error_edn.rs` | Remove `EvalSignal` import (unused) | Cleanup |
| `src/freeze.rs` | Add `EvalBreak` import; collapse `register_runtime_defs` boundary | Freeze boundary containment |
| `src/check.rs` | 3 doc-comment lines | Prose only (no code) |
| `crates/wat-telemetry-sqlite/src/auto.rs` | 1 `Box::new` wrap on `TypeMismatch.got` | Pre-existing 243.7a cascade residue |
| `crates/wat-telemetry-sqlite/src/cursor.rs` | 2 `Box::new` wraps on `TypeMismatch.got` | Pre-existing 243.7a cascade residue |

### Cascade waterfall

| Iteration | Errors |
|---|---|
| Handoff state (prior agent dropped) | 14 compile errors (4 pattern `.into()` syntax + 1 E0308 freeze + 6 E0277 freeze + 1 E0308 match + 1 E0277 EvalBreak→EvalBreak + 1 E0308 mismatched) |
| After pattern-position `.into()` fix (4 sites) | 10 errors |
| After freeze boundary containment (flip ~10 fns back to RuntimeError) | 7 errors |
| After remaining E0308/E0277 fixes (dispatch match arms, eval_apply) | 2 errors |
| After parse_type_slot + String/ alias fixes | 1 error |
| After io:: and time:: leaf arm fixes | 0 compile errors |
| After lib test fixes (test match patterns, 26 sites) | 0 compile + test errors |
| Final | **0 errors, 895/0/1 lib parity, 4/0 probe** |

### Boundary decision (Fix B — the key architectural question)

`register_defines` / `register_stdlib_defines` / `register_defalias` / `preregister_*` / `register_struct/enum/newtype_methods` / `register_type_predicates` / `parse_defclause_form` / `parse_defclause_clause` / `parse_type_keyword` are **startup-layer registration functions** with zero signal-path calls. The prior agent's transform tool over-propagated `EvalBreak` into them because they happened to call each other in a chain. Reversing these to `RuntimeError` is correct — the contract rule applies: only flip to `EvalBreak` if the function transitively constructs or propagates a signal. None of these do.

`register_runtime_defs` + `register_runtime_defs_form` LEGITIMATELY return `EvalBreak` because they call `eval_inner` (which constructs `TailCall`/`TryPropagate`/`OptionPropagate` on the signal path). The collapse at `freeze.rs` uses `unreachable!()` for `Signal` (established codebase idiom for proven-impossible internal states). Why it is unreachable: `TryPropagate`/`OptionPropagate` are rejected by the checker — top-level `?`/option-propagation is flagged "used outside any function body" at `check.rs:8406`/`:8520`, and the check pass runs before `register_runtime_defs` in the freeze pipeline. `TailCall` is trampolined inside `apply_function` and cannot escape. A `Signal` here means the checker or eval subgraph is mis-wired.

### 5 construction + 2 catch boundaries (base contract)

| Site | Status |
|---|---|
| `TailCall` construction @ runtime.rs:4695 | COMPLETE (prior agent) → `EvalBreak::Signal(EvalSignal::TailCall{…})` |
| `TryPropagate` construction @ 14358/14361 | COMPLETE (prior agent) → `EvalBreak::Signal(EvalSignal::TryPropagate(…))` |
| `OptionPropagate` construction @ 14408/14411 | COMPLETE (prior agent) → `EvalBreak::Signal(EvalSignal::OptionPropagate)` |
| `apply_function` trampoline catch @ 21881/21894/21897/21900 | COMPLETE (prior agent) → `Err(EvalBreak::Signal(EvalSignal::…))` |
| Propagation handler catch @ 25036/25064 | COMPLETE (prior agent) → `Err(EvalBreak::Signal(EvalSignal::…))` |

### Final metrics

| Metric | Value |
|---|---|
| `cargo build --release -p wat` | 0 errors |
| FM 2-bis probe (`probe_arc243_stone7b_signal_split`) | **4 / 0** |
| `cargo test --release --lib -p wat` | **895 / 0 / 1** (parity — behavior-preserving) |
| `cargo build --release --tests` | clean (0 errors) |
| `cargo clippy --release -p wat \| grep -c result_large_err` | **0** |
| `grep -rn "RuntimeError::(TailCall\|TryPropagate\|OptionPropagate)" src/` | **0** (trio fully excised from RuntimeError) |
| Ephemeral tool deleted | `tools/transform-evalbreak/` + `tools/` removed |
| Behavior-identical | TCO/try/option lib tests unchanged; 895 parity confirms no semantic drift |

### Trap-doors encountered

| # | Trap-door | Resolution |
|---|---|---|
| TA | Prior agent's transform tool appended `.into()` inside match PATTERNS — illegal Rust syntax (E0531 / E0023) | Fixed by wrapping: `Err(RuntimeError::X.into())` → `Err(EvalBreak::Diagnostic(RuntimeError::X))` at all 10 pattern sites |
| TB | EvalBreak over-propagated into startup layer (`freeze.rs` 6×  E0277) | Identified boundary functions (no signal calls); flipped ~12 fns back to `RuntimeError`; freeze boundary collapses via match |
| TC | `dispatch_keyword_head_value` match block type mismatch (leaf arm returns `RuntimeError`, function returns `EvalBreak`) | Added `.map_err(Into::into)` to ~37 leaf arm calls (string_ops, io, time, edn_shim, thread_io) |
| TD | `wat-telemetry-sqlite` `TypeMismatch.got` missed 243.7a boxing | `Box::new(ValueSnapshot::of(…))` at 3 sites; needed for `cargo build --release --tests` clean |
| TE | Lib test match patterns on `eval_expr/run` results still matched raw `RuntimeError` variants (E0308 in `--lib` tests) | Wrapped 26 test `matches!(err, RuntimeError::…)` / `match err { RuntimeError::…` sites → `EvalBreak::Diagnostic(RuntimeError::…)` |

### Structural verification

| Check | Result |
|---|---|
| `pub enum EvalSignal` present in runtime.rs | `src/runtime.rs:2094` |
| `pub enum EvalBreak` present in runtime.rs | `src/runtime.rs:2143` |
| `impl From<RuntimeError> for EvalBreak` present | `src/runtime.rs:2153` |
| `EvalSignal` Display impl present | `src/runtime.rs:2118` |
| Trio variants GONE from `RuntimeError` | 0 matches in grep |
| `register_defines` returns `RuntimeError` | `src/runtime.rs:2618` |
| `register_runtime_defs` returns `EvalBreak` (legitimately on signal path) | `src/runtime.rs:3362` |
| freeze.rs collapses `EvalBreak` at boundary | `src/freeze.rs:405-415` |
| `tools/` deleted | confirmed via ls |

---

**243.7b CLOSED at the CHANNEL-SPLIT bar.** `RuntimeError` is now diagnostic-only. A control signal is structurally unable to masquerade as a located diagnostic. Stepping stone for 243.7c (Pattern A retrofit on the pure diagnostic set).
