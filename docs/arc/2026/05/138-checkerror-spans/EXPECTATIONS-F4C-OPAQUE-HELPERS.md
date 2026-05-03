# Arc 138 F4c — Pre-handoff expectations

**Brief:** `BRIEF-F4C-OPAQUE-HELPERS.md`
**Targets:** src/rust_deps/marshal.rs + src/io.rs + crates/wat-telemetry-sqlite/src/auto.rs

## Setup — workspace state pre-spawn

- Baseline: F4b commit `fbcc1a4`. F1+F2+F3+F4a+F4b closed.
- 7 production-code Pattern E sites in marshal.rs (per F4b's honest delta): rust_opaque_arc ×2, ThreadOwnedCell::ensure_owner ×1, OwnedMoveCell::take ×2, downcast_ref_opaque ×2.
- ~6 with_mut callers in src/io.rs (inside WatReader/WatWriter trait method bodies — span is in scope from F3).
- ~4 helper callers in crates/wat-telemetry-sqlite/src/auto.rs.
- 6/6 arc138 canaries pass.

## Hard scorecard (6 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | 3 files: marshal.rs + io.rs + auto.rs. NO others. |
| 2 | 5 helpers gain span | rust_opaque_arc, ensure_owner, with_mut, OwnedMoveCell::take, downcast_ref_opaque. |
| 3 | Recursive: with_mut → ensure_owner passes span | confirmed. |
| 4 | All ~10 callers updated | io.rs (6) + auto.rs (4) pass real spans. Test callers use Span::unknown() acceptably. |
| 5 | Pattern E count drops | marshal.rs production-code 7 → 0. Test code unchanged. |
| 6 | All 6 canaries + workspace tests pass | empty FAILED. |

## Soft scorecard (3 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 7 | Calibration | ≤ 15 min sonnet (matches F4a shape). |
| 8 | No commits | working tree only. |
| 9 | Honest report | per-helper + per-caller confirmation. |

## Independent prediction

- **Most likely (~75%):** 6/6 + 3/3, sonnet 8-15 min. Same pattern as F4a.
- **with_mut closure-friendliness gotcha (~10%):** moving span before the closure may surface a borrow issue; sonnet uses span.clone() pattern.
- **Test pattern updates (~10%):** marshal.rs internal tests need Span::unknown() arguments.
- **Cross-file regression (~5%):** rare.

## Methodology

Standard verify (diff stat, grep counts, canaries, workspace) → score → commit + push → queue slice 5 (ConfigError) since all 4 cracks closed.
