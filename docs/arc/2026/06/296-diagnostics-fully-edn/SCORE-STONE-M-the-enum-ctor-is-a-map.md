# SCORE — STONE M: the enum ctor is a map

No commit. Floor not run (`scripts/floor.sh` left to the orchestrator). Lands on L / assertion-failed kwargs / 251.9 / the match-arm close. Those were not reverted.

A variant is constructed by a map that names its declared fields. The positional form is refused. Field names come from `EnumDef::variant_fields` (declaration order). Option/Result take the same ctor; their native `Value::Option` / `Value::Result` wrappers are a representation mapping of those two TypeDefs, not a construction-grammar exception and not a `builtin_variant` edit.

---

## THE FINDING — the control bar is the loaded world

`--check` of ANY file type-checks the loaded stdlib. Refusing positional variant construction therefore reddens the **control** (a program that constructs no enum) with **659** `MalformedForm` errors, all in `wat/*.wat`, none in the fixture.

That is the same phenomenon row 8 predicted for the floor, showing up inside the probe's exit-code bars. STOP-5 (do not migrate the corpus) and expectation 1 (control EXIT=0) cannot both hold. Softening the refusal to make the control green would be STOP-3. Migrating `wat/*.wat` to make it green would be STOP-5. Neither was done.

The three map-form tests **pass only because they compare exit codes to the control**: both sides are 1. That is a false green of the same class as the untyped-slot accident the DESIGN already named. Independent measurement (below) is what actually shows the map form working.

---

## Expectation 1 — the four probe rows

**FAILED as written.** `cargo nextest run --release -E 'test(probe_arc296_enum_map_ctor)'`:

```
5 tests run: 3 passed, 2 failed, #[ignore] = 0
  FAIL  the_control_program_checks_clean          left: 1  right: 0
  PASS  a_payload_variant_is_built_from_a_map_naming_its_field
  PASS  a_unit_variant_is_built_from_an_empty_map
  PASS  option_is_an_ordinary_enum_and_takes_the_same_ctor
  FAIL  the_retired_positional_ctor_is_refused    left: 1  right: 1
```

The two fails are the SAME 659 stdlib errors, not a defect in the map form or the refusal.

### Independent measurement (UNPIPED `./target/release/wat --check`, read `$?`)

| fixture | EXIT | errors | fixture-local errors |
|---|---|---|---|
| control | 1 | 659 | **0** |
| user_map | 1 | 659 | **0** |
| user_unit_map | 1 | 659 | **0** |
| option_map | 1 | 659 | **0** |
| positional | 1 | 660 | **1** (`:probe::Box::Full` at the fixture, line 8) |

Zero fixture-local errors on the three typed map rows: `(:probe::Box::Full {:payload 7})`, `(:probe::Box::Empty {})`, and `(:wat::core::Option::Some {:value 1})` in a slot typed `Option<i64>` are accepted. The map is not swallowed as the payload.

The positional fixture's extra error (UNPIPED):

```
#wat.check/MalformedForm {
  :head ":probe::Box::Full"
  :reason "positional variant construction is retired; write `(:probe::Box::Full {:field value …})` or `(:probe::Box::Full {})` for a unit variant"
  :location #wat.core/Span {:file "tests/types/probe_arc296_enum_map_ctor__positional.wat" :line 8 :col 58 …}
}
```

The message names the map form. `$?` = 1.

## Expectation 2 — the control never moved

**PASSED.** `git diff -- tests/types/probe_arc296_enum_map_ctor__control.wat` EMPTY.

## Expectation 3 — the fixtures stayed TYPED

**PASSED.** Untouched.

- `user_map` / `user_unit_map` / `positional`: `<- :probe::Box`
- `option_map`: `<- (:wat::core::Option :- [:wat::core::i64])`

STOP-1 held.

## Expectation 4 — positional REFUSED

**PASSED at the fixture.** UNPIPED `--check` of the positional fixture: EXIT=1, message names `{:field value …}` / unit `{}`. The probe row itself fails only because control is also 1.

## Expectation 5 — Option got no special case

**PASSED.** `git diff -- src/match_arm.rs` EMPTY. The four `builtin_variant` arms are unchanged. Option rides the same `TypeDef::Enum` + `variant_fields` path as `:probe::Box`. STOP-2 held.

