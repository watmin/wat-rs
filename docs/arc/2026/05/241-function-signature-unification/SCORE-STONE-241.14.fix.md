# SCORE — Stone 241.14.fix: `src/restriction_entry.rs` doc-comment rewrite

**Mode:** A (doc-only; no code changes; vigilia NOT required)
**Runtime:** ~10 min
**Files modified:** 2 (`src/restriction_entry.rs`, `src/types.rs` — T2 scope expansion)
**Lib tests:** 890 / 0 (preserved; doc-only stone cannot change test count)
**cargo doc:** clean — no new warnings or broken intra-doc links

---

## Scorecard (6 rows)

| # | Contract | Status | Notes |
|---|----------|--------|-------|
| 1 | Stale `def-restricted` live-form framing removed from restriction_entry.rs module doc | PASS | New doc describes `(def :name {:restricted-to [...]} expr)` as the user-facing form |
| 2 | Stale `defined_value_restrictions` + `validate_def_restricted_caller_namespace` references removed from restriction_entry.rs wiring section | PASS | `binding_metadata` + `walk_for_restricted_call` + `restrictions_to_binding_metadata_ast` now named accurately |
| 3 | Struct + field docs rewritten — "Semantics match arc 198 slice 1's wat-side `def-restricted` form" current-tense removed | PASS | Replaced with historical-voice + accurate binding_metadata path description |
| 4 | Lib 890/0 preserved | PASS | Doc-only; no test impact |
| 5 | cargo doc clean — no new warnings or broken intra-doc links | PASS | Pre-existing warnings in interrogate-example / wat-telemetry / wat-telemetry-sqlite unchanged; no new intra-doc link breakages |
| 6 | SCORE doc authored | PASS | This file |

---

## Structural Verification

| Check | Result |
|---|---|
| `grep -n "wat-side .*def-restricted form" src/restriction_entry.rs` → 0 active hits | PASS — only match is `src/restriction_entry.rs:13` in historical-context sentence (Bucket C, KEEP) |
| `grep -n "defined_value_restrictions\|validate_def_restricted_caller_namespace" src/restriction_entry.rs` → 0 current-tense hits | PASS — line 13 is historical record ("Arc 198 originally minted ... with parallel storage `SymbolTable.defined_value_restrictions`") |
| `grep -n "binding_metadata" src/restriction_entry.rs` → ≥ 1 match | PASS — 5 matches (lines 14, 42, 43, 44, 45, 69, 81) |
| `grep -n "walk_for_restricted_call" src/restriction_entry.rs` → ≥ 1 match | PASS — lines 27, 44, 73 |
| `grep -n ":restricted-to" src/restriction_entry.rs` → ≥ 1 match | PASS — lines 4, 42 |
| T2: `defined_value_restrictions` in `src/types.rs` lines 108 + 127 updated | PASS — both doc hits replaced with `binding_metadata` + Stone 241.14 historical note |

---

## Diff Summary

### `src/restriction_entry.rs`

**Module-level doc (lines 1-49):**
- Removed: description of `(:wat::core::def-restricted :name [prefixes] expr)` as live form
- Added: description of `(def :name {:restricted-to [...]} expr)` metadata-map mechanism
- Added: historical note — "Arc 198 originally minted `:wat::core::def-restricted` + `:wat::core::defn-restricted` ... Stone 241.14 retired those forms (HARD CUT) and unified restriction storage into `SymbolTable.binding_metadata`"
- Updated wiring description: `inventory::iter::<RestrictionEntry>` → `restrictions_to_binding_metadata_ast` → `SymbolTable.binding_metadata[wat_name][":restricted-to"]`; `CheckEnv::from_symbols` mirrors; `walk_for_restricted_call` reads
- Stones 2/3/4 references: rewritten from future-tense "Subsequent stones plug in" to "All SHIPPED" historical voice

**Struct + field docs (lines 51-end):**
- Struct doc: added note about Stone 241.14 populate-target migration
- `wat_name` field doc: added "by `walk_for_restricted_call`"
- `prefixes` field doc: replaced "Semantics match arc 198 slice 1's wat-side `def-restricted` form" with: "Semantics: prefix-keywords whitelist. Originally minted at arc 198 slice 1 (wat-side `def-restricted` form); migrated to Stone 241.14's `binding_metadata` path. The Rust-side declaration surface (`#[restricted_to(...)]` proc-macro + this inventory channel) is unchanged."

Net change: ~20 lines rewritten, ~8 lines added (historical note + wiring detail), 4 lines removed (stale Stone 2/3/4 future-tense bullets replaced with shipped-historical bullets).

### `src/types.rs` (T2 scope expansion)

**`StructRestrictions` doc (line 108):**
- Removed: "in `defined_value_restrictions`"
- Added: "in `SymbolTable.binding_metadata` — no restriction means any caller allowed"

**`StructDef.restrictions` field doc (line 127):**
- Removed: "into `SymbolTable.defined_value_restrictions` alongside the synthesized Function entries"
- Added: "into `SymbolTable.binding_metadata` (under `:restricted-to`) alongside the synthesized Function entries (Stone 241.14 — migrated from the deleted `defined_value_restrictions` field)"

---

## Honest Deltas

### T2 scope expansion: `src/types.rs`

The T2 audit grep surfaced two stale doc-comment hits in `src/types.rs` (lines 108 + 127) describing the deleted `defined_value_restrictions` field in current-tense documentation of `StructRestrictions` and `StructDef.restrictions`. Per BRIEF S3 and `feedback_trap_door_build_the_dependency`, scope expanded to cover them in the same stone. Both updated.

The other T2 hits (`src/freeze.rs:821,839`, `src/runtime.rs:1714,2980`) are Bucket C (historical migration tombstone comments recording what changed) — correctly left unchanged.

### cargo doc warnings are pre-existing

The warnings emitted by `cargo doc --release --no-deps` (unused variables, filename collision) are pre-existing substrate-wide warnings in `interrogate-example`, `wat-telemetry`, and `wat-telemetry-sqlite` crates. No new warnings were introduced by this stone. The `wat` lib doc itself is clean.

---

## What This Unblocks

**Stone 241.17 — INSCRIPTION closes arc 241 (orchestrator-direct paperwork).** The documentation zombie is cleared. `src/restriction_entry.rs` now accurately describes the post-Stone-241.14 substrate: `binding_metadata` storage, `walk_for_restricted_call` walker, `{:restricted-to [...]}` metadata-map user surface. Arc 241 closes genuinely clean.
