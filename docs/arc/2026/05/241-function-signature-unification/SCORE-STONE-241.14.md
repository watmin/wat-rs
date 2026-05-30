# SCORE — Stone 241.14: `:wat::core::def-restricted` + `:wat::core::defn-restricted` HARD CUT — restrictions absorbed into binding metadata-map (Enemy 4 of 4)

**Mode:** A (substrate + cascade; vigilia NOT required — no new namespaced home)
**Runtime:** two sessions (context boundary mid-flight); resumed from compacted summary
**Cascade size:** 6 src files modified; 8 test files migrated; 2 doc files updated
**Lib tests:** 50 / 0
**Workspace test build:** clean
**Vigilia:** NOT CAST (legacy flat substrate; no new namespaced home)
**Auto-fixer:** NOT minted (cascade was compiler-guided; mechanical)

---

## Phase A Scorecard (12 rows)

| # | Contract | Status | Notes |
|---|----------|--------|-------|
| 1 | Probe C01 PASS (allowed caller passes under metadata-map restriction) | PASS | `contract_01_def_metadata_restricted_allowed_caller_passes` |
| 2 | Probe C02 PASS (non-allowed caller fails with DefRestrictedCallerNotAllowed) | PASS | `contract_02_def_metadata_restricted_non_allowed_caller_fails` |
| 3 | Probe C03 PASS (defn metadata-map :restricted-to enforces) | PASS | `contract_03_defn_metadata_restricted_enforces` |
| 4 | Probe C04 PASS (def-restricted HARD-CUT-rejected) | PASS | `contract_04_def_restricted_hard_cut_rejected` |
| 5 | Probe C05 PASS (defn-restricted HARD-CUT-rejected) | PASS | `contract_05_defn_restricted_hard_cut_rejected` |
| 6 | Probe C06 PASS (rejection remedies name def/defn respectively) | PASS | `contract_06_rejection_remedies_name_replacements` |
| 7 | Probe whole-suite 6/6 | PASS | `probe_arc241_stone14_restricted_absorbed` |
| 8 | Arc 198 test suite 5/5 (migrated fixtures) | PASS | `wat_arc198_def_restricted` — all 5 tests rewritten to metadata-map syntax |
| 9 | Arc 198 slice 2 Stone 1/2/3 (inventory wiring) migrated 6/6 | PASS | `binding_metadata` substitution; `extract_prefixes_from_binding_metadata_entry` helper |
| 10 | Arc 170 Stone B (walker collapse) 4/4 preserved | PASS | doc comments updated; enforcement contract unchanged |
| 11 | Workspace test-build clean | PASS | `cargo build --tests --workspace` exit 0 |
| 12 | Pre-INSCRIPTION grep gate clean | PASS | 0 active non-required hits for both retired forms |

---

## Structural Verification (10 rows)

| Verification | Result |
|---|---|
| `defined_value_restrictions` field DELETED from `SymbolTable` | confirmed; `binding_metadata` is sole restriction store |
| `defined_value_restrictions` field DELETED from `CheckEnv` | confirmed; `from_symbols` mirrors `binding_metadata` |
| `get_defined_value_restriction` + `register_defined_value_restriction` DELETED from `CheckEnv` | confirmed; replaced by `get_binding_metadata` |
| `extract_prefix_list_from_metadata` handles BOTH encoding paths | Vector (user metadata-map) + List-with-Vector-head (internal path) |
| `walk_for_def_restricted_call` → `walk_for_restricted_call` (renamed) | confirmed; reads from `binding_metadata` |
| `def-restricted` arm DELETED from `collect_splice_defs_ctx` + `infer_def_restricted` DELETED | confirmed; `infer_def_restricted`, `extract_def_restricted_binding`, `extract_prefix_vec` all deleted |
| `try_parse_fn_shape_def_restricted` DELETED from `runtime.rs` | confirmed |
| `def-restricted` REMOVED from `is_mutation_form` + `is_declaration_form` in `freeze.rs` | confirmed; Stone 241.14 comment added |
| `def-restricted` REMOVED from `special_forms.rs` active map | confirmed; retirement note in its place |
| 8th + 9th RETIREMENT_TABLE entries verbatim | `(":wat::core::def-restricted", ":wat::core::def")` + `(":wat::core::defn-restricted", ":wat::core::defn")` |

---

## HARD-CUT Arms (check.rs)

