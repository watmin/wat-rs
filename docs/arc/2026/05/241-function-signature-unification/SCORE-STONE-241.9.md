# SCORE — Stone 241.9: `:wat::core::defenum` HARD CUT

**Mode:** A (substrate-only, no new integration surface)
**Runtime:** ~2 h elapsed (two-session; context boundary mid-flight)
**Cascade size:** 33 files (7 src/ + 14 tests/ + 3 wat/ + 9 wat-tests/)
**Lib tests:** 834 / 0 (1 pre-existing ignored)
**Clippy:** 883 warnings (≤ 902 gate)

---

## Phase A Scorecard (8 rows)

| # | Contract | Status | Notes |
|---|----------|--------|-------|
| 1 | `(:wat::core::defenum :N :V1 :V2)` unit-only | PASS | |
| 2 | `(:wat::core::defenum :N :V1 [f <- :T] :V2)` mixed | PASS | one-token look-ahead |
| 3 | Interleaved unit + tagged variants | PASS | |
| 4 | defenum with variant metadata `{:kw val}` | PASS | |
| 5 | Empty metadata `{}` rejected | PASS | |
| 6 | `:wat::core::enum` UNIT form HARD CUT rejected | PASS | check.rs MalformedForm arm |
| 7 | `:wat::core::enum` TAGGED pair form HARD CUT rejected | PASS | check.rs MalformedForm arm |
| 8 | defenum registers usable unit + tagged variants | PASS | resolver fix required |
| — | Lib tests preserved | PASS | 834 / 0 |
| — | Clippy gate | PASS | 883 ≤ 902 |
| — | Arc 241.1–241.8 probes preserved | PASS | all green |

---

## Structural Verification (7 files)

| Component | Change | Verified |
|-----------|--------|---------|
| `src/types.rs` | `parse_defenum` minted (positional + look-ahead grammar); `parse_enum` + `parse_enum_variant` + `parse_field` HARD CUT deleted | ✓ |
| `src/check.rs` | Legacy `:wat::core::enum` arm deleted; HARD CUT rejection arm added; whitelist updated to defenum | ✓ |
| `src/freeze.rs` | `is_mutation_form` + `is_declaration_form`: enum → defenum | ✓ |
| `src/runtime.rs` | `is_enum_form`, `preregister_enum_constructors_from_form`, `typedef_to_define_ast` rewritten for defenum positional grammar | ✓ |
| `src/special_forms.rs` | Registry entry: `:wat::core::enum` → `:wat::core::defenum` with updated arity hint | ✓ |
| `src/closure_extract.rs` | `walk_defenum_form` (replaces walk_enum_form); `type_def_to_ast` emits defenum | ✓ |
| `src/resolve.rs` | **Resolver gap fix**: `is_resolvable_call_head` now checks `sym.unit_variants` (see gap section) | ✓ |

---

## Migration Cascade Audit (33 files)

### Substrate (7 files)
- `src/types.rs` — S1 (parse_defenum mint + parse_enum/parse_enum_variant/parse_field HARD CUT delete) + internal test migration
- `src/check.rs` — HARD CUT rejection arm + whitelist update
- `src/freeze.rs` — mutation/declaration dispatch update + test migration
- `src/runtime.rs` — enum form detection, constructor preregistration, AST emission
- `src/special_forms.rs` — registry entry
- `src/closure_extract.rs` — walk + emit for defenum
- `src/resolve.rs` — resolver gap fix (unit variant call-head resolution)

### Test cascade (14 files)
All test files with legacy `:wat::core::enum` declarations migrated to `:wat::core::defenum`:

**Semantic restructure (zero-field tagged → unit + arm syntax update):**
- `tests/probe_counter_actor_process_diag.rs` — 8 enum declarations; `(Get)`, `(Reset)`, `(Shutdown)` zero-field tagged → `:Get`, `:Reset`, `:Shutdown` UNIT; `(Increment (n :i64))` → `:Increment [n <- :wat::core::i64]`; pattern arms `((:Ns::E::V) body)` → `(:Ns::E::V body)` for all unit arms

