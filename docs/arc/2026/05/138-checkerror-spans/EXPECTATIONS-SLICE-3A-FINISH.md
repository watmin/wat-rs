# Arc 138 Slice 3a-finish — Pre-handoff expectations

**Written:** 2026-05-03, AFTER drafting brief, BEFORE spawning sonnet.

**Brief:** `BRIEF-SLICE-3A-FINISH.md`
**Target:** `src/runtime.rs` only

## Setup — workspace state pre-spawn

- Baseline (commit `9d2065a`): wat-rs workspace `cargo test --release --workspace` exit=0 excluding lab. Slice 3a partial shipped.
- Pre-sweep marker count in `src/runtime.rs`: **300** literal `// arc 138 slice 3a-finish: span TBD` (verified at commit `9d2065a`).
- Pre-sweep `Span::unknown()` count in `src/runtime.rs`: 435 total (54 synthetic-AST baseline + 300 marked stubs + ~80 unmarked sonnet stubs from slice 3a).
- 22 RuntimeError variants restructured. 22 Display arms updated via `span_prefix(span)`. 23+ helpers broadened with `list_span: &Span` (see lines 2035, 2098, 2150, 2203, 2341, 2394, 3178, 3242, 3452, 3527 et al).
- Canary `runtime::tests::arc138_runtime_error_message_carries_span` confirms `<eval>:` or `<test>:` (file:line:col) appears in rendered Display output via the real-spanned `UnboundSymbol` path.
- Slice 3b's 156 markers in 15 external files are OUT OF SCOPE for this slice — DO NOT TOUCH.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Single-file diff | Only `src/runtime.rs` modified. No other files. No tests outside the file. |
| 2 | Marker count drops | `grep -c "arc 138 slice 3a-finish" src/runtime.rs` ≤ 30 (down from 300). Drop ≥ 90%. |
| 3 | Each remaining stub has rationale | Every leftover `Span::unknown()` (excluding synthetic-AST 54 baseline and external-stub-untouched) has `// arc 138: no span — <reason>` on the same line or above. |
| 4 | Workspace tests pass | `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. |
| 5 | Canary passes | `runtime::tests::arc138_runtime_error_message_carries_span` PASSES. |
| 6 | No new variants / Display changes | Emission-site changes only. No RuntimeError variant additions; no Display string changes. |
| 7 | No commits | Working tree shows uncommitted modifications only. |
| 8 | Honest report | ~400 words; counts before/after; pattern distribution (A/B/C/D/E/F); diff stat; canary result; honest deltas; four questions. |

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | Pattern distribution | A (args[i]) ~30%; B (list_span) ~40%; C (match arm via eval(args[i])) ~20%; D (head keyword) ~5%; E (no span) ≤ 5%; F (broadened helper sig) ≤ 5%. |
| 10 | Span source quality | Spans point at the OFFENDING node (the arg / call site / head keyword), not always reaching for the enclosing list_span when args[i] is closer. |
| 11 | Workspace runtime | `cargo test --release --workspace` runtime ≤ baseline + 10%. |
| 12 | Honest delta on partial / threading | If sonnet hits context limits and ships partial: count remaining markers + line ranges. If sonnet broadens any helper sig (Pattern F): count and name them. |

## Independent prediction

This is the largest mechanical-only sweep in arc 138. 300 sites + variant + helper-sig + canary infra all already in place. Pattern is well-defined.

- **Most likely (~50%):** 8/8 hard + 4/4 soft. Sonnet ships in 40-60 min with leftover markers ≤ 30, each justified.
- **Partial completion (~30%):** sonnet ships ~200-250 sites + reports honestly with remaining queue. We score partial; re-spawn for the gap (~10-15 min next engagement).
- **Pattern F surfaces (~10%):** sonnet broadens 1-3 helper sigs to thread spans through; documents as honest delta. Hard 8/8 + soft 12 PASS+.
- **Test regression from Display assertions (~5%):** an integration test asserts an exact RuntimeError string that now has a `<test>:N:M:` prefix. Sonnet investigates rather than reverts.
- **Sonnet over-strips (~3%):** unlikely given canary's clear feedback.
- **Cross-file regression (~2%):** sonnet accidentally edits an external file; cargo build catches it; revert.

## Methodology

After sonnet reports back:

1. Read this file FIRST.
2. `git diff --stat` → only `src/runtime.rs` modified.
3. `grep -c "arc 138 slice 3a-finish" src/runtime.rs` → measure ≤ 30.
4. `grep -c "// arc 138: no span" src/runtime.rs` → matches the leftover-unknown count.
5. `grep -c "Span::unknown()" src/runtime.rs` → should drop substantially (54 baseline + ≤ 30 leftover + maybe some unmarked still ≈ 100 max).
6. `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` → empty.
7. Run canary by name.
8. Spot-check 10-15 random emission sites for pattern correctness.
9. Score each row; write `SCORE-SLICE-3A-FINISH.md`.
10. If clean → commit + push, queue slice 3b (external file sweep).
11. If partial → score partial honestly + re-spawn for the gap.

## What this slice tells us

- All clean → arc 138's pattern propagates to the LARGEST file in the substrate. Slice 3b dispatches with high confidence.
- Partial completion → the orchestrator-as-finisher pattern recurs; we re-spawn with tighter scope.
- Pattern F surfaces → real substrate observation about helper threading depth. Document for slice 4 planning.

## What follows

- Score → commit slice 3a-finish → write slice 3b BRIEF (external file sweep; 156 sites across 15 files).
- Spawn sonnet → score → continue to slices 4-6.
