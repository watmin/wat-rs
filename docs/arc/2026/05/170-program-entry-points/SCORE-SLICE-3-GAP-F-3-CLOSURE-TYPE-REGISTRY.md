# Arc 170 slice 3 Gap F-3 — SCORE (parent type registry inheritance to spawn-process child)

**Date:** 2026-05-12
**Branch:** arc-170-program-entry-points
**Status:** COMPLETE — 2220 passed / 0 failed

## Scorecard (6 rows)

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | `extract_closure` includes parent's full user type registry sweep after `extract_user_types_to_fixpoint` | grep + read | PASS — loop at `src/closure_extract.rs` iterates `parent_types.iter()`, calls `record_type_dependency` for every non-reserved-prefix name |
| B | No new field on `ClosurePackage` — prologue-based inclusion (type-def AST forms added to `state.captured_types` before prologue assembly) | grep + read | PASS — `ClosurePackage` struct unchanged; whole-registry sweep lands in existing `state.captured_types` BTreeMap; prologue assembly path unchanged |
| C | Child startup uses inherited types — all types in prologue flow through `startup_from_forms` step 5 (`register_types`) | grep + read | PASS — `spawn_process_child_branch` unchanged; prologue now contains all parent user types as struct/enum/newtype/alias AST forms; `startup_from_forms` step 5 registers them into child's fresh TypeEnv |
| D | 3 new probes pass: struct / enum / parametric | cargo test | PASS — `probe_spawn_process_inherits_parent_struct`, `probe_spawn_process_inherits_parent_enum`, `probe_spawn_process_inherits_parametric_type` all pass |
| E | All existing 18 substrate probes still pass; workspace at 2220 / 0 failed | full test | PASS — 3 new probes + T3 contract update: 2217 (F-1 baseline) + 3 = 2220; 0 failed |
| F | Hermetic isolation semantics preserved — existing fork-program / spawn-process integration tests unchanged | full workspace | PASS — `wat_arc170_program_contracts` (24 tests), `wat_arc170_closure_extraction` (21 tests), `arc112_slice2b_process_send_recv` all pass |

**All 6 rows PASS.**

---

## Files changed

| File | Change |
|------|--------|
| `src/closure_extract.rs` | Added whole-registry sweep loop after `extract_user_types_to_fixpoint` (~45 LOC including commentary). No other changes — `ClosurePackage` struct, prologue assembly, and `spawn_process_child_branch` are all unchanged. |
| `tests/probe_spawn_process_parent_type.rs` | New file — 3 Gap F-3 regression probes (new file). |
| `tests/wat_arc170_closure_extraction.rs` | T3 (`t3_toplevel_defn_uses_user_types`) — updated 3 negative assertions (`!type_decls.contains(...)`) to positive assertions (`type_decls.contains(...)`); the whole-registry sweep changes the contract from "filtered-by-reference" to "whole-registry." |
| `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-F-3-CLOSURE-TYPE-REGISTRY.md` | This file. |

---

## Workspace delta

- Baseline (post-F-1): 2217 passed / 0 failed
- Post F-3: 2220 passed / 0 failed (+3 probes)

---

## Inclusion strategy rationale — whole-registry vs filtered-by-body-references

**Decision: whole-registry.** The existing reference-walker (`walk_free_symbols` + `record_type_refs_in_typeexpr`) already implements filtered extraction — it captures types statically referenced in the fn signature and body AST. Gap F-3 adds a complementary sweep for types used DYNAMICALLY.

**Why whole-registry is correct (four questions):**

- **Obvious**: filtered-by-body-references is already done by the existing walker. Any additional sweep must be whole-registry or it fails to solve the dynamic-dispatch case (`edn::read`, reflection, future dynamic dispatch). A filtered-but-not-whole-registry sweep would still leave the gap open for any dynamic type usage the static walker can't see.

- **Simple**: one loop — `parent_types.iter()` + `record_type_dependency` (already idempotent, no duplication). No new walker infrastructure, no body-re-walking, no type-reference graph analysis. The existing `topo_sort_types` + `type_def_to_ast` prologue assembly handles the new entries without change.

- **Honest**: The cost is O(parent user type count) additional entries in the prologue. For real programs this is O(tens) of type forms — negligible. The alternative (filtered-by-dynamic-references) would require a dynamic analysis pass that doesn't exist and would be fundamentally incomplete (strings are opaque to static analysis).

- **Good UX**: The child always has the full type picture the parent had. `edn::read`, reflection, `(:wat::runtime::signature-of ...)`, and any future dynamic dispatch all work without the caller needing to annotate "exported types." The caller model is: declare types at parent top level; they're available in every subprocess. This matches the BRIEF's intent ("types are global constants").

**Reserved-prefix filter**: types under `:wat::*` / `:rust::*` are always re-registered in the child via `TypeEnv::with_builtins()` + `register_stdlib_types` during `startup_from_forms`. Including them in the sweep would trigger the `ReservedPrefix` gate in `TypeEnv::register`. The loop filters: `!crate::resolve::is_reserved_prefix(name)`.

