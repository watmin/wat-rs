# SCORE — Arc 207 Slice 2: mint `:wat::core::Uuid` + 6 verbs + edn_shim fix

**Status:** ALL ROWS PASS.

| Row | Result | Evidence |
|---|---|---|
| A — Verification gate passed (baseline clean, 5 file:line refs confirmed) | YES | See § A below |
| B — All 20 SCORE-SLICE-1 checklist items completed + item 7 (edn write) | YES | See § B below — all 21 items with file:line |
| C — Workspace baseline preserved at ≤4 pre-existing failures | YES | See § C below |
| D — All 10 test cases in `tests/wat_arc207_uuid_typed.rs` pass | YES | See § D below |
| E — `Uuid/from-string` strictness verified | YES | See § E below |
| F — EDN roundtrip works | YES | See § F below |
| G — `:wat::core::uuid::*` namespace verbs STILL WORK | YES | See § G below |
| H — Clippy clean on touched files | YES | See § H below |

---

## § A — Verification gate

**git status:** `git status --short` → `?? .claude/worktrees/` only. Clean.

**Baseline test failures recorded:**
```
test lifeline_pipe_zero_orphans_across_100_trials ... FAILED   (flaky)
test deftest_wat_tests_tmp_totally_bogus ... FAILED             (intentional canary)
test t6_spawn_process_factory_with_capture_round_trips ... FAILED  (Stone D2 known delta)
test startup_error_bubbles_up_as_exit_3 ... FAILED              (wat-cli pre-existing)
```
Exactly 4 pre-existing failures.

**5 file:line refs confirmed before any edits:**

| Ref | Expected | Actual |
|---|---|---|
| `runtime.rs:371` | `pub enum Value` | `pub enum Value` ✓ |
| `runtime.rs:611` | `Duration(i64)` | `Duration(i64),` ✓ |
| `runtime.rs:704` | `type_name()` fn | `pub fn type_name(&self) -> &'static str {` ✓ |
| `runtime.rs:6768` | `fn values_equal` | `fn values_equal(a: &Value, b: &Value) -> Option<bool> {` ✓ |
| `edn_shim.rs:404` | `Edn::Uuid` rejection arm | `Edn::Uuid(_) => Err(EdnReadError::Other(` ✓ |

No drift detected. Verification gate passed.

---

## § B — All 21 items completed

### `src/runtime.rs`

**Item 1 — `Value::wat__core__Uuid(uuid::Uuid)` variant added.**
`src/runtime.rs:618`: `wat__core__Uuid(uuid::Uuid),`
Doc comment cites arc 207, explains Pattern B (opaque variant per keyword/Instant/Duration precedent). Placed after `Duration(i64)` (former last variant).

**Item 2 — `type_name()` arm added.**
`src/runtime.rs:755`: `Value::wat__core__Uuid(_) => "wat::core::Uuid",`
FQDN form `"wat::core::Uuid"` (no leading `:`) consistent with all other arms.

**Item 3 — `values_equal` arm added.**
`src/runtime.rs:6809`: `(Value::wat__core__Uuid(x), Value::wat__core__Uuid(y)) => Some(x == y),`
Placed after `wat__core__keyword` arm. Arc 207 attribution comment explains: same-content Uuid equals same-content Uuid; String ≠ Uuid cross-type falls through `_ => None`.

**Item 4 — No `values_compare` arm (intentionally absent).**
UUIDs are identifiers not ordinals. Confirmed not added. Cross-type falls through to `_ => None` → TypeMismatch. Same pattern as keyword/Enum/Struct. SCORE affirms: correct.

**Item 5 — No `hashmap_key` arm (intentionally absent).**
Out of slice 2 scope per BRIEF Delta 3. Confirmed not added. SCORE affirms: correct.

