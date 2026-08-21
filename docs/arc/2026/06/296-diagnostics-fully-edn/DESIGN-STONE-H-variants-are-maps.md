# 296 · DESIGN STONE H — a variant is a tagged map, like everything else

> **STATUS: DRAWN, NOT BUILT.** Ruled by the builder 2026-08-15.

## THE FORM

```clojure
;; declaration — today
(:wat::core::defenum :wat::telemetry::Numeric :wat::enum::Pure
  :I64 [val <- :wat::core::i64]
  :F64 [val <- :wat::core::f64])

;; declaration — after 251's clojure flip
(wat.core/defenum wat.telemetry/Numeric wat.enum/Pure
  I64 [val :- wat.type/i64]
  F64 [val :- wat.type/f64])

;; the wire — IDENTICAL before and after 251
#wat.telemetry/Numeric.I64 {:val 42}
```

**Tag:** `#<ns>/<Enum>.<Variant>`. **Body:** a map keyed by the keywordized binder.

```clojure
#wat.core/Option.Some {:value "x"}     #wat.core/Option.None {}
#wat.core/Result.Ok   {:value "x"}     #wat.core/Result.Err  {:error "e"}
```

## WHY — this DELETES AN EXCEPTION, it does not add a mechanism

The rule is already live and already proven **for records**: `(defrecord :usr::Point [x <- :i64 y <- :i64])`
renders `#usr/Point {:x 3 :y 4}` — binder → keywordized map key. G-2's golden flipped to exactly that
form hours before this stone was drawn.

The tagged variant is the one place in the substrate where named data does **not** follow that rule.
It declares binders (`EnumVariant::Tagged { fields: Vec<(String, TypeExpr)> }` — always named), then
throws them away into a positional vector. This stone removes the exception. One rule for named data,
both kinds, both directions.

Three things fall out:

1. **The last `field-N` evaporates.** `edn_shim.rs:2727`'s enum arm needs names, the wire doesn't
   carry them, so it re-derives from the registry through `enum_variant_field_names` — which has three
   silent `return vec![]` arms that each render `field-0`. Carry the names and there is nothing to
   re-derive. No ruling needed on those arms; they go with the function.
2. **A live tag collision closes** (see the wall, below).
3. **The wire survives 251.** The tag is built from the type path and the variant name; neither the
   dot rule nor the map body cares whether the declaration currently spells things `:I64` or `I64`.
   This stone and the clojure flip do not have to be sequenced against each other.

## ⛔ THE WALL — a dot in a record name has NO FORM

The discriminator is: **a dot in the tag's NAME half means variant.** For that to be a wall rather
than a coincidence, a record must be *unable* to produce a dotted name.

Measured 2026-08-15: no type name in the corpus contains a dot, and `Tag::try_ns` validates only the
name's first character — a dot inside is legal EDN and would be accepted. So today the property holds
by luck. **Ban it authoritatively at the registration door, `src/types.rs:609`**, where
`ReservedPrefix` already fires. Same shape as 251.8a-ii, where `$bound/x` was refused at the reader
rather than checked downstream: the forgery gets no form.

Builder: *"records must disallow dots in names... authoritatively."*

### What the wall closes

`tag_from_type_path` splits on the LAST `::`, so today:

| | tag |
|---|---|
| record `:usr::Shape::Circle` | `#usr.Shape/Circle` |
| enum `:usr::Shape` variant `Circle` | `#usr.Shape/Circle` |

**Byte-identical, from the identical call.** Body shape is the only thing separating them, which means
the current design does not *have* a discriminator in the tag — it has an ambiguity that the body
happens to mask. Under this stone the record keeps `#usr.Shape/Circle` and the variant becomes
`#usr/Shape.Circle`. Distinct by construction, and the dot-ban keeps them that way.

### And it closes the unit seam

`#wat.core/Option.None {}` versus a zero-field record `#wat.core/Nothing {}` — both empty maps. The
tag's dot separates them, so body shape stops carrying any burden at all. That was the one genuine new
ambiguity in this design; the wall is what resolves it.

## WHAT IS LOST, HONESTLY

`ForeignRecord` (map body) / `ForeignVariant` (vector body) is a **real, reliable** discriminator —
the writer has no other behaviour, so for anything wat wrote, the shape is carried information, not an
accident of the sender. It was argued in this session that foreign classification was "always a
guess"; that was **wrong** and the builder corrected it.

The classification survives — it moves from the body to the tag, where it is *more* honest: today the
body says "variant" while the tag says nothing and can collide. After this, the tag says it. Foreign
decode reads the dot instead of the body shape.

## THE FOUR QUESTIONS

- **Obvious?** YES — `#wat.telemetry/Numeric.I64 {:val 42}` says what it is and names its field. A log
  line is readable without a registry.