**Field-for-field migrations (syntax only):**
- `tests/probe_arc237_stone5fix_nominal.rs` (2 unit enums)
- `tests/probe_arc237_stone6_is_predicate.rs` (1 unit enum)
- `tests/probe_closure_body_prelude_lift.rs` (2 unit enums inside spawn-process forms)
- `tests/probe_declaration_form_lift.rs` (1 enum + `is_declaration_form` string assertion updated)
- `tests/probe_def_not_special.rs` (1 enum)
- `tests/probe_do_splice_enum.rs` (2 tagged enums; pair syntax → `[field <- :T]`)
- `tests/probe_let_splice_enum.rs` (2 tagged enums; parallel to do_splice)
- `tests/probe_register_types_splice_aware.rs` (1 enum inside do form)
- `tests/probe_spawn_process_parent_type.rs` (2 unit enums; replace_all)
- `tests/wat_arc148_ord_buildout.rs` (1 unit enum)
- `tests/wat_arc170_closure_extraction.rs` (1 unit enum + 1 tagged `(Rect (w :i64) (h :i64))` → `:Rect [w <- :i64 h <- :i64]` + helper string assertion)
- `tests/wat_not_eq.rs` (1 unit enum)
- `tests/wat_user_enums.rs` (9 occurrences: 3 unit Color + Candle/Open/Pair tagged variants)

### WAT kernel files (3 files)
- `wat/kernel/services/stdin.wat` — `Read` UNIT variant; pattern arm updated
- `wat/kernel/services/stdout.wat` — drop-in
- `wat/kernel/services/stderr.wat` — drop-in

### WAT test files (9 files)
- `wat-tests/service-template.wat` — tagged variants
- `wat-tests/edn/roundtrip.wat` — multi-field tagged variants
- `wat-tests/counter-service-thread-N1.wat` — 5 enums; Stop/Get/Reset → UNIT; pattern arms updated
- `wat-tests/counter-service-thread-N3.wat` — 5 enums; Stopped/Deprovisioned UNIT/TAGGED; pattern arms updated
- `wat-tests/counter-service-process-N3.wat` — 15+ enum declarations (parent + subprocess); Wire/WireResp/AdminReq/AdminResp/UserReq/UserResp/ServiceError all migrated
- `wat-tests/counter-actor-proof-thread.wat` — 2 enums; Request/Response dispatch updated
- `wat-tests/counter-actor-proof-process.wat` — 4 enums (parent + subprocess duplicates); dispatch patterns updated
- `wat-tests/counter-client-capability-proof.wat` — 2 enums + dispatch pattern arms
- `wat-tests/counter-service-capability-N3.wat` — 6 enums; UNIT and TAGGED variants; many pattern arms updated

---

## Honest Deltas

### Resolver gap (R-gap): unit variants invisible to resolver

**Symptom:** After migrating zero-field tagged `(Get)` → `:Get` UNIT variant, subprocess tests using `defn` bodies with unit variant match arms failed:
```
resolve: 3 unresolved reference(s):
  - :counter::Request::Get (call head — not a builtin, not a registered function)
```

**Root cause:** `is_resolvable_call_head` only checked `sym.get()` (which searches `sym.functions`). Unit variants are stored in `sym.unit_variants` only after `register_enum_methods` (step 6.5). The asymmetry exposed because `defn` forms stay in residue and ARE walked by the step-7 resolver, whereas `define` forms are consumed and NOT walked — which is why `wat_user_enums.rs` (using `define`) didn't expose this.

**Fix:** Added `sym.unit_variants.contains_key(canonical)` check to `is_resolvable_call_head` in `src/resolve.rs`. One-line fix, load-bearing for all subprocess + thread variants that use `defn` with unit variant dispatch.

### Pre-existing failures (unchanged)
- `not_eq_f64_cross_numeric_coerce` in `tests/wat_not_eq.rs` — numeric type coercion for `not=`; confirmed pre-existing, unrelated to enum migration

---

## parse_defenum Grammar (positional + one-token look-ahead)

```
(:wat::core::defenum :Ns::Name
  :UnitVariant              ; bare keyword → UNIT
  :TaggedVariant [f <- :T]  ; keyword + Vector → TAGGED (one-token look-ahead)
  ...)
```

Internal representation unchanged: `TypeDef::Enum(EnumDef)` with `unit_variants` and `tagged_variants` fields. No `EnumDef` schema extension — D5 silent generic honored.

---

## PHASE 3 CONTINUES

Stone 241.9 closes the HARD CUT on legacy enum syntax. The foundation is:

- `parse_defenum` is the sole enum declaration path
- Positional grammar: bare keyword = UNIT variant; keyword + Vector = TAGGED variant
- TAGGED variant fields use canonical `parse_argspec_triples` (Stone 241.1 substrate)
- Resolver correctly walks unit variant call-heads in `defn` bodies
- All 33 cascade files green; 834 lib tests, 8/8 Stone 241.9 contracts