```rust
// Stone 241.14 — HARD CUT: :wat::core::def-restricted retired.
// Restrictions now live as {:restricted-to [...]} metadata-map on def/defn.
":wat::core::def-restricted" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.14); use ':wat::core::def' with metadata-map: ...", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}

// Stone 241.14 — HARD CUT: :wat::core::defn-restricted retired.
":wat::core::defn-restricted" => {
    return CheckResult::errs(vec![CheckError::MalformedForm { ... }]);
}
```

---

## RETIREMENT_TABLE post-stone (9 entries)

```rust
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    (":wat::core::struct",            ":wat::core::defstruct"),
    (":wat::core::struct-restricted", ":wat::core::defstruct"),
    (":wat::core::enum",              ":wat::core::defenum"),
    (":wat::core::define",            ":wat::core::defn"),
    (":wat::core::Char",              ":wat::core::char"),
    (":wat::runtime::define-alias",   ":wat::core::defalias"),
    (":wat::core::define-dispatch",   ":wat::core::defclause"),
    // Stone 241.14 — def + metadata-map replaces def-restricted; defn + metadata-map replaces defn-restricted.
    (":wat::core::def-restricted",    ":wat::core::def"),
    (":wat::core::defn-restricted",   ":wat::core::defn"),
];
```

---

## Critical Bug Found + Fixed (C02/C03 probe failure)

The initial implementation of `extract_prefix_list_from_metadata` only handled `WatAST::List` with `:wat::core::Vector` head (the internal-path encoding from `restrictions_to_binding_metadata_ast` and freeze.rs `RestrictionEntry` iteration). It returned `None` for `WatAST::Vector` — the shape produced when user-written `{:restricted-to [:prefix::]}` brace-forms are parsed by the brace-form parser.

Root cause: `[...]` inside `{...}` parses to `WatAST::Vector(items, span)`, NOT to `WatAST::List([Keyword(":wat::core::Vector"), ...], span)`. The two encoding paths are distinct:

- User metadata-map path: `WatAST::Vector([Keyword(":prefix::"), ...], _)` — direct vector
- Internal path (struct-restrictions + RestrictionEntry): `WatAST::List([Keyword(":wat::core::Vector"), Keyword(":prefix::"), ...], _)` — List with Vector head

Fix: `extract_prefix_list_from_metadata` updated to match both patterns. C02 + C03 went from FAIL to PASS immediately after the fix.

---

## Cascade Audit

### S1-S2 — Deletion of `defined_value_restrictions` storage

**`src/runtime.rs`:**
- `defined_value_restrictions: HashMap<String, Vec<String>>` field deleted from `SymbolTable`
- Debug impl field removed
- `restrictions_to_binding_metadata_ast` helper added (internal-path encoding for struct-restrictions + freeze.rs RestrictionEntry)
- def-restricted fn-shape arm deleted from `register_defines`
- struct-restrictions migrated to `binding_metadata` in `register_struct_methods`
- def-restricted arms deleted from `preregister_fn_defs_in_do` + `preregister_fn_defs_in_let`
- def-restricted arm deleted from `register_runtime_defs_form`
- def-restricted arm deleted from `DeclarationInExpressionPosition`
- `try_parse_fn_shape_def_restricted` deleted
- def-restricted removed from `SPECIAL_FORMS` array
- Two test helper guards updated (removed `| ":wat::core::def-restricted"`)

**`src/check.rs`:**
- `defined_value_restrictions: HashMap<String, Vec<String>>` field deleted from `CheckEnv`
- `from_symbols` updated to mirror `binding_metadata`
- `with_types` initializer updated
- `get_defined_value_restriction` + `register_defined_value_restriction` deleted
- `get_binding_metadata` added
- `extract_prefix_list_from_metadata` added (handles both encoding paths)
- `walk_for_def_restricted_call` renamed to `walk_for_restricted_call`; migrated to read `binding_metadata`
- `DefRestrictedCallerNotAllowed` error message updated to drop `def-restricted` reference
- def-restricted arm deleted from `collect_splice_defs_ctx`
- HARD-CUT arm added for `:wat::core::def-restricted`
- HARD-CUT arm added for `:wat::core::defn-restricted`
- `infer_def_restricted`, `extract_def_restricted_binding`, `extract_prefix_vec` deleted

**`src/freeze.rs`:**
- `use std::collections::HashMap;` added
- `RestrictionEntry` inventory loop migrated to populate `binding_metadata`
- `def-restricted` removed from `is_mutation_form` + `is_declaration_form`

### S3 — RETIREMENT_TABLE

- `src/remedy/retirement.rs`: 8th + 9th entries added; arc history table updated with Stone 241.14 rows

### S4 — `wat/core.wat`

