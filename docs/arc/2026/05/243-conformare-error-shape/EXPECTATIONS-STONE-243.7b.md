# EXPECTATIONS — Stone 243.7b — eval-loop signal split

Independent scorecard for `DESIGN-STONE-243.7b.md` / `BRIEF-STONE-243.7b.md`. Written before the strike; scored against the orchestrator's OWN re-run.

## Baseline (pinned pre-strike, FM 9)

- `cargo test --release --lib -p wat` → **895 passed / 0 failed / 1 ignored** (the parity number).
- Probe disconfirms at HEAD: **single `E0432`** (`no EvalSignal/EvalBreak in runtime`); `RuntimeError` + import path resolve clean (gap isolated). ✅ already verified.
- No `From<…> for RuntimeError` impl exists (T3 risk EMPTY).
- 243.7a left `result_large_err` at **0** (function/ + rust_deps/ clippy-clean).

## Scorecard (row · command · expected)

| # | Row | Command | Expected |
|---|---|---|---|
| 1 | Probe passes post-stone | `cargo test --release --test probe_arc243_stone7b_signal_split` | **4 / 0** |
| 2 | Lib parity held | `cargo test --release --lib -p wat` | **895 / 0 / 1** (UNCHANGED — behavior-preserving) |
| 3 | Tests build clean | `cargo build --release --tests` | clean |
| 4 | Trio gone from RuntimeError | `grep -nE "RuntimeError::(TailCall\|TryPropagate\|OptionPropagate)" src/` | **0** |
| 5 | EvalSignal owns the trio | `grep -nE "EvalSignal::(TailCall\|TryPropagate\|OptionPropagate)" src/` | **≥ 5** (5 construction + 2 catch boundaries) |
| 6 | From boundary minted | `grep -n "impl From<RuntimeError> for EvalBreak" src/runtime.rs` | **1** |
| 7 | clippy not regressed | `cargo clippy --release -p wat 2>&1 \| grep -c result_large_err` | **0** (box the Diagnostic arm if it fires; no `#[allow]`) |
| 8 | No scratch crate | `git status --porcelain` | authorized files only (runtime.rs, runtime_error_edn.rs, check.rs, probe); ephemeral tool DELETED |
| 9 | Behavior identical | orchestrator reads the catch-boundary diffs | the trampoline + try/option handlers do the SAME thing over `EvalBreak::Signal(...)`; no semantics change |

Rows 2 + 9 are LOAD-BEARING (verify by independent re-run + diff read, not the returned SCORE).

## Runtime-band prediction

**90–180 min Mode A.** STOP at 360 min (2× upper-bound → ScheduleWakeup). Mint types + move Display/EDN impls + flip the signal subgraph return types + cascade-to-green. The variable is the subgraph size (subset of runtime.rs's 432 `Result<_, RuntimeError>` sigs); the `From`-at-leaf-boundary bounds it well below 432. Ephemeral Cargo tool if the mechanical flip is ≥ ~50 sites (build → use → DELETE).

Calibration anchors: 243.6a (459-site Rust-syntax cascade via `transform-checkerror` ephemeral tool); 241.11 (271-site cascade via `fix-defines`, ~98 min UNDER its 120–240 band). A return-type flip is more uniform than a struct-reshape → expect the cascade to run clean if the type boundary is defined first.

## Trap-doors (enumerated)

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Signal-path fn missed (stays RuntimeError) | cargo type error at catch boundary | flip to EvalBreak (it's on the path) |
| **T2** | Leaf wrongly flipped to EvalBreak → cascade balloons | review: leaf has no signal construction | revert to RuntimeError; `?` lifts via From |
| **T3** | `From<RuntimeError> for EvalBreak` collides | cargo E0119 | EMPTY (no existing From<…> for RuntimeError) |
| **T4** | Trio EDN/Display arms orphaned/duplicated | cargo + runtime_error_edn test | move all 3 arms to EvalSignal |
| **T5** | TCO trampoline semantics drift over EvalBreak | cargo + lib parity (row 2) | pure rehoming of the same match; STOP if it requires behavior change |
| **T6** | apply_function sig change ripples to non-eval callers | cargo fail-count | follow cascade; callers are within the eval subgraph |
| **T7** | `EvalBreak::Diagnostic(RuntimeError)` re-trips `result_large_err` | clippy (row 7) | box the arm → `Diagnostic(Box<RuntimeError>)` + update From + match sites; NO `#[allow]` |

## Score methodology

After the strike: re-run rows 1–8 locally; READ the catch-boundary + construction-site diffs to confirm row 9 (behavior identical — the match arms do the same work over the renamed channel). A failed row 2 or a row-9 semantics change = Mode B (the split changed behavior; reject + reland). Write `SCORE-STONE-243.7b.md` mirroring `SCORE-STONE-243.6a.md`. Commit STRIKE-READY artifacts (DESIGN + probe + BRIEF + EXPECTATIONS) BEFORE spawning; commit the stone on green AFTER scoring.
