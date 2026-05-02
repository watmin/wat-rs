# Arc 138 Slice 3a — Pre-handoff expectations

**Written:** 2026-05-03, AFTER drafting brief, BEFORE spawning sonnet.

**Brief:** `BRIEF-SLICE-3A.md`
**Target:** primarily `src/runtime.rs`; transient compile-keep stubs in 10 external src/ files.

## Setup — workspace state pre-spawn

- Baseline (commit `3d5420b`): wat-rs workspace `cargo test --release --workspace` exit=0 excluding lab. Slice 1 + slice 2 shipped.
- `Span::unknown()` in src/runtime.rs PRE-SLICE: 54 instances — all pre-existing legitimate uses for SYNTHETIC AST construction (struct-new dispatchers, enum constructors, lambda body wrappers). These DO NOT COUNT toward the sweep; they're not error emission sites.
- `Err(RuntimeError::*)` emission sites in src/runtime.rs: 489. In external files: ~100 (across io, time, marshal, fork, string_ops, spawn, assertion, edn_shim, hologram, freeze).
- TailCall already has `call_span: Span`; SandboxScopeLeak (arc 140) already has two spans. These serve as IN-FILE worked examples.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | `src/runtime.rs` modified primarily; up to 10 external src files modified ONLY for compile-keep transient `Span::unknown()` stubs (each marked `// arc 138 slice 3b: span TBD`). No tests file changes outside the canary. |
| 2 | All 22 user-facing variants gain `span` | Per the BRIEF table. Tuple variants: `(String, Span)` extension. Struct variants: `span: Span` field. Unit variants: convert to single-field tuple `(Span)`. Internal signals + UserMainMissing + EvalVerificationFailed + SandboxScopeLeak + TailCall LEFT UNCHANGED. |
| 3 | All 22 Display arms prefix coords | New `span_prefix` helper near top of src/runtime.rs (mirror check.rs / types.rs). Each of the 22 arms prefixes `{span_prefix(span)}` when non-unknown. Internal signals' arms unchanged. |
| 4 | Emission sites in src/runtime.rs thread real spans | ≥ 440 of ~489 sites use real spans (≥ 90%). Each leftover `Span::unknown()` (excluding synthetic-AST 54 baseline) carries `// arc 138: no span — <reason>` rationale. |
| 5 | Workspace tests pass | `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. Critical: external file transient stubs must keep ALL crates compiling. |
| 6 | Canary added + passes | A new test in `src/runtime.rs::tests` triggers any user-facing RuntimeError variant and asserts `<test>:` substring in rendered Display. Passes. |
| 7 | No commits | Working tree shows uncommitted modifications only. |
| 8 | Honest report | ~500 words; counts per file; variant list; Display arms; helper signatures broadened; canary verification; substrate observations including any `_with_span` sibling additions; transient external-file stub counts; four questions. |

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | Span source quality in src/runtime.rs | Sites use the OFFENDING node's span (arg / call form / condition expression) rather than always reaching for the enclosing function's span. |
| 10 | External transient marking | Every external `Span::unknown()` added has `// arc 138 slice 3b: span TBD` immediately above or on same line. Slice 3b agent grep-finds them mechanically. |
| 11 | Workspace runtime | `cargo test --release --workspace` runtime ≤ baseline + 15%. Span field additions shouldn't slow runtime materially. |
| 12 | Honest delta on _with_span pattern | Any public `apply_function` / `eval` / `eval_call` signature change is named explicitly in honest deltas. If sonnet adds `_with_span` siblings to preserve public API, count them. |

## Independent prediction

This is the largest sonnet engagement of the arc. ~489 src/runtime.rs sites + variant restructuring + 22 Display arms + 10 external file stubs + canary. Predicted runtime: 45-75 min based on slice 1 finish (200 sites = 30 min) + slice 2 (35 sites + variants = 10 min) extrapolation.

- **Most likely (~55%):** 8/8 hard + 4/4 soft. Sonnet ships in 50-70 min. ≥ 440 sites real-spanned. Modest helper-signature additions; ~10 external stubs marked.
- **Helper signatures broadened heavily (~25%):** sonnet threads spans through `apply_function` / `eval` / `eval_call` and creates 2-4 `_with_span` siblings to preserve public API. Hard 8/8 + Soft 12 PASS+.
- **Partial completion (~10%):** sonnet ships variants + Display + ~60-70% of emissions; reports honestly that the remainder needs follow-up. Score per-row; possibly ship-as-is + slice 3a-finish for the gap.
- **External file regression (~5%):** sonnet's transient stubs miss a callsite; an external crate fails to compile. Diagnose; either fix on the spot or note for slice 3a-finish.
- **Variant restructure breaks something subtle (~5%):** an existing pattern match uses positional destructure that doesn't extend cleanly; e.g., `RuntimeError::UnboundSymbol(s) =>` becomes `RuntimeError::UnboundSymbol(s, _) =>` and one site is missed. Cargo build catches it; sonnet fixes.

## Methodology

After sonnet reports back:

1. Read this file FIRST.
2. `git diff --stat` → src/runtime.rs primary; up to 10 external files (each small, transient stubs).
3. `grep -c "span: Span::unknown()" src/runtime.rs` → measure (should be ≤ 50 of EMISSION sites; the 54 synthetic-AST sites pre-existing aren't counted).
4. `grep -c "// arc 138 slice 3b: span TBD" src/` → matches the external-file transient count.
5. Read variant defs — confirm all 22 changed; internal signals + DON'T-TOUCH list unchanged.
6. Read Display arms — confirm 22 prefixed; internal arms unchanged.
7. Spot-check 10-20 emission sites — confirm real spans where possible.
8. `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading | head -5` → empty.
9. Run canary by name — passes.
10. Score each row; write `SCORE-SLICE-3A.md`.
11. If clean → commit + push, queue slice 3b (external file sweep).

## What this slice tells us

- All clean → arc 138's pattern propagates to the LARGEST file in the substrate. Slice 3b (external sweep) dispatches with high confidence.
- Helper signatures broadened heavily → real substrate observation about apply_function / eval boundary span propagation. Document for future slice planning.
- Partial completion → slice was too large for one engagement; refine: split into 3a-1 (variants + Display) and 3a-2 (sweep). Re-spawn for the gap.
- Hard fail — investigate. Likely: variant restructuring missed a case; iterate.

## What follows

- Score → commit slice 3a → write slice 3b BRIEF (external file sweep; ~100 sites across 10 files; replace transient `Span::unknown()` stubs with real spans). Spawn sonnet → score → continue to slices 4-6.