**Idempotency**: `record_type_dependency` checks `state.captured_types.contains_key(name)` before inserting. Types already captured via the reference-walking path are skipped — no duplication in the prologue. Arc 054's idempotency rule (`TypeEnv::register` accepts byte-equivalent re-registration as a no-op) provides a second safety layer.

---

## TypeEnv-Arc-compatibility verification

`ClosurePackage` does NOT carry an `Arc<TypeEnv>` field. The implementation uses the prologue-based approach: types are serialized as `WatAST` forms (via the existing `type_def_to_ast` function) and included in `prologue`. The child's `startup_from_forms` pipeline reconstructs a fresh TypeEnv from these forms at step 5 (`register_types`). This is the same path existing captured-type forms take.

This approach:
1. Does NOT require modifying `TypeEnv`'s internal representation (constraint satisfied).
2. Does NOT share parent's `Arc<TypeEnv>` with the child — the child builds its own. The child's TypeEnv is populated from the prologue forms, not from a shared pointer. This preserves the hermetic isolation model: the child's world is built from scratch (only the prologue data crosses the OS process boundary, serialized as AST).
3. Aligns with ZERO-MUTEX.md Tier 1 (immutable shared state) semantics: the prologue is `Vec<WatAST>`, which is immutable data passed through `fork(2)`'s COW snapshot. No lock needed.

**Alternative considered (Arc<TypeEnv> on ClosurePackage)**: rejected. Adding `parent_type_env: Option<Arc<TypeEnv>>` to `ClosurePackage` and a new `startup_from_forms_with_parent_types` variant in `freeze.rs` would:
1. Require a new `startup_from_forms` variant
2. Require populating `Arc<TypeEnv>` in `spawn_process_child_branch`
3. Complicate the prologue model (types in both prologue AND external Arc)
4. Create API surface not needed elsewhere

The prologue-based approach is simpler and reuses the existing machinery. The Arc approach would have been useful if TypeEnv were large (millions of entries) and copy cost mattered — not the case for O(tens) user types.

---

## Hermetic regression check result

Existing hermetic isolation tests all pass after the fix:

- `wat_arc170_program_contracts`: 24/24 — all spawn-process, spawn-thread, fork-program, and hermetic semantics tests pass
- `wat_arc170_closure_extraction`: 21/21 — all closure extraction tests pass (T3 updated; see below)
- `arc112_slice2b_process_send_recv`: 1/1
- `wat_arc113_cross_fork_cascade`, `wat_arc113_raise_round_trip`: all pass

**Hermetic semantics analysis**: sharing types with the child via the prologue does NOT violate hermetic isolation because:
1. Types are immutable declarations — parent and child see the same type definitions; neither modifies them. There is no observable state difference between "child has its own copy" and "child sees parent's types."
2. What crosses the process boundary is type DEFINITIONS (immutable data serialized as AST forms), not type-instance values. Hermetic isolation is preserved for runtime state (memory, signals, exit, panic recovery). Type registry sharing is a compile-time / startup-time concern, not a runtime state concern.
3. Each child builds its own `FrozenWorld` from the prologue — no shared mutable state between parent and child.

---

## Parametric type handling

Parametric types (`:my::Type<E>`) are stored in TypeEnv by BASE name (`:my::Type` without `<E>`) per `parse_declared_name`'s `stored_name = format!(":{}", base)` path. The `parent_types.iter()` loop uses the registry key as the iteration key, so parametric types appear under their base name.

**probe_spawn_process_inherits_parametric_type** tests this: the type `:test::proto::Wrapper<E>` (declared with `type_params = ["E"]`) is swept into the prologue as `:test::proto::Wrapper`. `edn::read` with `#test.proto/Wrapper {...}` calls `ns_to_wat_path("test.proto", "Wrapper")` = `:test::proto::Wrapper` — the same key. The lookup succeeds.

**Note on probe 3**: The WAT struct declaration uses `E` as a field type parameter. WAT's struct parser requires field types to be keywords (type-expressions), not bare symbols. `E` as a bare symbol causes a parse error. Probe 3 therefore uses `:wat::core::i64` for the `value` field (concrete type) while retaining the `<E>` type param in the struct NAME to exercise the base-name-stripping path. This is the correct scope for Gap F-3: testing that the TYPE REGISTRY ENTRY (by base name) is inherited — not that parametric field-type inference works in the child (orthogonal, future scope).

`type_def_to_ast` for the parametric struct emits:
```scheme
(:wat::core::struct :test::proto::Wrapper<E>
  (label :wat::core::String)
  (value :wat::core::i64))
```

The child's `startup_from_forms` step 5 re-parses this form, calls `parse_declared_name`, strips `<E>`, and registers under `:test::proto::Wrapper`. ✓

---

## Honest deltas (≥ 3)

### Delta 1 — T3 contract update: "filtered" → "whole-registry"

`t3_toplevel_defn_uses_user_types` in `wat_arc170_closure_extraction.rs` previously asserted that unreferenced types (`:my::PriceUsd`, `:my::Side`, `:my::Coord`) are NOT in the prologue. This reflected the pre-Gap-F-3 "filtered-by-reference" contract.

