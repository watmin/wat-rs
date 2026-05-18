# Arc 212 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 10–15 min Mode A. Tiny scope; one match arm added.
- **Lines changed:** ~8 LOC added to `src/runtime.rs`
- **Workspace failure delta:** 2 → 1 (t6 passes; probe_lifeline remains for arc 213)
- **Surprises expected:** 0–1 (possibly: an existing test depended on the bug; unlikely given the fix is purely additive — Vector arm where leaves arm used to fire silently)

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | Vector arm added | YES |
| 2 | t6 passes alone | YES |
| 3 | arc170 24/24 | YES |
| 4 | Workspace 2 → 1 | YES |
| 5 | SCORE inscribes EDN validation | YES |

## Honest-delta watch

1. **The fix might be insufficient** — t6 could fail at a DIFFERENT point post-fix (e.g., the substituted Vector<WatAST> constructor argument might have its own substrate gap). Less likely given the precise diagnostic, but possible. If so: surface honestly + decide next sub-arc.

2. **Some other test might silently rely on Vector-children-not-being-walked** — additive change in principle, but a test could exist that depends on `[a b c]` literal preservation through quasiquote. Should surface as new failure if so; investigate.

3. **The fix may surface unquote-splicing-in-Vector as a follow-up** — `unquote-splicing` is documented as not yet handled. If a test relies on it inside a Vector, that's a sibling sub-arc.

## Tooling-proven-by-use validation criteria

The SCORE-212.md must inscribe:
- The EDN diagnostic that led to the fix
- The time-from-panic-to-diagnosis (~10 minutes)
- Counter-factual: without arc 211b's EDN format, diagnosis would have required substantial spelunking
- Conclusion: arc 211 panic-tooling was LOAD-BEARING for this fix

That inscription is the validation arc 211 closure needs (one of two; arc 213 is the other).

## Cross-references
- BRIEF-212.md — the work definition
- DESIGN.md — arc origin + scope + tooling-proven principle
