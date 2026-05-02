# Arc 138 Slice 1 (Finish) — Pre-handoff expectations

**Written:** 2026-05-03, AFTER drafting brief, BEFORE spawning sonnet.

**Brief:** `BRIEF-SLICE-1-FINISH.md`
**Target:** `src/check.rs` only

## Setup — workspace state pre-spawn

- Baseline (commit `4d0d299`): wat-rs workspace `cargo test --release --workspace` exit=0. Lab tests intentionally broken; out of scope.
- Pre-sweep `Span::unknown()` count in `src/check.rs`: **206** literal occurrences (verified via `grep -c "span: Span::unknown()" src/check.rs` at commit `4d0d299`).
- Variant + Display + diagnostic infrastructure shipped at top of slice 1 (commit `9c8305c`); 6 emission sites already use real spans (commit `1b4dab8` + `fd03c59`).
- Canary test `check::tests::type_mismatch_message_carries_span` confirms `<test>:` (file:line:col) appears in rendered TypeMismatch output.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Single-file diff | Only `src/check.rs` modified. No other Rust files. No tests. No docs. |
| 2 | Span::unknown() count drops | `grep -c "span: Span::unknown()" src/check.rs` ≤ 30 (down from 206). Drop ≥ 85%. |
| 3 | Each remaining Span::unknown() has a justifying comment | Every leftover line has `// arc 138: no span — <reason>` on the same line or immediately above. |
| 4 | Workspace tests pass | `cargo test --release --workspace` exit=0. No FAILED lines. |
| 5 | Canary passes | `check::tests::type_mismatch_message_carries_span` passes — confirms TypeMismatch Display includes `<test>:` coordinates. |
| 6 | No new variants / Display / diagnostic changes | Emission-site changes only. No CheckError variant additions; no Display string changes; no diagnostic field additions. |
| 7 | No commits | Working tree shows uncommitted modifications only. |
| 8 | Honest report | ~400 words; counts before/after; pattern distribution (A/B/C/D/E); diff stat; canary result; honest deltas; four questions. |

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | Pattern distribution | A (arg-iter) sites should be the largest class (~40%); B (head-kw) ~25%; C (form span) ~20%; D (poison) ~10%; E (no span) ≤5%. |
| 10 | Span source quality | Spans point at the OFFENDING node (the arg / call site / binding), not the enclosing form. Sonnet picks the user-clickable level. |
| 11 | Workspace runtime | Total `cargo test --release --workspace` runtime ≤ baseline + 10%. Span threading shouldn't slow the type checker meaningfully. |
| 12 | Honest delta on threading | If sonnet had to add a parameter to any helper fn (to thread a span through), it's named in the report — that's a substrate observation, not a refactor we missed. |

## Independent prediction

This is the second sonnet sweep on arc-138-class work (the first was the implicit per-variant span retrofit happening in-orchestrator-thread). The pattern is well-defined; the worked sites are clear; the canary catches regressions.

- **Most likely (~70%):** 8/8 hard + 4/4 soft. Sonnet ships in 30-50 min with leftover unknowns ≤15 each justified.
- **Pattern E surfaces a substrate observation (~15%):** several sites genuinely need a parameter threaded through a helper fn signature (e.g., `infer_let_binding` taking a span). Sonnet either threads it (✓) or flags it as honest delta and we revisit (also ✓).
- **Test regression from Display-string assertions (~10%):** an integration test asserts an exact CheckError string that now has a `<test>:N:M:` prefix. Sonnet investigates rather than reverts; we sweep the test in a follow-up.
- **Sonnet over-strips and breaks something (~3%):** unlikely given the canary's clear feedback and the local-only edits.
- **Sonnet truncates a batch (~2%):** halts mid-sweep and reports unfinished. Minor risk; we resume manually or re-spawn.

## Methodology

After sonnet reports back:

1. Read this file FIRST.
2. `git diff --stat src/check.rs` → file modified, ~200-220 line edits.
3. `grep -c "span: Span::unknown()" src/check.rs` → measure ≤30.
4. `grep -c "// arc 138: no span" src/check.rs` → matches the leftover-unknown count.
5. `cargo test --release --workspace` → exit=0, no FAILED.
6. Spot-check 5-10 random emission sites for pattern correctness (does the span point at the offending node, not the enclosing form?).
7. Score each row; write `SCORE-SLICE-1-FINISH.md`.
8. If clean → commit + push, then queue slice 2 (TypeError variants).

## What this slice tells us

- All clean → arc 138's pattern is reproducible by sonnet across substrate sweeps. Future slices (2-6) dispatch with high confidence.
- Pattern E surfaces — substrate has structural span gaps. We adjust DESIGN before slice 2.
- Hard fail — the discipline isn't propagating. Re-examine BRIEF; possibly orchestrator's worked sites were insufficient.

## What follows

- Score → commit slice 1 finish → write slice 2 BRIEF (TypeError, ~7 variants in `src/types.rs`) → spawn sonnet → continue.
- If a substrate observation surfaces (Pattern E), update DESIGN before slice 2 BRIEF.
