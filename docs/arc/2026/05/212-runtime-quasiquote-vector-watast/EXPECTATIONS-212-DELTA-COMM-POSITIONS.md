# Arc 212 stone δ-comm-positions — EXPECTATIONS

## Independent prediction

- **Runtime band:** 20-40 min Mode A. Sharpening is substantive design (not mechanical collapse). Implementation: helper fn for consumed-name collection + let-form handler in the walker + generic-recursion-via-children() for non-let nodes.
- **LOC changed:** ~50-100 (new helper fn ~30 LOC; let-form handler ~20 LOC; existing match-arm collapse ~20 LOC delta)
- **New files:** 1 (SCORE)
- **Surprises expected:** MODERATE — sharpening introduces new walker semantics; edge cases (nested lets; shadowed names; complex match patterns) may surface during implementation

## Honest-delta watch (HIGH PRIORITY)

This stone is genuinely riskier than the mechanical migrations:

1. **The fourth permitted slot may have edge cases the BRIEF didn't enumerate.** Examples:
   - Nested lets: inner `let` rebinds a name from outer scope
   - Match scrutinee that's a compound expression (not just bare symbol)
   - Expect-value that wraps a compound expression
   - The bound name shadowed by a later binding in the same let
   
   If sonnet encounters any of these and the BRIEF's rule doesn't cleanly handle, STOP-trigger 6 fires: surface in SCORE, do not invent semantics.

2. **Other tests may break from the sharpened coverage.** A test that has a comm-call in a position that was previously slipping past (NOT in any of the four slots, NOT in body) would NEWLY fail. That's Mode B substrate-teaching. STOP-trigger 1 sub-rule fires.

3. **The implementation may discover a cleaner approach than pre-walk.** If so: ship the cleaner approach as long as it (a) implements the fourth permitted slot, (b) migrates recursion to children(), (c) passes the named test gate. Document in SCORE.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `validate_comm_positions` recognizes the fourth permitted slot | YES |
| 2 | Walker uses `node.children()` for generic recursion (outside the let-form handler) | YES |
| 3 | `cargo test --release --test arc112_slice2b_process_send_recv` green | YES (primary gate) |
| 4 | `cargo test --release --test arc112_scheme_probe` green | YES |
| 5 | `cargo test --release --test wat_arc208_process_io_result` green | YES |
| 6 | `cargo build --release` clean | YES |
| 7 | SCORE file written; helper fn approach documented | YES |
| 8 | Zero other code edits anywhere | YES |

## Mode classification

- **Mode A:** sharpening implemented; all three tests green; SCORE clean
- **Mode B (acceptable):** test failure traceable to substrate-teaching (previously-silent comm pattern now caught) OR architectural problem surfaced during implementation; REVERTED + inscribed honestly
- **Mode C:** STOP rule broken (modified a test, scope-crept, "improved" CommCtx beyond fourth-slot)

## Calibration metadata

- **Orchestrator confidence:** MEDIUM-HIGH on the design (pre-walk approach is the clean shape); MEDIUM on first-attempt completion (sharpening always has more surface area than mechanical migrations).
- **Risk factors:**
  - Edge cases in nested lets or shadowed names
  - CommCtx flow may need adjustment for the fourth slot
  - Pre-walk efficiency: walking the let twice (once for consumed-names, once for actual analysis) is O(2n) per let — acceptable for the analysis pass
- **Why this matters:** validates the discipline at a higher complexity tier. If sharpening stones can ship clean Mode A, the L1 phase is fully proven; ζ-newtype-wall (L2) becomes the next focus.

## Stone-discipline note

Sharpening stones extend semantics, not just shape. The BRIEF specifies the new rule precisely; sonnet implements; the named test is the empirical proof. Mode B is more likely here than in mechanical migrations — that's fine. Honest Mode B reports give the orchestrator the data to decide either: (a) the sharpened rule needs refinement, (b) a test fixture is buggy, or (c) the substrate has a deeper gap that needs its own stone.

## Cross-references

- Arc 212 DESIGN § "Single-shape-walker classification REJECTED" — this stone implements the sharpening that reframing called for
- Inscribed comment at src/check.rs:2137 — the audit-evidence comment for this walker
- BRIEF-212-DELTA-COMM-POSITIONS.md — the brief itself
- `tests/arc112_slice2b_process_send_recv.rs::arc112_slice2b_schemes_wire_through_typechecker` — the primary gate; contains the exact pattern the fourth slot must permit
- δ-process-scope (next sharpening stone) — sibling pattern for collect_process_calls
