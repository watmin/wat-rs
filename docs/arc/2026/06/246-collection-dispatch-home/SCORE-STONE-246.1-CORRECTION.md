# CORRECTION — SCORE-STONE-246.1.md shipped two false claims

**Forward-correction (per `feedback_inscription_immutable` — the original SCORE is not edited; this entry corrects the record).** Filed 2026-06-04 after the 246.2 vigilia ward.

## What SCORE-246.1 claimed, and why it was false

`SCORE-STONE-246.1.md` certified the 246.1 lift PASSED with two claims that the disk contradicted:

1. **":11 / :41 — `runtime.rs` cleared of collection ops; no dead duplicates."** FALSE. All **23** collection `*_inner` helpers still lived in `runtime.rs:9474–10425`, byte-identical to the home's copies — 8 kept alive by a second (value-based) dispatch path, 15 fully dead.
2. **":10 — build clean (no new warnings beyond pre-existing)."** FALSE. The 15 dead duplicates emitted `dead_code` warnings (`cargo build` showed ~22 total vs ~7 baseline). The scorer (me) read `cargo build … | tail -1` ("Finished") and never grepped the body for warnings.

## Root cause — TWO blind spots, one in the gate and one in the scorer

- **The gate's pattern was incomplete.** The move-gate `grep "^fn (infer_…|eval_…)"` structurally **cannot match a `*_inner` name** — so the entire inner-helper class was never checked. SCORE-246.1 itself banked the doctrine "a name-pattern gate is gameable by *rename*"; the deeper truth is the pattern didn't even *cover* the class. The R2 deleted only `_lifted_*`-prefixed wrappers; the `*_inner` bodies were never in scope.
- **The scorer trusted the gate + skipped the warning check.** Even a deliberate examinare pass inherited the gate's hole and didn't independently grep `dead_code`. The full vigilia ward (circumspicere) caught both — which the inward lenses and the lone scorer could not.

## What fixed it

**Stone 246.2** (the razing): closed the dispatch fork (Path-B `eval_get`/`conj`/`contains`/`assoc` redirected to `crate::collection::eval::*_inner`), deleted all 23 `runtime.rs` collection `*_inner` duplicates, dropped dead_code 22→7. Verified: GATE `grep "fn (vector|hashmap|hashset|list)_…_inner" runtime.rs` → empty; suite 895/0/1.

## Doctrine sharpened (so this class is structurally caught next time)

A "move" gate must assert on **every symbol class the move touches** + the **warning count**, not just the dispatch-fn name pattern:
- `grep "fn .*_inner" <flat-file>` (the moved class) is empty post-move — not only `^fn eval_`.
- `cargo build | grep -c "dead_code|never used"` does not **rise** vs baseline (a move that leaves dead code is incomplete).
- The orchestrator greps the build **body** for warnings, never `tail -1`.

This correction is referenced by the 246.2 SCORE; SCORE-246.1's verdict line should be read as **"PASSED the lift mechanics; FAILED completeness — corrected by 246.2."**
