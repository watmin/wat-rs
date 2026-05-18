# Arc 212 stone δ-comm-purge — EXPECTATIONS

## Independent prediction

- **Runtime band:** 5-10 min Mode A. Mechanical wrap-each-comm-in-Result/expect; 4 sites in 2 files.
- **LOC changed:** ~16 (4 sites × ~4 lines per Result/expect wrap)
- **New files:** 1 (SCORE)
- **Surprises expected:** 0-1 (the recv's :T annotation requires brief context-reading to determine the right Option<I> shape; if context is ambiguous, sonnet asks via SCORE rather than guessing)

## Honest-delta watch

This stone closes the substrate-as-teacher cascade δ-comm-positions opened. Two scenarios:

1. **Both tests pass post-migration.** The cascade is fully closed. Workspace baseline drops by 2 (the two "pre-existing" failures we now know weren't pre-existing — they were always wrong; δ-comm-positions made them visible).

2. **A test fails despite the migration.** Most likely cause: wrong :T annotation on the Result/expect form. Mode B: revert + report. If the recv's element type isn't obvious from context, the SCORE should name the ambiguity for orchestrator decision.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | tests/wat_arc170_stone_a_drain_and_join.rs:101-103 — 3 sends wrapped in Result/expect | YES |
| 2 | tests/probe_lifeline_orphan_clean_via_fork_program.rs:209 — 1 recv wrapped in Result/expect | YES |
| 3 | `cargo test --release --test wat_arc170_stone_a_drain_and_join` green (all sub-tests) | YES |
| 4 | `cargo test --release --test probe_lifeline_orphan_clean_via_fork_program` green | YES |
| 5 | `cargo build --release` clean | YES |
| 6 | Zero substrate edits (src/*.rs untouched) | YES |
| 7 | Zero other test edits | YES |
| 8 | SCORE inscribes per-site :T choice + message string | YES |

## Mode classification

- **Mode A:** all criteria satisfied
- **Mode B (acceptable):** test still fails post-wrap; honest report of which :T annotation didn't match
- **Mode C:** STOP rule broken (edited substrate, "fixed" other sites, scope-crept)

## Calibration metadata

- **Orchestrator confidence:** VERY HIGH. The diagnostic is precise (arc 110's diagnostic names exactly what's wrong), the migration is mechanical (Result/expect wrap), the test gates are direct (the failing tests become passing tests).
- **Risk factors:** the recv's :T annotation; possible the lifeline probe has a specific shape that needs careful handling.
- **Why this matters:** closes the cascade δ-comm-positions opened; restores workspace baseline; proves the protocol violation discovery was real + bounded + closable.

## Stone significance

This stone IS the cascade closure. It's not in the L0→L4 trajectory itself — it's the substrate-as-teacher cascade response to δ-comm-positions' sharpening. Closing it:

1. Restores workspace test baseline to clean
2. Unblocks δ-process-scope (whose gate failure was this same protocol violation)
3. Validates the per-stone trust gate discipline: the broken test was DETECTED, the root cause TRACED to the substrate-honest rule, the fix BOUNDED, the cascade CLOSED
4. Demonstrates `feedback_any_defect_catastrophic` operationally: >0 defects = 0 trust → immediate pivot to fix → trust restored

## Cross-references

- Arc 212 DESIGN — the L0→L4 endgame; this stone is the cascade closure
- SCORE-212-DELTA-COMM-POSITIONS.md — the sharpening that surfaced these violations
- INTERSTITIAL § 2026-05-18 (post-radiance) — the realization conversation that named the protocol violation
- BRIEF-212-DELTA-COMM-PURGE.md — the brief itself
- `docs/ZERO-MUTEX.md:295-297` — the mini-TCP doctrine the violations violate
- `feedback_any_defect_catastrophic` — the discipline that drives immediate pivot
- `feedback_attack_foundation_cracks` — the discipline that says "fix forward, use the crack as compass"