**Item 18 — Dispatch wiring.**
`src/runtime.rs:4285-4289`:
```rust
":wat::core::Uuid/v4" => crate::string_ops::eval_uuid_typed_v4(args, env, sym),
":wat::core::Uuid/v5" => crate::string_ops::eval_uuid_typed_v5(args, env, sym),
":wat::core::Uuid/from-string" => crate::string_ops::eval_uuid_typed_from_string(args, env, sym),
":wat::core::Uuid/to-string" => crate::string_ops::eval_uuid_typed_to_string(args, env, sym),
":wat::core::Uuid/nil" => crate::string_ops::eval_uuid_typed_nil(args, env, sym),
```

**render_value arm added (bonus — required by exhaustiveness).**
`src/runtime.rs:14634`: `Value::wat__core__Uuid(u) => format!("#uuid \"{}\"", u),`
Renders as EDN reader literal form, consistent with the edn_shim write arm.

### `src/closure_extract.rs`

**closure_extract.rs exhaustive match arm added (required by exhaustiveness).**
`src/closure_extract.rs:1492-1499`: Uuid is portable — encoded as `(:wat::core::Uuid/from-string "canonical-form")` WatAST. Placed with other portable primitives (after keyword, before Unit).

### `src/edn_shim.rs`

**Item 6 — Fix `edn_to_value` arm (read side).**
`src/edn_shim.rs:404-406`:
```rust
// Arc 207 slice 2: `#uuid "..."` EDN reader literal → typed `:wat::core::Uuid`.
// `uuid::Uuid` is `Copy`; mirrors `Edn::Inst(t) → Value::Instant(*t)` pattern.
Edn::Uuid(u) => Ok(Value::wat__core__Uuid(*u)),
```
Replaces the former `Err(EdnReadError::Other("EDN Uuid — wat has no UUID value type yet",...))`.

**Item 7 — Add `value_to_edn_with` arm (write side).**
`src/edn_shim.rs:1609-1613`:
```rust
// Arc 207 — typed Uuid → EDN `#uuid "..."` reader literal.
// Mirrors `Value::Instant → OwnedValue::Inst` pattern.
// `uuid::Uuid` is `Copy`; `OwnedValue::Uuid` already exists in wat-edn.
Value::wat__core__Uuid(u) => OwnedValue::Uuid(*u),
```

### `src/check.rs`

**Items 8-12 — 5 type scheme registrations.**
`src/check.rs:12292-12348`:

| Item | Verb | Line | Scheme |
|---|---|---|---|
| 8 | `Uuid/v4` | 12299 | `[] -> :wat::core::Uuid` |
| 9 | `Uuid/v5` | 12311 | `[:Uuid, :String] -> :Uuid` (namespace typed) |
| 10 | `Uuid/from-string` | 12322 | `[:String] -> :Option<:Uuid>` |
| 11 | `Uuid/to-string` | 12332 | `[:Uuid] -> :String` |
| 12 | `Uuid/nil` | 12342 | `[] -> :Uuid` |

`uuid_ty = || TypeExpr::Path(":wat::core::Uuid".into())` at `check.rs:12292` — opaque Path per Pattern B.
`opt_uuid_ty = || TypeExpr::Parametric { head: "wat::core::Option", args: [uuid_ty()] }` at `check.rs:12293`.

### `src/string_ops.rs` (items 13-17)

**`is_canonical_uuid_string` helper.**
`src/string_ops.rs:336-349`: Enforces canonical 8-4-4-4-12 lowercase hyphenated form. 36 chars, hyphens at positions 8/13/18/23, all hex chars lowercase.

**Item 13 — `eval_uuid_typed_v4`.**
`src/string_ops.rs:355-370`: 0-arg; returns `Ok(Value::wat__core__Uuid(wat_edn::new_uuid_v4()))`.

**Item 14 — `eval_uuid_typed_v5`.**
`src/string_ops.rs:377-426`: 2-arg `(ns: Uuid, name: String)`; extracts `Value::wat__core__Uuid(ns_uuid)` from arg 0 (TypeMismatch if not Uuid); calls `wat_edn::new_uuid_v5(ns_uuid, &name_str)` directly — no panic foot-gun. Returns `Ok(Value::wat__core__Uuid(result))`.

**Item 15 — `eval_uuid_typed_from_string`.**
`src/string_ops.rs:427-465`: 1-arg `(s: String)`; calls `is_canonical_uuid_string` + `uuid::Uuid::parse_str`; returns `Ok(Value::Option(Arc::new(result)))` where result is `Option<Value::wat__core__Uuid>`.

**Item 16 — `eval_uuid_typed_to_string`.**
`src/string_ops.rs:468-503`: 1-arg `(u: Uuid)`; extracts `Value::wat__core__Uuid(u)` (TypeMismatch if not Uuid); returns `Ok(Value::String(Arc::new(u.to_string())))`.

**Item 17 — `eval_uuid_typed_nil`.**
`src/string_ops.rs:504-520`: 0-arg; returns `Ok(Value::wat__core__Uuid(uuid::Uuid::nil()))`.

### `src/types.rs`

**Item 19 — No `register_builtin` call (intentionally absent).**
Per Pattern B (Instant/Duration/keyword precedent): opaque leaf types with own Value variant do NOT get a TypeDef entry. Confirmed not added. SCORE affirms: correct.

### Tests

**Item 20 — New test file `tests/wat_arc207_uuid_typed.rs`.**
10 test cases covering all 8 BRIEF requirements + 2 additional EDN roundtrip cases.
All 10 pass. See § D.

---

## § C — Workspace baseline preserved

Post-slice-2 `cargo test --release --workspace --no-fail-fast` failures:

```
test deftest_wat_tests_tmp_totally_bogus - should panic ... FAILED   (intentional canary)
test t6_spawn_process_factory_with_capture_round_trips ... FAILED    (Stone D2 known delta)
test startup_error_bubbles_up_as_exit_3 ... FAILED                  (wat-cli pre-existing)
```

3 failures (lifeline_pipe_zero_orphans_across_100_trials passed this run — documented as flaky, acceptable). All 3 failures are in the pre-existing set. NO new failures introduced. Baseline preserved.

---

## § D — All test cases pass

`cargo test --release -p wat --test wat_arc207_uuid_typed` output:
```
running 10 tests
test uuid_to_string_roundtrip ... ok
test uuid_eq_uses_values_equal_arm ... ok
test uuid_edn_write_produces_reader_literal ... ok
test uuid_string_not_equal_to_typed_uuid ... ok
test uuid_nil_is_zero ... ok
test uuid_edn_roundtrip_typed ... ok
test uuid_equality_v4_differ_v5_equal ... ok
test uuid_v5_with_typed_namespace ... ok
test uuid_v4_returns_typed_uuid ... ok
test uuid_from_string_canonical_and_invalid ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

