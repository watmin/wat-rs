# 296 · DESIGN STONE L — reflection answers for EVERY type, with ONE row

> Builder, 2026-09-07: *"we need to grow our reflection tooling… you hitting this failure is
> disappointing… reflection has been grown organically… we must be able to do this work at
> runtime… i think we need new intrinsics."*

## THE GAP, MEASURED

```
TypeDef kinds        Aggregate · Enum · Newtype · Alias · Union · Surface      6
reflection answers   field-names-of / field-types-of — Aggregate ONLY          1
```

Two questions about one of six kinds. Even for `Aggregate` it cannot report `nature`
(Struct/Record/HolonRecord), `type_params`, or restrictions. An `Enum` is fully opaque — no
variants, no per-variant fields, no purity, no params. Measured at HEAD:

```
field-names-of :probe::Rec        ->  [:alpha]                          ok
field-names-of :probe::Box        ->  "is not a struct/record type"     refused
field-names-of :probe::Box::Full  ->  "unknown type"                    refused
type-of        :probe::Box        ->  UnknownFunction                   does not exist
```

## ⛔ WHAT THE HOLE COST, TODAY

The match-arm codemod needed exactly one fact — *what are this variant's declared field names* —
and had no way to ask. It guessed the **binder name**, wrote `{:_cur _cur}` where `_cur` is a
binder and not a field, and the guess reached the corpus across 1869 files before surfacing.

★ **The loud failure is the lucky half.** `:_cur` errored only because no such field exists. Had the
variant HAD a field called `_cur`, the arm would have bound the **wrong field, silently**. A
reflection hole does not merely block tooling — **it makes tooling guess**, and a guess about names
fails silently far more often than it fails loudly.

★★ This is the third time in one session a heuristic stood in for an authority that existed
elsewhere: `ns_to_wat_path`'s casing test (halted), stone K's would-be root allowlist (replaced by
asking `Nature::from_root_keyword`), and now this. The pattern is not carelessness — **it is what
happens when the authority is not reachable from where the question is asked.**

## THE CONTRACT DECISION — ONE ROW, NOT N VERBS

`metadata-of` already set this precedent, landed 2026-09-06: it answers with the **whole row** so a
`#wat.doc/Row` renders from one lookup instead of a scan. Reflection over types takes the same
shape:

```clojure
(wat.runtime/type-of probe/Box)
;; => #wat.runtime/TypeInfo { … }   — kind, name, type-params, purity, and a kind-appropriate body
```

**Why a row beats a verb-per-question:**
- adding a `TypeDef` field never needs a new intrinsic — the row grows
- the row's SHAPE is declared in wat and Rust sources from it (stones J/K's principle, applied to
  reflection itself)
- it renders, diffs, and round-trips through the existing EDN tooling — no new surface to learn
- one lookup, not a scan; the `metadata-of` argument verbatim

**The variant question is answerable in the same row** — an `Enum`'s body carries its variants, and
each variant its declared field names and types. The codemod's question becomes a field access.

## THE AUTHORITY ALREADY EXISTS IN RUST

Nothing here is new machinery — it is exposure:

```
src/runtime.rs:10047   builtin_enum_variant_names(type_path, variant)  — the hand table + the
                       TypeEnv lookup it falls through to
src/types.rs           EnumVariant::Tagged { name, fields: Vec<(String, TypeExpr)> }
                       EnumDef { name, type_params, purity, variants }
                       AggregateDef { class, names, fields, nature, restrictions }
```

## FOUR QUESTIONS

| | |
|---|---|
| **Obvious?** | YES — "ask the type what it is" is the question tooling already needed and could not phrase |
| **Simple?** | YES — one verb, one declared row, mirroring one existing enum. Strictly fewer moving parts than six verbs per kind |
| **Honest?** | YES — it closes the gap that MADE the codemod guess, rather than fixing the guess. The alternative (patch the codemod) leaves the next tool to guess again |
| **Good UX?** | YES — a wat program can interrogate any declared type at runtime, which is the stated requirement |

## OUT OF SCOPE, AFFIRMATIVELY

The match-arm codemod's own fix (RELAND 4) — it becomes trivial once this lands, but bundling would
make a red ambiguous between the reflection surface and the codemod's use of it. `field-names-of` /
`field-types-of` stay as they are; this stone adds, it does not retire them (their retirement, if
the row subsumes them, is a later ruling).
