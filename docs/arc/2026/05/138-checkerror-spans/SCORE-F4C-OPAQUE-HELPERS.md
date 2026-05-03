# Arc 138 F4c — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `ad2fe5c0ecfa1ed43`
**Runtime:** ~10 min (614 s) — within 8-15 min prediction.

## Verification

| Claim | Disk-verified |
|---|---|
| 5 helpers updated | rust_opaque_arc, ensure_owner, with_mut, OwnedMoveCell::take, downcast_ref_opaque ✓ |
| Recursive: with_mut → ensure_owner | ✓ |
| Files modified | 7 (sonnet honestly expanded from BRIEF's 3) — marshal.rs, io.rs, auto.rs, runtime.rs, codegen.rs, cursor.rs, tests/wat_dispatch_e4_shared.rs ✓ |
| diff stat 67+/70- | ✓ |
| Pre/post Span::unknown() in marshal.rs production | 7 → 1 (the 7 F4b-identified sites all resolved; the 1 leftover is in with_ref body, NOT in F4c's 5-helper scope) ✓ |
| 6/6 arc138 canaries | PASS ✓ |
| Workspace tests | empty FAILED ✓ |

## Hard scorecard

| # | Criterion | Result |
|---|---|---|
| 1 | File scope | **PASS+** — sonnet correctly expanded from BRIEF's 3 to 7 because compiler required it. BRIEF undercounted; sonnet's honest expansion is correct discipline. |
| 2 | 5 helpers gain span | **PASS** |
| 3 | Recursive call from with_mut | **PASS** |
| 4 | All callers updated | **PASS** — io.rs (10 sites), auto.rs (4), runtime.rs (12), codegen.rs (4 generated-code templates), cursor.rs (4), tests/wat_dispatch_e4_shared.rs (1) |
| 5 | Pattern E count drops | **PASS** — 7 → 0 in F4c-scoped helpers; 1 leftover in with_ref is honestly out of scope |
| 6 | Workspace tests + 6 canaries | **PASS** |

**HARD: 6/6 PASS.**

## Soft: 3/3 PASS+

## Substrate observation — BRIEF call-site undercount

The BRIEF named 3 files (marshal.rs + io.rs + auto.rs). Sonnet found 4 additional files with `with_mut` / opaque-helper call sites the compiler required (runtime.rs, codegen.rs, cursor.rs, test file). All 4 were necessary — same-file scope expansion is correct discipline; cross-file scope expansion required by compiler is honest necessity, not violation.

**Lesson for future BRIEFs:** when expanding a helper signature, grep the workspace exhaustively for callers BEFORE drafting the BRIEF, not after sonnet starts. The user's "discipline must not falter" applies to BRIEF scope accuracy too.

## Substrate observation — ALL FOUR CRACKS CLOSED

Per CRACKS-AUDIT:
- F1 MacroDef ✓ (commit c1cdcee)
- F2 SchemeCtx ✓ (commit 6c08b26)
- F3 WatReader/WatWriter ✓ (commit 6327840)
- F4a Value-shaped helpers ✓ (commit ec4b465)
- F4b FromWat trait ✓ (commit fbcc1a4)
- F4c opaque-cell helpers ✓ (this commit)

Arc 138's known foundation cracks are CLOSED. The substrate emits real coordinates everywhere a span is available; remaining Pattern E sites are genuine architecture (parse_program raw strings, with_ref helper, OS errors, test code).

## Calibration

Predicted 8-15 min; actual 10 min. In-band.

## Ship decision

**SHIP.** All four cracks closed. Next: slice 5 (ConfigError form_index → Span) — the original arc 138 slice 5 deferred during cracks-fix campaign. Then F-NAMES-1 (wat::test! macro emit per NAMES-AUDIT). Then slice 6 (closure).
