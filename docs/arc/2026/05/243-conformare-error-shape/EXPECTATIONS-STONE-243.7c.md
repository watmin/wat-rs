# EXPECTATIONS — Stone 243.7c — RuntimeError → Pattern A

Independent scorecard for `DESIGN-STONE-243.7c.md` / `BRIEF-STONE-243.7c.md`. Written before the strike; scored against the orchestrator's OWN re-run.

## Baseline (pinned pre-strike, FM 9)

- `cargo test --release --lib -p wat` → **895 passed / 0 failed / 1 ignored**.
- `cargo test --release --test probe_arc243_stone7b_signal_split` → **4/0** (the EvalBreak wrap, must stay intact).
- 7c probe disconfirms at HEAD: **7 errors** — `E0432` (RuntimeErrorKind unresolved) + `E0574`×5 (RuntimeError is enum, not struct) + `E0609` (no `.span`). RuntimeError + Span resolve clean (gap isolated). ✅ verified.
- 243.7b shipped clean (`62355866`/`2c1019ed`); clippy `result_large_err` 0.

## Scorecard (row · command · expected)

| # | Row | Command | Expected |
|---|---|---|---|
| 1 | 7c probe passes | `cargo test --release --test probe_arc243_stone7c_runtimeerror_pattern_a` | **4/0** |
| 2 | Lib parity | `cargo test --release --lib -p wat` | **895/0/1** (behavior-preserving) |
| 3 | 7b probe still passes (wrap intact) | `cargo test --release --test probe_arc243_stone7b_signal_split` | **4/0** |
| 4 | Tests build clean | `cargo build --release --tests` | clean |
| 5 | clippy not regressed | `cargo clippy --release -p wat 2>&1 \| grep -c result_large_err` | **0** |
| 6 | Pattern A shape minted | `grep -n "pub struct RuntimeError" src/runtime.rs` + `grep -n "pub enum RuntimeErrorKind" src/runtime.rs` | **1 each** |
| 7 | No scratch crate | `git status --porcelain` + `ls tools/ 2>&1` | ephemeral Rust tool DELETED; no `tools/` |
| 8 | Behavior identical | read Display-split + a sample of cascade diffs | messages unchanged; pure span-relocation |

Rows 2 + 3 + 8 are LOAD-BEARING (independent re-run + diff read, not the returned SCORE).

## Runtime-band prediction

**120–240 min Mode A.** STOP at 480 min (the largest cascade in the arc). Ephemeral **Rust** Cargo tool MANDATORY for the ~1186-site construction reshape (Python/shell blocked). Anchor: 243.6a (459-site CheckError Pattern A via `transform-checkerror` ephemeral tool). 7c ≈ 2.5× that; expect the tool to do the bulk, hand-fixes for match-site destructuring + residue.

## Trap-doors

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | ~1186-site cascade volume | cargo fail-count | substrate-as-teacher; ephemeral **Rust** tool; waterfall to 0 |
| **T2** | Multi-span primary ambiguity | the §contract | SandboxScopeLeak=call_span, PostconditionFailed=body_span |
| **T3** | Freeze pair has no span | construct `Span::unknown()` | outer unknown + honest elision (probe contract 4) |
| **T4** | `EvalBreak::Diagnostic(RuntimeError)` wrap breaks | 7b probe (row 3) | the wrap holds a value; struct reshape is transparent |
| **T5** | `result_large_err` shifts on the struct | clippy (row 5) | box the large kind payload; no `#[allow]` |
| **T6** | Payload field dropped in reshape | probe + cargo | preserve every field on the kind variant |
| **T7** | Cross-crate sites (wat-telemetry-sqlite) | cargo --tests | reshape reaches them; cargo names; fix in-place |
| **T8** | Python reflex on the cascade tool | the SCORE / git | Rust Cargo binary ONLY (named imperatively in the BRIEF + spawn prompt) |

## Score methodology

After the strike: re-run rows 1–7 locally; READ the Display-split + a sample of construction-site diffs to confirm row 8 (messages unchanged — pure span relocation). Verify the ephemeral Rust tool is deleted (`git status`, no `tools/`). A failed row 2/3 or a row-8 message change = Mode B (reject + reland). Write `SCORE-STONE-243.7c.md` mirroring `SCORE-STONE-243.6a.md`. Commit STRIKE-READY artifacts BEFORE spawning; commit the stone on green AFTER scoring.