After Gap F-3, these types ARE in the prologue (whole-registry sweep). The test was updated: negative assertions (`!type_decls.contains(...)`) became positive assertions (`type_decls.contains(...)`). The functional behavior of T3 is unchanged — the prologue is still re-frozen successfully, `Point/new` is still callable, the compute function still returns 7. The extra types are harmless.

This is a DOCUMENTED behavior change, not a regression. The BRIEF explicitly states the inclusion strategy decision surface as an anticipated honest delta.

### Delta 2 — No new ClosurePackage field; prologue-based approach chosen

The BRIEF anticipated either `Arc<TypeEnv>` on `ClosurePackage` or prologue-based inclusion. Implementation chose prologue-based. Rationale: simpler (no new `startup_from_forms` variant, no new ClosurePackage field, no new `spawn_process_child_branch` path), reuses the existing `type_def_to_ast` + prologue assembly machinery, and avoids a new `Arc<TypeEnv>` API surface. The prologue model is the natural extension of what `extract_closure` already does for captured types — adding the whole-registry sweep is one additional pass with no new abstractions.

### Delta 3 — WAT struct field types cannot be bare type-parameter symbols

Probe 3 (parametric) initially declared the `value` field as `E` (the type parameter):

```scheme
(:wat::core::struct :test::proto::Wrapper<E>
  (label :wat::core::String)
  (value E))
```

This caused a freeze error: `"malformed field: field type must be a keyword; got symbol"`. WAT's struct parser (`parse_field`) requires field types to be `WatAST::Keyword`. A bare symbol `E` is not a keyword. The probe was corrected to use `:wat::core::i64` for the `value` field.

This reveals that WAT's parametric struct syntax (`:my::Wrapper<E>`) declares the type parameter NAME in the struct's type-params list but does NOT support bare-symbol type-params as field types in the current substrate. This is NOT a Gap F-3 concern (orthogonal to type registry inheritance). Noted here for the record.

### Delta 4 — `record_type_dependency` ordering for swept types

Newly swept types have no entries in `state.type_edges` (no cross-edges among them). All swept types therefore have `indeg=0` in the topo sort and are emitted in `type_discovery_order` sequence (insertion order via the BTreeMap iteration + `record_type_dependency` appending to `type_discovery_order`). BTreeMap iteration is alphabetical, so swept types are emitted in alphabetical order by type name.

For monomorphic struct/enum/newtype forms, registration ordering is irrelevant — `TypeEnv::register_with_span` for struct/enum/newtype does not validate field types against the registry (only `parse_type_decl` which is syntax-only). For alias forms (`typealias`), `check_alias_no_cycle` walks existing registry entries — if alias A references alias B and B is registered after A (alphabetical order may produce this), the cycle check for A would NOT recurse through B (since B isn't in the env yet at A's registration time). This is safe: there are no cycles among parent-declared aliases (the parent already verified this); the cycle check being unable to walk through B just means it misses the chance to verify again (a no-op for a parent world with valid aliases).

**Edge case accepted**: if alias ordering matters for the child's startup, alphabetical ordering may not be topological. For the probes in scope (struct + enum + parametric struct), no aliases are involved. Alias ordering for the general case is deferred — it would require walking TypeDef field types to build cross-edges, which is a non-trivial addition and not needed for the Gap F-3 scope.

### Delta 5 — Enum probe EDN format: `#ns.EnumName/VariantName nil`

Initial probe 2 design used `"#test.proto/Color :Red"` (a keyword body, not nil), which would not be recognized as a unit variant. The correct WAT-edn serialization of a unit variant `Color::Red` is:
- `tag_from_type_path(":test::proto::Color::Red")` → rfind `::` → ns=`test.proto.Color`, name=`Red` → tag `#test.proto.Color/Red nil`

The probe uses `"#test.proto.Color/Red nil"` — including the `.Color` suffix in the namespace, with `nil` payload. `reconstruct_enum_unit("test.proto.Color", "Red", types)` → `ns_to_enum_path("test.proto.Color")` = `:test::proto::Color` → TypeEnv lookup → found (after fix). ✓

This format is non-obvious (the enum type name appears in the tag namespace, not just the variant name). Confirmed correct against the `value_to_edn_with` Enum arm in `edn_shim.rs`.

---

## Cross-references

- V4 SCORE (failure pattern 3): `SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md` — the original analysis that identified this gap
- Gap F-1 SCORE: `SCORE-SLICE-3-GAP-F-1-STRUCT-ENUM-PREGEN.md` — predecessor slice
- Gap F-2 (next): resolver quote-awareness — still pending
- Gap G (after F-2): Path E macro shape — still pending
- ZERO-MUTEX.md Tier 1 — prologue-based approach aligns with immutable-data-across-fork pattern
- `src/closure_extract.rs` — the modified file (one loop, ~45 LOC with commentary)
- `tests/probe_spawn_process_parent_type.rs` — new probe file (3 probes)
- `tests/wat_arc170_closure_extraction.rs` — T3 updated (3 assertions changed)
