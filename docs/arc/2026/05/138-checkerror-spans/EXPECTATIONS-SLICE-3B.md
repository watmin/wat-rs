# Arc 138 Slice 3b — Pre-handoff expectations

**Written:** 2026-05-03, AFTER drafting brief, BEFORE spawning sonnet.

**Brief:** `BRIEF-SLICE-3B.md`
**Target:** 15 external files (10 src/ + 3 crates/wat-telemetry-* + crates/wat-macros)

## Setup — workspace state pre-spawn

- Baseline (commit `371e831`): wat-rs workspace `cargo test --release --workspace` exit=0 excluding lab. Slice 3a + 3a-finish shipped.
- Pre-sweep marker count: **156** literal `// arc 138 slice 3b: span TBD` across 15 files (verified at commit `371e831`):
  - src/io.rs (36), src/time.rs (32), crates/wat-telemetry-sqlite/src/auto.rs (18), src/rust_deps/marshal.rs (17), src/fork.rs (14), src/string_ops.rs (13), crates/wat-telemetry-sqlite/src/cursor.rs (7), src/spawn.rs (6), src/edn_shim.rs (4), src/assertion.rs (4), src/sandbox.rs (1), src/hologram.rs (1), src/freeze.rs (1), crates/wat-telemetry/src/shim.rs (1), crates/wat-macros/src/codegen.rs (1)
- Existing canaries pass: `runtime::tests::arc138_runtime_error_message_carries_span`, `types::tests::arc138_type_error_message_carries_span`.
- Slice 3a sonnet broadened ~30 helper sigs in src/runtime.rs to `list_span: &Span`; sonnet has the worked example in muscle memory from slice 3a-finish (just completed).

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | Only the 15 listed files modified. No others. |
| 2 | Marker count drops to 0 | `grep -rc "arc 138 slice 3b" src/ crates/ \| awk -F: '{s+=$2} END {print s}'` returns **0**. Every marker resolved to real span OR Pattern E rationale. |
| 3 | Each remaining Span::unknown() (added during slice) has rationale | Every Pattern E leftover carries `// arc 138: no span — <reason>` on the same line or above. Especially: io.rs trait methods, codegen.rs proc-macro emit. |
| 4 | Workspace tests pass | `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. |
| 5 | Both canaries pass | `runtime::tests::arc138_runtime_error_message_carries_span` + `types::tests::arc138_type_error_message_carries_span` PASS. |
| 6 | No new variants / Display / trait changes | Emission-site changes only. No RuntimeError variant additions; no Display string changes; no WatReader/WatWriter/SchemeCtx trait expansion. |
| 7 | No commits | Working tree shows uncommitted modifications only. |
| 8 | Honest report | ~400 words; counts before/after; pattern distribution (A/B/C/D/E/F per-file); diff stat; canary results; honest deltas (especially trait-method Pattern E count + proc-macro Pattern E + helper-sig broadenings). |

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | Pattern distribution | A (~50%); B (~5%); C (~10%); D (~3%); E (~25-30%, dominated by io.rs trait methods); F (~10%, helper-sig broadening within string_ops/spawn/etc.). |
| 10 | Pattern E classification | io.rs trait methods (~16 sites) dominate Pattern E. codegen.rs (1 site). Plus minor sandbox.rs/hologram.rs leaf-failure sites. Each named explicitly with rationale. |
| 11 | Workspace runtime | `cargo test --release --workspace` runtime ≤ baseline + 5%. |
| 12 | Honest delta on Pattern F broadening | Helper sigs broadened with `list_span: &Span` per-file count named (e.g., string_ops.rs broadened `two_strings` helper). No cross-file broadening (forbidden). |

## Independent prediction

This is a smaller mechanical sweep than slice 3a-finish (156 vs 300 sites) with a known substrate observation (io.rs trait gap) already documented in BRIEF. Sonnet has slice 3a-finish in muscle memory.

- **Most likely (~65%):** 8/8 hard + 4/4 soft. Sonnet ships in 25-40 min. Pattern distribution close to predicted. io.rs trait methods all Pattern E with named rationale.
- **Pattern E ratio higher than expected (~15%):** more leaf I/O sites genuinely lack span. Hard 8/8; soft row 9 prediction off but row 10/12 PASS+. Substrate observation documented.
- **Pattern F broadening surfaces in unexpected files (~10%):** sonnet broadens helpers in string_ops, spawn, marshal that weren't anticipated. All in-file; ship clean.
- **Test regression from Display assertions (~5%):** an integration test asserts an exact RuntimeError string with new `<file>:N:M:` prefix. Sonnet investigates.
- **Sonnet over-strips or breaks something (~3%):** unlikely given canary's clear feedback + smaller scope.
- **Cross-file regression (~2%):** sonnet accidentally edits an out-of-scope file; cargo build catches; revert.

## Methodology

After sonnet reports back:

1. Read this file FIRST.
2. `git diff --stat` → only the 15 listed files modified.
3. `grep -rc "arc 138 slice 3b" src/ crates/ | awk -F: '{s+=$2} END {print s}'` → 0.
4. `grep -rc "// arc 138: no span" src/ crates/ | awk -F: '{s+=$2} END {print s}'` → measure Pattern E count; should match sonnet's report.
5. `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` → empty.
6. Run both canaries by name.
7. Spot-check 5-10 random emission sites for pattern correctness.
8. Score each row; write `SCORE-SLICE-3B.md`.
9. If clean → commit + push, queue slice 4 (MacroError + EdnReadError + ClauseGrammarError + LowerError).
10. If partial → score partial honestly + re-spawn for the gap.

## What this slice tells us

- All clean → arc 138's RuntimeError sweep is COMPLETE. Slice 4 (other error types) dispatches with confidence.
- Pattern E ratio confirms substrate trait-surface gaps (WatReader, WatWriter, possibly SchemeCtx) — earned-for-follow-up data.
- Pattern F broadening in shim files is in-file only; the cross-file constraint holds.

## What follows

- Score → commit slice 3b → write slice 4 BRIEF (MacroError + EdnReadError + ClauseGrammarError + LowerError; likely much smaller — these error types have fewer variants and emission sites).
- Spawn sonnet → score → continue to slices 5-6 → arc 138 closure.