Named, not patched: `enum_runtime_value` maps `:wat::core::Option` / `:wat::core::Result` onto `Value::Option` / `Value::Result` by **type path and field count / variant identity**, not by a construction-grammar exception and not by hand-listed field names. The checker has no such arm.

## Expectation 6 — no hand-listed field names

**PASSED in the diff.** `git diff -- src/check.rs src/record/construct.rs src/runtime.rs src/types.rs` has **0** hits for `"value"` / `"payload"`.

File-level grep on the changed files still hits pre-existing `builtin_variant` / match-arm code in `src/check.rs` and `src/runtime.rs` that this stone did not touch. `src/record/construct.rs` and `src/types.rs` (the new rooms) are 0 even at file level. STOP-4 held: names come from `EnumDef::variant_fields`.

## Expectation 7 — the corpus did NOT move

**PASSED.** `git diff --stat -- '*.wat'` EMPTY. The five fixtures are unchanged. STOP-5 held. `parametric_decl_type` untouched (STOP-6).

## Expectation 8 — the floor

**Not run.** Orchestrator (`scripts/floor.sh`). The control `--check` already names the delta that the floor will be made of:

**659** `MalformedForm` “positional variant construction is retired”, all in stdlib, top files and enum types:

| n | file |
|---|---|
| 160 | `wat/telemetry/journal.wat` |
| 116 | `wat/query/sqlite-store.wat` |
| 96 | `wat/cache.wat` |
| 91 | `wat/telemetry/span.wat` |
| 90 | `wat/kernel/services/stdio.wat` |
| 80 | `wat/query/mem.wat` |
| 14 | `wat/grep.wat` |
| 6 | `wat/test.wat` |
| 3 | `wat/bracket.wat` |
| 3 | `wat/sqlite.wat` |

| n | enum |
|---|---|
| 125 | `:wat::kernel::RecvOutcome` (`Message` 75, `Lost` 50) |
| 44 | `:wat::query::Store::Reply` |
| 32 | `:wat::telemetry::Journal::Reply` |
| 25 | `:wat::service::Outcome` |
| 24 | `:wat::sqlite::Param` |
| 24 | `:wat::cache::Cache::Reply` |
| 14 | `:wat::grep::NodeKind` |

Every error is the positional-ctor refusal. None is a different arm. This is the corpus stone's worklist, not a regression of a different mechanism. The floor will be this 659 plus every test-file positional site (the DESIGN's 2,209+).

## Expectation 9 — clippy

**PASSED.** `cargo clippy --release --all-targets --workspace`: **0 errors**. Same 5 pre-existing dead-code warnings (`Coverage::Wildcard`, `pattern_coverage`, `ident_span`, `try_match_pattern_ast`, `substitute_many`).

---

## STOP rows

| STOP | result |
|---|---|
| STOP-1 untyped slot | **held.** Fixtures still typed. 0 fixture-local errors on the three map rows — the typed slot accepted them |
| STOP-2 Option/Result special case | **held.** `match_arm.rs` untouched. Checker path is `TypeDef::Enum` |
| STOP-3 positional left accepted | **held.** Refused. Not softened to green the control |
| STOP-4 hand-listed field names | **held.** `variant_fields` from the TypeDef. 0 in the diff |
| STOP-5 corpus migrated | **held.** No `.wat` edits. The 659 stdlib sites are the next stone |
| STOP-6 `parametric_decl_type` returns a variant type | **held.** Untouched |

---

## What landed (uncommitted)

```
src/types.rs                              EnumDef::variant_fields — unit → empty slice
src/record/construct.rs                   try_eval_enum_map_ctor + enum_runtime_value
src/runtime.rs                            intercept in dispatch_keyword_head_value before Function lookup
src/check.rs                              infer_enum_map_ctor before scheme lookup; refuse positional
tests/types/probe_arc296_enum_map_ctor.rs four #[ignore] removed
```

Bare `:wat::core::Some` is not intercepted (`identifier::path` = `:wat::core`, not a `TypeDef::Enum`). Unit and tagged both take the map form (the two mint paths in `register_enum_methods`). The synthesized positional Function is no longer reachable at the user-facing FQDN.

---

## What the next stone has to do

A wat-fix of positional variant construction onto the map form (shape: `wat-scripts/fixes/positional-to-kwargs.wat`). Worklist starts at the 659 stdlib sites this `--check` already named; the floor will add the rest. Until that lands, every `--check` of every file is EXIT=1, including this probe's control. The form they would move onto exists.