Test ↔ BRIEF mapping:
| BRIEF case | Test function |
|---|---|
| 1 — `Uuid/v4` returns typed Uuid | `uuid_v4_returns_typed_uuid` |
| 2 — `Uuid/v5` with typed namespace | `uuid_v5_with_typed_namespace` |
| 3 — `from-string` canonical/invalid | `uuid_from_string_canonical_and_invalid` |
| 4 — `to-string` roundtrip | `uuid_to_string_roundtrip` |
| 5 — `Uuid/nil` is zero | `uuid_nil_is_zero` |
| 6 — Equality (v4 differ, v5 equal) | `uuid_equality_v4_differ_v5_equal` |
| 7 — Cross-type inequality | `uuid_string_not_equal_to_typed_uuid` |
| 8 — `(= u1 u2)` via values_equal | `uuid_eq_uses_values_equal_arm` |
| EDN roundtrip (bonus) | `uuid_edn_roundtrip_typed` |
| EDN write form (bonus) | `uuid_edn_write_produces_reader_literal` |

---

## § E — `Uuid/from-string` strictness verified

Test `uuid_from_string_canonical_and_invalid` exercises 6 sub-cases:

| Input | Expected | Result |
|---|---|---|
| `"550e8400-e29b-41d4-a716-446655440000"` (canonical lowercase) | `Some` | `VALID-SOME` ✓ |
| `"550E8400-E29B-41D4-A716-446655440000"` (uppercase) | `None` | `UPPER-NONE` ✓ |
| `"urn:uuid:550e8400-..."` (URN prefix) | `None` | `URN-NONE` ✓ |
| `"{550e8400-...}"` (braced) | `None` | `BRACED-NONE` ✓ |
| `"not-a-uuid"` (garbage) | `None` | `GARBAGE-NONE` ✓ |
| `"00000000-0000-0000-0000-000000000000"` (nil UUID, canonical) | `Some` | `NIL-STR-SOME` ✓ |

