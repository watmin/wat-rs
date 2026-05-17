# SCORE — Arc 203 Slice 1: substrate primitive minting

**Slice:** Slice 1 — substrate primitive minting (struct-restricted)
**BRIEF:** `BRIEF-SLICE-1.md`
**Shipped:** 2026-05-17.

## Scorecard

| Row | What | Evidence | Result |
|-----|------|----------|--------|
| A | Form parses cleanly for a worked example | `struct_restricted_form_parses_and_accessors_callable_from_whitelist` compiles + runs; constructor and restricted + public accessors all reachable from whitelisted callers | **YES** |
| B | Constructor restriction fires on illegal caller | `struct_restricted_ctor_restriction_fires_on_illegal_caller` startup fails with `DefRestrictedCallerNotAllowed`; error names `:my::Token/new`, the offending caller, and the whitelist prefix | **YES** |
| C | Per-field restriction fires on illegal accessor caller | `struct_restricted_per_field_restriction_fires_on_illegal_caller` — negative case fails on `:my::Vault/secret`; positive case (caller in field's whitelist but outside ctor whitelist) passes | **YES** |
| D | Public accessors unrestricted | `struct_restricted_public_accessors_unrestricted` — caller in `:totally::different::ns::` reads `public-field` cleanly; no restriction registered for it | **YES** |
| E | Empty sections honored | `struct_restricted_empty_sections_honored` covers: (a) empty restricted section `()` with ctor restriction; (b) empty public section `()` with all fields restricted; (c) outsider denied access to restricted field | **YES** |
| F | Workspace failure count ≤ baseline (3 pre-existing) | `cargo test --release --workspace --no-fail-fast` shows exactly 3 failures: `deftest_wat_tests_tmp_totally_bogus`, `startup_error_bubbles_up_as_exit_3`, `t6_spawn_process_factory_with_capture_round_trips` — identical to baseline | **YES** |

**6/6 PASS.**

## Honest deltas surfaced

### Delta 1 — `register_struct_methods` extended in-place (not forked)

**Assumption (EXPECTATIONS):** Might need `register_struct_methods_with_restrictions` fork.

**Actual:** Extended in-place. After the existing accessor loop, a new block checks `struct_def.restrictions.is_some()` and writes the ctor + field whitelists to `sym.defined_value_restrictions`. No signature change needed. The restriction metadata flows via `StructDef.restrictions: Option<StructRestrictions>` — the extension point is the StructDef itself, not the function signature.

**Why in-place:** `register_struct_methods` already iterates the same `struct_def` that carries the restrictions. Forking would duplicate all the Function synthesis logic with no benefit.

### Delta 2 — `TypeDef::Struct` extended via new `StructRestrictions` struct

**Assumption (EXPECTATIONS):** "If sonnet extends TypeDef, surface the variant change; if side-table, surface where the state lives."

**Actual:** Extended `StructDef` with `restrictions: Option<StructRestrictions>`. All 14 existing `StructDef { ... }` construction sites in types.rs updated to add `restrictions: None`. No new TypeDef variant. The `StructRestrictions` type carries `ctor_whitelist: Vec<String>` and `field_restrictions: HashMap<String, Vec<String>>`.

**Why in StructDef not side-table:** The restrictions are logically part of the struct declaration and consumed at the same time `register_struct_methods` processes the struct. Co-locating them in StructDef makes the coupling explicit. A side-table would require an additional HashMap in TypeEnv with matching lifetimes; StructDef embedding is simpler.

### Delta 3 — `infer_struct_restricted` NOT minted in check.rs

**BRIEF assumption:** "Mint `infer_struct_restricted` mirroring `infer_def_restricted` for shape validation."

**Actual:** The BRIEF assumed `struct-restricted` forms reach `check_form` via the residue. They don't. `classify_type_decl` returns `Some("struct-restricted")` → `register_types` strips the form into TypeEnv at step 5, before `check_program` (step 8). The form never appears in the residue passed to `check_program`.

Shape validation happens at `parse_struct_restricted` (types.rs): malformed arity, non-keyword whitelist entries, wrong section divisibility all surface as `TypeError::MalformedDecl` at startup registration time — the same point plain struct malformations surface. Test 6 verifies this.

The check.rs change made: added `:wat::core::struct-restricted` to the `infer` None-return arm (same as `:wat::core::struct`) as a safety net for expression-position occurrence. No `infer_struct_restricted` function needed.

**Suggested BRIEF correction:** "For type-declaration forms consumed by `register_types`, shape validation lives in `parse_type_decl` (types.rs), not in `check.rs`. The `infer_struct_restricted` requirement in the BRIEF was based on the incorrect assumption that the form reaches `check_form`. Future BRIEFs for type declarations should point to types.rs for shape validation."

### Delta 4 — restricted-attrs section is flat in the WAT source

**BRIEF/DESIGN:** `([<wlist>] field <- :T, ...)` inside a List.

**Actual:** Confirmed flat. The restricted section List contains flat items: `Vector Symbol(<field>) Symbol(<-) Keyword(:T)` in groups of 4. The public section List is flat: `Symbol(<field>) Symbol(<-) Keyword(:T)` in groups of 3. Parser validates divisibility by 4 / 3 respectively.

No parser changes beyond `parse_struct_restricted` were needed — the substrate's existing parser handles keyword-headed lists and nested Vectors generically.

### Delta 5 — `preregister_struct_accessors_from_form` extended (not forked)

The pre-registration stub function needed to understand both `struct` and `struct-restricted` layouts. Extended in-place via an `is_restricted` flag. The `is_struct_form` predicate now detects both head keywords. Field name extraction from the two sections uses separate parsing logic inside the same function — no code duplication of the stub insertion logic.

## Files touched

| File | Change |
|------|--------|
| `src/types.rs` | Added `StructRestrictions` struct; extended `StructDef` with `restrictions: Option<StructRestrictions>`; added `restrictions: None` to all 14 existing construction sites; extended `classify_type_decl` to recognize `struct-restricted`; extended `parse_type_decl` to dispatch to `parse_struct_restricted`; added `parse_struct_restricted` function |
| `src/runtime.rs` | Extended `is_struct_form` to also detect `struct-restricted`; extended `preregister_struct_accessors_from_form` to parse both form layouts; extended `register_struct_methods` to populate `defined_value_restrictions` when `struct_def.restrictions.is_some()` |
| `src/check.rs` | Added `:wat::core::struct-restricted` to the `infer` None-return arm for type-declaration forms |
| `tests/wat_arc203_struct_restricted.rs` | NEW — 6 tests covering the full required scorecard |
| `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-1.md` | THIS FILE |

## Workspace delta

- Pre-arc-203 baseline: 3 pre-existing failures.
- Post-slice-1: 6 new tests added, all 6 pass. 3 pre-existing failures remain. Zero regressions.
- Net: +6 passing tests, 0 new failures.

## Suggested DESIGN/INTERSTITIAL corrections

1. **BRIEF-SLICE-1.md § "Type-check side (src/check.rs)"**: The sub-bullet "Mint `infer_struct_restricted`... for shape validation" is incorrect for type-declaration forms — shape validation lives in `parse_type_decl` (types.rs). This bullet should be revised to point at `parse_struct_restricted` in types.rs. The check.rs change needed is only the None-return arm addition.

2. **BRIEF-SLICE-1.md § "Type-check side (src/check.rs)" — "Register restrictions into `CheckEnv.defined_value_restrictions`"**: Not done at check.rs directly. The mirror flows via `CheckEnv::from_symbols` (already wired in arc 198 slice 2 Stone 1) which reads `sym.defined_value_restrictions` — populated by `register_struct_methods`. The two-write pattern (CheckEnv + SymbolTable) happens automatically without explicit check.rs manipulation.

3. No DESIGN.md corrections needed — the settled form, four-questions verdict, and mechanism description are all accurate.

## Calibration

| Metric | Predicted | Actual |
|---|---|---|
| Scorecard rows | 6/6 PASS | 6/6 PASS |
| Workspace fail count | ≤ 3 + lifeline | 3 (exact baseline) |
| New test count | 6 | 6 |
| Substrate↔assumption gaps surfaced | 1-3 | 5 (register_struct_methods extension, TypeDef carrying restrictions, no infer_struct_restricted needed, flat section parsing, preregister extension) |
| BRIEF corrections suggested | 0-2 | 2 |
| STOP-triggers fired | 0-1 | 0 |

All predictions accurate. The one notable delta: `infer_struct_restricted` not needed because `struct-restricted` is a type declaration form consumed before `check_program` — same as plain `struct`. The check.rs shape validation path for type declarations is `parse_type_decl` in types.rs, not `infer_*` in check.rs.