- `defn-restricted` defmacro deleted (was lines 202-209)
- Comment block (lines 187-201) updated to historical voice: "Arc 198 defined... both forms retired by Stone 241.14"

### S5 — `src/special_forms.rs`

- `def-restricted` entry removed from active special-forms map; retirement note added

### S6 — Test cascade (8 files)

| File | Action | Rationale |
|---|---|---|
| `tests/wat_arc198_def_restricted.rs` | Fully rewritten (5/5 tests) | All tests used old `def-restricted`/`defn-restricted` syntax; migrated to `defn + {:restricted-to [...]}` metadata-map |
| `tests/wat_arc198_slice2_stone_1_inventory_wiring.rs` | Migrated (1 test) | `defined_value_restrictions` → `binding_metadata`; `extract_prefixes_from_binding_metadata_entry` helper added |
| `tests/wat_arc198_slice2_stone_2_attribute.rs` | Migrated (3 tests) | Same `defined_value_restrictions` → `binding_metadata` migration; helper added |
| `tests/wat_arc198_slice2_stone_3_apply.rs` | Migrated (2 tests) | Same migration for Thread/join-result + Process/join-result probes |
| `tests/wat_arc170_stone_b_walker_collapse.rs` | Comment updated | Module doc + inline comment: `walk_for_def_restricted_call` → `walk_for_restricted_call`; `def-restricted` diagnostic reference updated to `DefRestrictedCallerNotAllowed` variant name |

### S7 — Doc migrations

| File | Change |
|---|---|
| `docs/USER-GUIDE.md:734-795` | Section renamed + rewritten: `def-restricted`/`defn-restricted` → `{:restricted-to [...]}` metadata-map; `defined_value_restrictions` → `binding_metadata`; historical note added |
| `docs/CONVENTIONS.md:19-61` | Section renamed + rewritten: same migration; `walk_for_def_restricted_call` → `walk_for_restricted_call`; historical note added |

---

## Pre-INSCRIPTION Grep Gate

```
grep -rn "def-restricted\|defn-restricted" src/ wat/
```

**All matches categorized:**

| Category | Location | Status |
|---|---|---|
| HARD-CUT arm | `src/check.rs:5679` (def-restricted) | REQUIRED — the retirement arm itself |
| HARD-CUT arm | `src/check.rs:7176` (defn-restricted) | REQUIRED — the retirement arm itself |
| RETIREMENT_TABLE | `src/remedy/retirement.rs:62-63` | REQUIRED — table entries drive remedy |
| Comment-only | All remaining in `src/` + `wat/` | ACCEPTABLE — historical references |

**Active substrate callers: 0**

Gate CLEAN.

---

## Honest Deltas

### Context boundary mid-flight

Stone 241.14 crossed a context boundary mid-implementation (4/6 probe passes achieved before compaction). The compacted summary accurately preserved the C02/C03 root cause diagnosis. Resume was immediate — the fix to `extract_prefix_list_from_metadata` was the first action.

### Two-encoding-path problem

The `restrictions_to_binding_metadata_ast` doc comment claimed its encoding "mirrors the brace-form parser encoding" — this was wrong. The brace-form parser produces `WatAST::Vector` for `[...]`; the internal helper produces `WatAST::List` with Vector head. The doc comment was corrected as part of the fix. The two paths now coexist explicitly in `extract_prefix_list_from_metadata`.

### `DefRestrictedCallerNotAllowed` variant name preserved

Per `feedback_inscription_immutable` — historical naming stays. The error message text (Display impl) was updated to drop the `def-restricted` reference and accurately describe the metadata-map mechanism; the variant name itself is immutable.

### S11 scope widened

The grep gate found additional active references in `freeze.rs` (two sites in `is_mutation_form` + `is_declaration_form`) and `special_forms.rs` (active form entry). Both cleaned up as part of S11. These were not in the original brief scope but were honest cleanup required for HARD-CUT totality per `feedback_hard_cut_admits_no_bypasses`.

---

## What This Unblocks

**Stone 241.15** — INSCRIPTION closes arc 241. The define-family death campaign is complete:

| Enemy | Stone | Status |
|---|---|---|
| Enemy 1: `:wat::runtime::define-alias` | 241.12 | DONE |
| Enemy 2: `:wat::core::define-dispatch` | 241.13 | DONE |
| Enemy 3: `:wat::core::define` (eval residue) | 241.11 + runtime arm | DONE |
| Enemy 4: `:wat::core::def-restricted` + `:wat::core::defn-restricted` | 241.14 | DONE |

**Arc 237.8b** — reopens after Stone 241.15 per `feedback_no_regression_until_arc_done`.
