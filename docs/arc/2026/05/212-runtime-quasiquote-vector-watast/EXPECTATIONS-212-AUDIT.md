# Arc 212 — EXPECTATIONS for slices γ + δ (audit + migration)

## Independent prediction

- **Runtime band:** 60–90 min Mode A. Audit ~50 sites; migrate ~10-15 walkers; verify.
- **Lines changed:** ~150-300 LOC across many files (each migration: ~5-10 LOC diff)
- **New files:** 1 (SCORE-212-AUDIT.md)
- **Workspace failure delta:** 0 (current baseline 1; only probe_lifeline_pipe_proof; unrelated)
- **Surprises expected:** 1-3 (an "is this really a walker?" classification edge case; a walker whose recursion has custom child-selection that doesn't map cleanly to children(); a test that relied on the previous Vector-skip behavior — unlikely)

## Predicted classification breakdown

| Class | Predicted count |
|---|---|
| Walker (must migrate) | 10-15 |
| Leaf-decomposition (leave) | 25-30 |
| Single-shape-walker (case-by-case) | 5-10 |

If the Walker count is significantly higher (>20) or lower (<5), that's calibration data worth noting.

## Honest-delta watch

1. **Some walker has fundamentally non-migrable recursion** — e.g., it picks specific child positions (`items[0]`, `items[2..]`) rather than walking all children. For these: `children()` doesn't fit; the walker stays as-is OR uses `children()` for the generic case + custom indexing for the specific positions. Sonnet surfaces in SCORE.

2. **A test depended on the previous Vector-skip behavior** — unlikely (the behavior was buggy; tests passing despite the bug were silently lucky), but possible. If a test fails post-migration, surface honestly.

3. **closure_extract.rs walkers** — preliminary grep showed multiple Vector arms; some might NOT be using children() pattern but rather explicit Vector handling for specific cases. Audit each carefully; don't over-migrate where the explicit handling has reason.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | All ~50 sites classified | YES |
| 2 | Walkers migrated to children() | YES (preserving special-case logic) |
| 3 | Leaf-decomposition sites unchanged | YES |
| 4 | Workspace failure count unchanged | YES (still 1) |
| 5 | t6 still passes (substrate-level fix preserved) | YES |
| 6 | cargo build clean | YES |
| 7 | SCORE inscribes per-site catalog + arc 211 doctrine extension | YES |

## Mode classification

- **Mode A:** audit complete; all migrations shipped; no regressions; SCORE comprehensive.
- **Mode B:** ships with honest deltas (different classification counts; some walker requires non-mechanical migration; one walker preserved as single-shape-walker with explicit reasoning).
- **Mode B-time-violation:** ran >90 min. Investigate; the work is bounded; significant overrun suggests classification edge cases warrant orchestrator decision.
- **Mode C:** STOP trigger hit during audit.

## Calibration metadata

- Orchestrator confidence: MEDIUM-HIGH. The pattern (recurse on List, miss Vector) is well-understood; the fix shape (children()) is settled; the audit is bounded by a finite list of files.
- Risk factors: edge-case walkers with custom child-selection; minor LOC sprawl across migrations.
- Why this matters: validates arc 211 doctrine at the substrate-ownership layer; eliminates a permanent class of bug.

## Tooling-proven-by-use cascade

Slice α validated arc 211b (panic-EDN diagnostic enabled t6 fix). THIS slice validates arc 211's DOCTRINE — the same pattern (substrate owns; consumers benefit; failure class structurally eliminated) that produced:
- arc 211a (ctor-install) — every binary gets the hook automatically
- arc 211e (process_stdio module dedup) — one canonical implementation
- arc 212-β (children() primitive) — one canonical recursion

That recurring shape IS the substrate's discipline. SCORE inscribes the cascade.

## Cross-references

- BRIEF-212-AUDIT.md — work definition
- DESIGN.md § "Scope EXPANDED 2026-05-18" — locked scope + sub-slice decomposition
- Arc 211 DESIGN § "Tooling-proven-by-use closure condition" — the doctrine this slice extends
- `scratch/FAILURE-ENGINEERING.md` — the discipline this slice embodies
