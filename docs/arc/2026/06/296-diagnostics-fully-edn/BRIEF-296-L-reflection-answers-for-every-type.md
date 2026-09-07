# BRIEF — 296 L: reflection answers for every type, with one row

> Read `DESIGN-STONE-L-reflection-answers-for-every-type.md` first. The probe is committed and RED:
> `tests/reflection/probe_arc296_reflection_answers_for_every_type_kind.rs`.

## THE WORK

Add `:wat::runtime::type-of`, answering with ONE declared row that covers **every** `TypeDef` kind —
`Aggregate · Enum · Newtype · Alias · Union · Surface`. For an `Enum` the row must carry its
variants, and each variant its **declared field names and types**, because that is the fact tooling
needs and today cannot obtain.

**The row's shape is DECLARED IN WAT** and Rust sources from it — `wat_record_from!` /
`wat_enum_register_from!`, exactly as stones J and K established. Reflection's own answer must not
be a hand-written Rust literal.

## READ IN ORDER

```
src/intrinsic/reflect.rs                  field-names-of / field-types-of — the existing shape and
                                          its refusals ("is not a struct/record type", "unknown type")
src/types.rs  TypeDef / EnumDef            what must be reportable. EnumVariant::Tagged carries
              EnumVariant / AggregateDef   { name, fields: Vec<(String, TypeExpr)> } — the answer
              AliasDef / UnionDef          already exists, it is simply not exposed
src/runtime.rs:10047 builtin_enum_variant_names   the hand table + the TypeEnv fall-through
src/intrinsic/reflect.rs (metadata-of)     THE PRECEDENT — one lookup, whole row, landed 2026-09-06
wat/doc.wat · wat/runtime-meta.wat         where a reflection row's shape is declared today
```

## SKETCH

```clojure
(wat.runtime/type-of probe/Box)
;; #wat.runtime/TypeInfo {:name … :kind … :type-params [] :purity … :body …}
;;   where :body for an Enum carries variants, each with its declared field names + types
```

`:kind` mirrors `TypeDef`'s discriminant. A row that cannot express one of the six kinds is
incomplete — see STOP-1.

## STOP TRIGGERS

- **STOP-1 — the row covers only the kinds this stone's caller needs.** All six, or report which
  cannot be expressed and why. A reflection surface that answers for the kinds someone happened to
  need is how the current hole was made.
- **STOP-2 — the row's shape is a hand-written Rust literal.** Stone J's wall exists; declare it in
  wat and source from it. If the wall does not catch a `TypeInfo` literal, that is a finding about
  the wall.
- **STOP-3 — a per-question verb is added instead** (`variants-of`, `variant-fields-of`, …). The
  contract decision is ONE row; N verbs is the shape being replaced, and each new one is another
  thing to remember to add next time.
- **STOP-4 — the codemod is fixed in this stone.** RELAND 4 is out of scope; bundling makes a red
  ambiguous between the reflection surface and its first consumer.
- **STOP-5 — `field-names-of` / `field-types-of` are retired here.** They keep working. Whether the
  row subsumes them is a later ruling, not a side effect.

## EXPECTATIONS

| # | what | expected |
|---|---|---|
| 1 | the probe passes | `reflection_answers_for_an_enum`; `#[ignore]` 0. Names the variant AND its declared field |
| 2 | ⛔ all six kinds | one fixture per `TypeDef` kind, each answered — or each unanswerable one named with its reason (STOP-1) |
| 3 | ⛔ the row is declared in wat | `grep 'TypeInfo' wat/**/*.wat` finds the declaration; `src/` sources it via the derive |
| 4 | a VARIANT's field names are obtainable | the exact fact the codemod needed: given an enum + variant, its declared field names in order |
| 5 | nothing retired | `field-names-of` / `field-types-of` still answer as before (STOP-5) |
| 6 | the floor | `0 failed`, read from `^ +Summary`, never a tail |
| 7 | clippy | 0 |

## WHY THIS BEFORE THE CODEMOD

The codemod guessed because it could not ask. Patching the codemod leaves the next tool to guess
again — and today produced three separate instances of exactly that, two of which were caught only
by a wall and one of which reached 1869 files.
