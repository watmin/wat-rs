# Arc 138 F2 — Pre-handoff expectations

**Written:** 2026-05-03, AFTER drafting brief, BEFORE spawning sonnet.

**Brief:** `BRIEF-F2-SCHEMECTX.md`
**Targets:** src/rust_deps/mod.rs (trait def), src/check.rs (impl), crates/wat-telemetry-sqlite/src/{auto,cursor}.rs (callers), crates/wat-macros/src/codegen.rs (proc-macro emit), crates/wat-telemetry/src/shim.rs (caller).

## Setup — workspace state pre-spawn

- Baseline: F1 commit (TBD when F1 ships).
- 3 leftover Pattern E sites in src/check.rs::CheckSchemeCtx (lines 8398–8423).
- 16 caller sites across 4 external files using current trait signatures.
- 6/6 arc138 canaries pass.

## Hard scorecard (7 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | 6 files modified: src/rust_deps/mod.rs + src/check.rs + 4 external (cursor.rs, auto.rs, codegen.rs, shim.rs). No others. |
| 2 | Trait gains span params | 3 push_* methods gain `span: Span` parameter. |
| 3 | CheckSchemeCtx impl updated | 3 methods use threaded span; 3 rationale comments deleted. |
| 4 | All 16 callers updated | Pattern A/B real spans; Pattern E only for arity-0 cases (e.g., uuid::v4) with rationale. |
| 5 | Proc-macro emit produces compiling code | codegen.rs quote blocks emit `args[i].span().clone()` or equivalent; emitted code compiles and runs. |
| 6 | Workspace tests pass | All 6 arc138 canaries PASS; `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. |
| 7 | No commits | Working tree shows uncommitted modifications only. |

## Soft scorecard (3 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 8 | Span source quality | Type mismatches use args[i].span() (the offending arg); arity uses args[0].span() when args exist or Pattern E for arity-0. |
| 9 | Pattern E count | ≤ 2 sites (only `uuid::v4` and possibly one other arity-0 case). Down from 3 (the trait gap itself). |
| 10 | Honest report | Comprehensive ~400 words; per-file pattern distribution; proc-macro emit confirmation; honest deltas. |

## Independent prediction

- **Most likely (~60%):** 7/7 hard + 3/3 soft. Sonnet ships in 25-40 min. Pattern A dominates for type mismatches; arity uses args[0].span() or Pattern E for arity-0. Proc-macro emit lands cleanly.
- **Proc-macro emit subtlety (~20%):** sonnet investigates the codegen.rs quote blocks carefully; may need to read existing emit patterns to understand how to thread span. Adds 10 min.
- **Test regression in caller crates (~10%):** sonnet's caller updates miss one signature; cargo build catches; quick fix.
- **Helper-sig broadening in callers (~5%):** rare; most callers have args directly; broadening unlikely.
- **Cross-file regression (~5%):** sonnet accidentally edits the wrong file; cargo build catches.

## Methodology

After sonnet reports back:

1. Read this file FIRST.
2. `git diff --stat` → 6 files modified.
3. `grep -c "Span::unknown()" src/check.rs` → should drop by 3 from baseline (CheckSchemeCtx 3 sites).
4. `grep -c "// arc 138: no span" src/check.rs` → should drop by ~3.
5. Run all 6 canaries; workspace tests.
6. Spot-check 5 caller sites across the 4 external files.
7. Verify proc-macro emit by reading codegen.rs.
8. Score; commit + push; queue F3 (WatReader/WatWriter).

## What this slice tells us

- All clean → trait expansion as a fix pattern is durable. F3 (WatReader/WatWriter) dispatches with confidence.
- Proc-macro emit handled cleanly → calibration data on sonnet's codegen.rs work.
