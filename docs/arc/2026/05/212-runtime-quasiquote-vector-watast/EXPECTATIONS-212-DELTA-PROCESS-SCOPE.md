# Arc 212 stone δ-process-scope — EXPECTATIONS

## Independent prediction

- **Runtime band:** 5-10 min Mode A. Simpler than δ-comm-positions because the sharpening is structurally close to mechanical (1 keyword added to existing scope-boundary match arm + standard children() collapse).
- **LOC changed:** ~20 (1 line in match arm + comment block update + recursion collapse)
- **New files:** 1 (SCORE)
- **Surprises expected:** 0-1 — the architecture is clean (caller already runs per-let-scope); the let scope-boundary just aligns the walker's RULE with the caller's framing

## Honest-delta watch

Two scenarios:
1. **All three tests pass.** Mode A. Expected — the let boundary mirrors fn/lambda which already works.
2. **A test fails with ProcessJoinBeforeOutputDrain false positive.** Substrate teaching about a real Process pattern that was previously slipping past. REVERT + report. Mode B acceptable.

Less likely scenario: a test PASSES that shouldn't have under the sharpened rule. That would be silent — only surfaces via review. The SCORE should describe the implementation honestly so review can catch this.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `collect_process_calls` uses `node.children()` for recursion | YES |
| 2 | `:wat::core::let` added to scope-boundary match arm alongside fn/lambda | YES |
| 3 | Existing Process classification logic preserved verbatim | YES |
| 4 | `cargo test --release --test wat_arc170_stone_a_drain_and_join` green | YES |
| 5 | `cargo test --release --test wat_arc202_process_join_holds_stdin` green | YES |
| 6 | `cargo test --release --test probe_run_hermetic_no_deadlock` green | YES |
| 7 | `cargo build --release` clean | YES |
| 8 | SCORE file written; sharpening described | YES |
| 9 | Zero other code edits anywhere | YES |

## Mode classification

- **Mode A:** all criteria satisfied
- **Mode B (acceptable):** test failure traceable to substrate-teaching; REVERTED + inscribed
- **Mode C:** STOP rule broken

## Calibration metadata

- **Orchestrator confidence:** VERY HIGH. The architecture is clean (caller per-let-scope), the implementation is bounded (1 keyword + recursion collapse), the pattern is precedented (fn/lambda already in the scope-boundary arm).
- **Risk factors:** minimal. The biggest risk would be a stdlib pattern that DOES need the walker to descend into a nested let (unlikely; arc 117 rule is per-scope).
- **Why this matters:** completes the L1 phase. Seventh L1 stone shipped → all walker-level sharpening done; ζ-newtype-wall (L2) becomes next focus.

## Cross-references

- Arc 212 DESIGN § "Single-shape-walker classification REJECTED" — this stone implements one of the two sharpenings called for
- Inscribed reframed comment at src/check.rs:~3749 — the audit-evidence
- SCORE-212-DELTA-COMM-POSITIONS.md — sibling sharpening (different mechanism: pre-walk vs scope-boundary)
- SCORE-212-DELTA-PROCESS-STDIN-JOINS.md — sibling Process walker (already migrated, same fn/lambda pattern)
- BRIEF-212-DELTA-PROCESS-SCOPE.md — the brief itself
