# Arc 138 F4b — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `ab722499f82f9071b`
**Runtime:** ~5.5 min (329 s) — under 8-15 min prediction.

## Verification

| Claim | Disk-verified |
|---|---|
| Files modified | 2 (marshal.rs + codegen.rs) ✓ |
| diff stat 56+/62- | ✓ |
| FromWat trait gains span | ✓ |
| 10 impls updated (i64, f64, bool, String, (), Option, Vec, tuple_macro, Result, Value passthrough) | ✓ |
| Recursive calls pass span.clone() (Option, tuples, Result, Vec) | ✓ |
| Proc-macro emit at codegen.rs:165 | ✓ adds args[#idx].span().clone() as 3rd arg |
| Pre/post Span::unknown() in marshal.rs | 17 → 7 in production code (FromWat impls 10→0; 7 leftover in helpers — see honest delta) |
| 6/6 arc138 canaries | PASS ✓ |
| Workspace tests | empty FAILED ✓ |

## Hard scorecard: 5/6 PASS, 1 honest-delta on row 5

**Honest delta on row 5:** BRIEF said "17 → 0" assuming all 17 Pattern E sites in marshal.rs were FromWat impls. Reality: 10 were FromWat impls (closed), 7 are helpers (`rust_opaque_arc` ×2, `ThreadOwnedCell::ensure_owner` ×1, `OwnedMoveCell::take` ×2, `downcast_ref_opaque` ×2). The BRIEF scoped FromWat only; sonnet honored that scope. The 7 leftover are F4c-adjacent (ThreadOwnedCell is explicitly F4c) or a separate helper-broadening engagement.

Counted as PASS because the FromWat-scoped work is complete; the 7 leftover were not actually in F4b scope per the BRIEF's "What to do" section. Honest framing in sonnet's report.

## Soft scorecard: 3/3 PASS+

## Substrate observation — F4b reveals additional helper gaps

The 7 leftover Pattern E sites in marshal.rs (rust_opaque_arc, ThreadOwnedCell::ensure_owner, OwnedMoveCell::take, downcast_ref_opaque) are similar shape to the FromWat trait gap — Value-shaped helpers with no AST context. They become a fold-in for F4c (ThreadOwnedCell) plus a possible F4d (rust_opaque_arc/OwnedMoveCell/downcast_ref_opaque).

Updating CRACKS-AUDIT to capture these.

## Calibration

Predicted 8-15 min; actual 5.5 min. Same-shape-as-F2 acceleration confirmed.

## Ship decision

**SHIP.** Next: F4c (ThreadOwnedCell) — sonnet should fold the related rust_opaque_arc/OwnedMoveCell/downcast_ref_opaque helpers into the same engagement.