Nil UUID in canonical form IS accepted (EXPECTATIONS edge case: "what about all-zero nil-uuid? — that IS canonical; should be Some(nil)"). Correct.

---

## § F — EDN roundtrip works

**Write arm:** `(:wat::edn::write uuid-val)` produces `#uuid "..."` EDN reader literal.
Verified by `uuid_edn_write_produces_reader_literal`: output length is 44 (`#uuid "` 7 + 36-char UUID + `"` 1).

**Read arm:** `(:wat::edn::read "#uuid \"...\"")` produces typed `:wat::core::Uuid`.
Verified by `uuid_edn_roundtrip_typed`: `(= back u)` returns true after write+read roundtrip.

Both test cases pass. EDN roundtrip symmetric: typed Uuid → `#uuid "..."` string → typed Uuid, with Uuid equality holding end-to-end.

---

## § G — arc 206 namespace verbs STILL WORK

`cargo test --release -p wat --test wat_arc206_uuid_substrate --test wat_arc206_uuid_v5`:

```
test result: ok. 5 passed; 0 failed   (wat_arc206_uuid_substrate)
test result: ok. 4 passed; 0 failed   (wat_arc206_uuid_v5)
```

All 9 arc 206 tests pass. `:wat::core::uuid::v4` and `:wat::core::uuid::v5` are unchanged and continue to return `:wat::core::String`. Slice 3 retires them; they are preserved through slice 2 per BRIEF hard constraint.

---

## § H — Clippy clean on touched files

`cargo clippy --release -p wat 2>&1 | grep -E "(string_ops|runtime\.rs|edn_shim|check\.rs|closure_extract)"` → no output (no warnings in touched files).

Pre-existing clippy warnings (5, in unrelated functions) are unchanged. No new warnings introduced by arc 207 slice 2 changes.

---

## Honest deltas from SCORE-SLICE-1 predictions

**Delta A — `closure_extract.rs` exhaustive match required an arm.**
SCORE-SLICE-1 checklist did not explicitly mention `closure_extract.rs`. When `Value::wat__core__Uuid` was added to the enum, Rust's exhaustiveness checker caught a match in `closure_extract.rs:encode_value_with_path`. Added a portable encoding arm: `Value::wat__core__Uuid(u) => WatAST::List([":wat::core::Uuid/from-string", u.to_string()])`. This is the correct shape — Uuid is portable (fixed 16-byte value). NOT a stop trigger; a one-line mechanical addition, confirmed consistent with existing portable-type encodings.

**Delta B — `render_value` in runtime.rs also required an arm.**
`runtime.rs:14469` `render_value` function had an exhaustive match. Added `Value::wat__core__Uuid(u) => format!("#uuid \"{}\"", u)` at `runtime.rs:14634`. Renders as EDN reader literal — consistent with what the edn_shim write arm produces.

**Delta C — Test count is 10, not 8.**
BRIEF requested 8 cases from SCORE-SLICE-1 item 20, plus orchestrator suggestion for EDN roundtrip cases. Added 2 extra: `uuid_edn_roundtrip_typed` (structural equality via write+read) and `uuid_edn_write_produces_reader_literal` (44-char form verification). Both pass. Net: 10 tests, superset of the 8 required. All 10 pass.

**Delta D — Slice 1 ref drift: none.**
All 5 file:line refs were accurate as of slice 2 spawn. No drift.
