# SCORE — 296 L: reflection answers for every type, with one row

No commit. Floor left to the orchestrator (read `^ +Summary`, never a tail). Lands on J/K and the match-arm stones; those were not reverted.

`:wat::runtime::type-of` answers with one declared row, `#wat.runtime/TypeInfo`, covering all six `TypeDef` kinds. An Enum body's variants carry declared field names and types **in declaration order**.

---

## Expectation 1 — the probe

**PASSED.** `reflection_answers_for_an_enum`: exit 0, non-empty answer. `#[ignore]` = 0.

The probe's own stdout (not asserted there; shown so row 4 is visible in the same value):

```
#wat.runtime/TypeInfo {
  :name :probe/Box
  :kind #wat.runtime/TypeKind.Enum {}
  :type-params []
  :body #wat.runtime/TypeBody.Enum {
    :purity #wat.runtime/TypePurity.Pure {}
    :variants [
      #wat.runtime/TypeVariant {:name :Full :fields [#wat.runtime/TypeField {:name :payload :type wat.type/i64}]}
      #wat.runtime/TypeVariant {:name :Empty :fields []}
    ]
  }
}
```

## Expectation 2 — all six kinds

Each kind is a named row. Fixture `tests/reflection/probe_arc296_type_of_six_kinds.wat`. Test `reflection_answers_for_all_six_kinds_and_variant_field_order` asserts the exact stdout.

| kind | fixture type | answered? |
|---|---|---|
| Aggregate | `:probe::Rec` (`defrecord`) | **yes** — kind `Aggregate` |
| Enum | `:probe::Box` | **yes** — kind `Enum` |
| Newtype | `:probe::Count` | **yes** — kind `Newtype`, body `{:inner wat.type/i64}`. Not an omission: kind + inner type **is** the answer |
| Alias | `:probe::Alias` | **yes** — kind `Alias` |
| Union | `:probe::Num` (`typeunion`) | **yes** — kind `Union` |
| Surface | `:probe::Surf` (`defsurface`) | **yes** — kind `Surface` |

None skipped. STOP-1 held.

## Expectation 3 — the row is declared in wat

`grep -n TypeInfo wat/**/*.wat`:

```
wat/runtime-typeinfo.wat:66:(:wat::core::defrecord :wat::runtime::TypeInfo
```

`src/types.rs` sources it:

```
wat_enum_register_from!(env, "wat/runtime-typeinfo.wat", ":wat::runtime::TypeKind")
… TypeNature TypePurity TypeSurfaceMember TypeBody …
wat_record_from!(env, "wat/runtime-typeinfo.wat", ":wat::runtime::TypeField")
wat_record_from!(env, "wat/runtime-typeinfo.wat", ":wat::runtime::TypeVariant")
wat_record_from!(env, "wat/runtime-typeinfo.wat", ":wat::runtime::TypeInfo")
```

File loads after `Record.wat` (defrecord expands via `Record::def`). `every_wat_record_from_source_is_in_the_stdlib_load_set` PASS. `no_hand_written_enumdef_literals_in_types_rs` PASS. `aggregate_literals_are_only_category_roots` PASS.

**STOP-2 / the wall:** a hand-written `TypeDef::Aggregate(AggregateDef { name: ":wat::runtime::TypeInfo", … })` **would be refused** by `no_hand_written_type_floor` (TypeInfo is not a category root). A hand-written `EnumDef` for TypeKind/TypeBody would be refused by `no_hand_written_enumdef`. The wall catches the literal. No finding against the wall.

## Expectation 4 — a variant's field names, in order

`:probe::Box::Pair` is declared `[left <- :i64  right <- :String]`.

The six-kind fixture walks `TypeBody::Enum` → `variants` (declaration order) → index 1 (`Pair`) → `TypeVariant/fields` → `TypeField/name`.

Stdout lines 7–8: `:left` then `:right`. A set would not have distinguished order. A guess from the binder would have written `:_cur`. The names come from `EnumVariant::Tagged { fields: Vec<(String, TypeExpr)> }`, the authority the codemod lacked.

## Expectation 5 — Aggregate nature and type_params

- `:probe::Rec` body is `TypeBody::Aggregate {:nature TypeNature::Record, …}`. Stdout line 9: `Record`.
- `:wat::core::Option` row carries `type-params ["T"]`. Stdout line 11: `T`.

`field-names-of` still cannot report these; `type-of` can.

## Expectation 6 — nothing retired

Six-kind fixture line 10 is `(:wat::core::first (:wat::runtime::field-names-of :probe::Rec))` → `:alpha`. Same answer as before. `field-types-of` is untouched. STOP-5 held.

## Expectation 7 — no per-question verb

No `variants-of` / `variant-fields-of` intrinsic. One pre-existing comment in `wat/telemetry.wat:292` ("a future `variants-of`") — not a verb, not this stone. STOP-3 held.

## Expectation 8 — the codemod was not touched

This ingest did not edit `wat-scripts/fixes/`. RELAND 4 is out of scope. STOP-4 held.

## Expectation 9 — the floor

**Not run.** Orchestrator. This stone does not touch match-arm forms or the codemod, so it does not change the match-arm residue Claude named as 16. Targeted tests for this stone: 0 failed.

## Expectation 10 — clippy

`cargo clippy --release --all-targets --workspace` **0 errors**. 5 pre-existing dead-code warnings (`Coverage::Wildcard`, `pattern_coverage`, `ident_span`, `try_match_pattern_ast`, `substitute_many`).

---

## STOP rows

| STOP | result |
|---|---|
| STOP-1 only the kinds a caller needed | **held.** All six answered. Newtype is kind + inner, not skipped |
| STOP-2 hand-written Rust literal for the row | **held.** wat_record_from / wat_enum_register_from. The J/K walls would refuse a TypeInfo/TypeKind literal |
| STOP-3 per-question verb | **held.** One verb, one row |
| STOP-4 codemod fixed here | **held.** Not touched |
| STOP-5 field-names-of / field-types-of retired | **held.** field-names-of `:probe::Rec` still `[:alpha]` |

## Targeted checks

```
cargo nextest run --release -E 'test(reflection_answers_for_an_enum)|test(reflection_answers_for_all_six_kinds)|test(every_wat_record_from_source)|test(no_hand_written)|test(verify_stdlib)|test(every_dispatched_verb)|test(checker_skip_debt)'
  14 passed (plus purity + debt in the earlier filter)
cargo clippy --release --all-targets --workspace
  0 errors
```
