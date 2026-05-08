# Arc 163 Slice 3b — Unit name + type verification

**Verified 2026-05-07 by orchestrator.** Audit confirmed both
unit-related retirement surfaces are HARD-retired. Zero source
edits needed.

## Surfaces verified

| Surface | Walker fn | Variant | Live keyword usage | Status |
|---|---|---|---|---|
| `:wat::core::unit` (value position; arc 153) | `walk_type_for_legacy_unit_name` body retired arc 153 slice 2 | `BareLegacyUnitName` | 0 (37 sites all Bucket C/D — typealias-removal comment, walker reintroduction recipe, retirement-test fixtures in `wat_arc153_nil_rename.rs`) | HARD ✓ |
| `:()` (type position; arc 109 slice 1d) | (paired with same walker family) | `BareLegacyUnitType` | 0 | HARD ✓ |

## Substrate-internal evidence

From `src/types.rs:421-425`:
> Note: the retired `:wat::core::unit` typealias was removed in
> [arc 153 slice 2]. ... `:wat::core::unit` now produce a TypeMismatch
> resolving the [...] poison

So:
- Typealias `:wat::core::unit` → `:wat::core::nil` was DELETED (not just walker-fired)
- Source-level `:wat::core::unit` produces `TypeMismatch` (Pattern 2 poison) at type-check time
- No runtime alias arm exists (canonical `:wat::core::nil` parses to `Tuple(vec![])`)

## Audit method

1. Confirm walker `walk_type_for_legacy_unit_name` retirement
   comment present (line 2210 — "Arc 153 slice 2 — walker retired")
2. Confirm Display impls preserved (lines 626, 631 — emit canonical
   redirect message)
3. `grep -rEn ':wat::core::unit\b'` excluding `complected/` —
   sample residuals to classify
4. Confirm no `":wat::core::unit"` runtime alias arm

## Verdict

**2 surfaces hard-retired; no slice 3b source work required.**

Same shape as slice 3a: walker body retired (arc 113 orphaned-
scaffolding); variant + Display preserved (allows future
reintroduction of similar diagnostic patterns). All consumer code
already migrated; remaining residuals are documentation +
retirement-test fixtures.

## Next slice

Slice 3c — substrate canonicalization update (the medium-cost
prerequisite for the dependency chain): make `eval-step!` /
Bundle-input recognition / lower.rs match the canonical
`:wat::core::Vector`, not the retired `:wat::core::vec`. Once
shipped, slice 3d (kill `:wat::core::list` runtime arm + migrate
tests to canonical Vector) becomes safe.