- **Simple?** YES — one rule for named data (binder → keywordized key), one discriminator in one place
  (the tag), one wall making it real. Strictly fewer moving parts than today's body-shape rule plus an
  unguarded colliding tag.
- **Honest?** YES — nothing is dropped: names that were declared and discarded are now carried, and an
  existing ambiguity closes. The re-derivation that produced the last `field-N` stops existing.
- **Good UX?** YES — logs stay readable EDN; consumers get named fields; the double-wrap
  (`{:is … :data …}`) tax on every value never happens.

## MIGRATION — measured

**213 occurrences across 103 files.**

| tag | count |
|---|---|
| `#wat.core.Option/Some` | 84 |
| `#wat.core.Option/None` | 21 |
| `#wat.core.Result/Ok` | 10 |
| `#wat.core.Result/Err` | 8 |
| everything else (`Outcome` `Cmd` `Frame` `LociDiedError` `PoolMsg` `Purity` …) | 90 |

58% is Option/Result — the most mechanical part, and both are still Rust literals in `types.rs`
(neither is among the six enums generated from wat). Moving them to wat declarations is the arc's
standing wat-as-source-of-truth move and belongs in this stone: their binders become the wire keys, so
the binder names should live where they can be read.

**The `.wat`/golden sweep is a wat-fix codemod, not hand edits** (R21). The Rust half is small: two tag
builders (`tag_from_type_path`, `enum_variant_ns`), the writer's `Value::Enum` arm, the reader's enum
coerce, the foreign decode's shape test, and the deletion of `enum_variant_field_names`.

## STOP TRIGGERS

- **STOP-1 — a record name with a dot already exists in the corpus.** The census says none does. If the
  wall goes red on real code, the wall is right and that name is the finding — report it, do not widen
  the rule to admit it.
- **STOP-2 — a variant whose binder collides with another binder in the same variant.** Map keys must
  be unique; the declaration already forbids duplicate field names, but prove it rather than assume it.
- **STOP-3 — a golden that cannot be migrated by the codemod.** Hand-editing `.wat` for a structural
  rewrite is the thing the codemod exists to prevent. Report the shape that defeated it.

## WHAT THIS STONE IS NOT

Not 251's keyword→symbol flip. The wire form above is identical on both sides of that migration, so
neither blocks the other.

---

## ★ AMENDMENT 2026-08-20 — the variant payload takes `:-`, and the field names DO NOT change

Two rulings from arc 109's `:-` work land on this stone. H is still DRAWN, NOT BUILT; these amend
the form it will build.

**1 — the payload spec is a param-spec, so it takes the operator.** Arc 109 ruled `:-` as the one
declaration operator: *"the symbol ` :- ` is declaring 'this thing on the left is parameterized by
the thing on the right'… this is the same as arg-spec and ret-type."* A variant's payload is a
declaration like any other, so H's variant line gains it — and a parametric enum gains the
type-param binder in the same motion:

```clojure
;; H as drawn 2026-08-15
(wat.core/defenum wat.telemetry/Numeric wat.enum/Pure
  I64 [val :- wat.type/i64]
  F64 [val :- wat.type/f64])

;; H as amended — the payload is :--marked; a parametric enum binds its type params
(wat.core/defenum wat.telemetry/Numeric wat.enum/Pure
  I64 :- [val :- wat.type/i64]
  F64 :- [val :- wat.type/f64])

(wat.core/defenum wat.core/Option :- [T] wat.enum/Pure
  Some :- [val :- T]
  None :- [])
```

Builder, 2026-08-20, on `Some`/`None`/`Ok`/`Err`: *"those are all enums... they are being updated
too... **we are now asserting they have param-spec as well**."*

**2 — the field names stay `value` / `value` / `error`.** This one is a DECISION, not a
transcription, and it is worth writing down precisely because it is a decision to change nothing.

H repeals `src/types.rs`'s standing note that *"Field names are INTERNAL, not API: the wire form is
positional — measured, `(:wat::core::Some 42)` prints `#wat.core.Option/Some [42]`."* Once a variant
is a tagged map, the field name is **on the wire** (`{:value 42}`), so any rename here is a
wire-format change, not a cosmetic one.

A sketch in conversation used `val` / `ok` / `err`; the registered names (`src/types.rs`, the
2026-08-05 Option/Result enum registration) are:

| variant | field |
|---|---|
| `Option.Some` | `value` |
| `Result.Ok` | `value` |
| `Result.Err` | `error` |

Builder: *"value, value, error are fine - i misremembered what we named them."* **So H ships with
no field rename**, and the wire form for these three is `{:value …}` / `{:value …}` / `{:error …}`.

★ Consequence for scope: this stone now ALSO closes the Option/Result half of
`109/NOTE-six-parametric-constructors-never-got-the-bracket.md` — `Some`/`None`/`Ok`/`Err` are not
a separate constructor-wiring stone, they are H's variants declaring their param-spec. Amend H;
do not mint a sibling.
