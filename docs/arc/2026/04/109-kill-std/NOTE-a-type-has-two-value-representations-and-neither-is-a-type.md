# NOTE — a type has TWO value representations, and neither of them is a type

> Builder, 2026-08-29, reading `:wat::runtime::return-type-of`'s doc block during arc 255's P6-c-W3:
> *"why does this not return a type for the type?… something like wat.type/Type — it's what all
> types are? wat.type/i64 /is a/ wat.type/Type?"*
>
> Filed in arc 109 because this is a types-as-forms question, not a registry one. **No row, no
> stone, nothing drawn** — this records what is true and what it costs, so whoever opens the
> question inherits the measurement instead of re-taking it.

## Measured, four lines, on the shipped binary at `fa0713722`

```wat
(:wat::core::let [f (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)]
  (:wat::runtime::return-type-of f))                       ;=> "wat::core::i64"     a String
  (:wat::core::type (:wat::runtime::return-type-of f))     ;=> "wat::core::String"
  (:wat::runtime::extract-arg-types
     (:wat::runtime::signature-of-fn f))                   ;=> [wat.type/i64]       a WatAST
  (:wat::core::type (:wat::core::nth ts 0))                ;=> "wat::WatAST"
```

## What that says

**1. One reflection family, two representations of "a type".**

```
:wat::runtime::return-type-of     -> Value::String, FQDN, COLON-FREE      runtime.rs (eval_return_type_of)
:wat::core::type                  -> Value::String, FQDN, COLON-FREE      the same convention
:wat::runtime::extract-arg-types  -> Value::wat__WatAST, `wat.type/i64`   runtime.rs:14048
:wat::runtime::signature-of-fn    -> the same `wat.type/` nodes           arc 294.f
```

**Neither is wrong on its own terms, and that is what makes it worth writing down.** Each side has a
recorded reason:

- `return-type-of`'s doc says it is *"the STATIC sibling of `(:wat::core::type <value>)`… colon-free,
  in the SAME convention, so the two are directly comparable."* Deliberate, and it achieves exactly
  what it claims.
- Arc **294.f** moved the signature surface to canonical `wat.type/` nodes — *"Reflection is now
  ZERO-holon"* (`runtime.rs:12499-12505`) — and `extract-arg-types` returns the sub-node **verbatim**,
  *"no re-canonicalization, no HolonAST bridge."* Also deliberate, and also right.

They were right about **different neighbours**. `return-type-of` aligned with `type`; the signature
family aligned with the AST. Nothing ever asked the two to agree, so nothing caught that they don't.
A consumer holding a return type and an argument type holds a `String` and a `WatAST` and must
convert to compare them. `[[feedback_a_slot_with_two_implementations_is_two_slots]]`

**2. Asking a type for its type gives you `"wat::core::String"`.** That is the builder's question
answered literally. There is no `Type`. `wat.type/i64` is not an inhabitant of `wat.type/Type` — at
the value level it is a `wat::WatAST` symbol node, and its *stringly* twin is a `wat::core::String`.
Types are **erased to a representation** at the reflection boundary; the reflection surface hands
back a *description of* a type, never a type.

## What opening this would actually cost — the honest part

`wat.type/i64 : wat.type/Type` is a **kind**, and minting one is not a verb-level change:

- **Every reflection verb's `@ret` changes**, and its TypeScheme with it.
- `:wat::core::type` currently returns a String that is compared with `=` all over the corpus and by
  rete. A `Type` value needs equality, rendering, EDN framing, and a `wat.type/` reader story.
- **The parametric forms** — `(Head :- [args])` — must be inhabitants too, so `Type` is not an enum
  of names; it is the whole type-expression grammar reified as values.
- FM 10 applies and points the same way: *when the substrate seems to be missing something, the
  answer is usually a new ENTITY KIND, not a type-system feature.* A `Type` kind IS an entity-kind
  addition — which is the cheap end — **but it lands on `TypeExpr`, which is the type system.**

⚠ **And the sequencing objection is decisive for now.** THE ROAD is `1 home everything · 2 crates ·
3 kill :: in keywords · 4 every call head a symbol · 5 EDN/Clojure-compliant · 6 totality`. **Step 3
rewrites how a type is spelled.** Minting `Type` before the spelling settles means minting it twice.

## The cheap thing that is NOT this

If the two representations ever need to be reconciled *before* `Type` is on the table, the small
move is to make `return-type-of` join its own family — return the `wat.type/` node its siblings
already return — and let a caller who wants the colon-free String ask `type` for it. **That is a
breaking change to a shipped `@ret` and it is NOT proposed here**; it is recorded so the option is
visible when someone weighs the big one.

★ **What is genuinely cheap and worth doing whenever the reflection docs are next touched:**
`return-type-of`'s doc says it is the static sibling of `type` and says nothing about differing from
`extract-arg-types`. **Its own prose is what hides the split.** A sentence naming the other
representation would have made this question unnecessary — and the builder found it by reading the
doc block, which is exactly the reader the doc was for.
