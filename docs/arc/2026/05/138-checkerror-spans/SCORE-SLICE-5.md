# Arc 138 Slice 5 — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `a7f958b195d2508e7`
**Runtime:** ~5.75 min (345 s) — under 8-15 min prediction.

## Verification

| Claim | Disk-verified |
|---|---|
| Files modified | 1 (src/config.rs) ✓ |
| diff stat 113+/43- | ✓ |
| All 8 variants restructured (form_index dropped from 2; span on all 8) | ✓ |
| `form_index` references in config.rs | **0** ✓ (fully gone) |
| Span::unknown() in config.rs | **0** ✓ (Pattern E count: 0) |
| All 8 Display arms updated via span_prefix | ✓ |
| 2 intentional content changes (drop "form index N" suffix) | ✓ named in report |
| New canary `config::tests::arc138_config_error_message_carries_span` | PASS ✓ |
| All 7 arc138 canaries (6 existing + 1 new) | 7/7 PASS ✓ |
| Workspace tests | empty FAILED ✓ |
| 30/30 config module tests pass | ✓ |

## Hard scorecard: 7/7 PASS. Soft: 3/3 PASS+.

## Substrate observation — RequiredFieldMissing has zero emission sites

Sonnet noted: arc 037 made all fields optional, so `RequiredFieldMissing` is no longer constructed anywhere. Variant retained for API completeness but emits zero. This is honest substrate observation; the variant could be removed in a future cleanup.

## Substrate observation — clean Pattern A/B split

23 emission sites in src/config.rs — Pattern A (~10) for arg-specific errors using `args[0].span()`; Pattern B (~13) for whole-form errors using `form_span = form.span()` captured at top of loop iteration. Zero Pattern E (no rationale comments needed). The substrate has full AST context throughout config parsing.

## Calibration

Predicted 8-15 min; actual 5.75 min. Sub-prediction calibration continues from F1/F2/F4b: small single-file slices with proven patterns ship in single-digit minutes.

## Ship decision

**SHIP.** Last variant-restructure slice complete.

## Arc 138 status post-slice-5

All variant restructures shipped:
- CheckError (slice 1 + finish)
- TypeError (slice 2)
- RuntimeError (slices 3a + 3a-finish + 3b)
- MacroError + ClauseGrammarError (slice 4a)
- EdnReadError + LowerError (slice 4b)
- ConfigError (slice 5)

All 4 cracks closed: F1 MacroDef, F2 SchemeCtx, F3 WatReader/WatWriter, F4a Value-shaped helpers, F4b FromWat, F4c opaque-cell helpers.

**Next:** F-NAMES-1 (wat::test! macro emit per NAMES-AUDIT) — replace `<test>`/`<unnamed>` with real Rust file paths + test names. Then F-NAMES-2/3/4 audits, then slice 6 closure.
