# Arc 195 — Struct kwarg constructor (`Struct/from`)

**Status:** STUB. Captured 2026-05-15 from user direction. Not yet designed.

## Goal

Make struct construction less annoying. Today the canonical constructor is positional:

```scheme
(:my::Point/new 0 0)
(:my::Failure/new "msg" :None :Vector [] :None :None)   ;; ← order-dependent, easy to mis-arrange
```

Mint a kwarg-shaped constructor as an alternative:

```scheme
(:my::Point/from
  :x 0
  :y 0)

(:my::Failure/from
  :message  "msg"
  :location :None
  :frames   (:Vector [])
  :actual   :None
  :expected :None)
```

## Why

- **Order-independent:** users name what they're setting; no off-by-one when a field gets inserted into the middle of a struct
- **LLM-friendly:** matches the kwarg pattern LLMs already reach for; reduces drift when reading/writing struct construction sites
- **Self-documenting:** the call site reads as documentation of which value lands in which field
- **Clojure-faithful:** parallel to Clojure's `map->Foo` shape (positional `->Foo` + kwarg `map->Foo` both ship)

## Proposed shape (sketch only — final form earned through four-questions)

`:T/from` is the kwarg constructor; `:T/new` stays as the positional form. Both produce the same struct value. Compile-time check: every field of `T` is named exactly once in the `:T/from` call; unknown keys fail check; duplicates fail check.

Possible syntax variants to evaluate:
- Plain keyword-arg pairs: `(:T/from :x 0 :y 0)` — matches Clojure idiom
- Bindings-vector form: `(:T/from [:x 0 :y 0])` — parallels arc 168 let-shape
- Map-literal form: `(:T/from {:x 0 :y 0})` — once map literals exist

Pick via four-questions when the arc gets DESIGNed.

## Open questions

- Does `Struct/from` apply to every struct type uniformly (substrate-minted via `defstruct`/`struct`), or opt-in per type?
- Static-type check coverage: how does the compile-time field-existence check integrate with `register_types` + struct-field-accessor pregen (arc 170 Gap F-1)?
- Interaction with arc 169's struct-destructure form A — symmetry: if `{:x x :y y}` destructures fields by name on the LHS of `let`, the `:T/from` constructor is the natural symmetric form on the RHS
- Macro vs special form vs runtime: probably defmacro that expands to `:T/new` with reordered args, but with compile-time check of key set; verify when designing

## Out of scope (until DESIGN)

- Anything that touches `:T/new`'s positional form (stays)
- Default values for unspecified fields (separate feature; possibly its own arc)
- Partial application / builder pattern

## Cross-references

- Arc 168 — let-flat-shape + tuple destructure (the parallel binding-vector idiom)
- Arc 169 — struct-destructure form A (the LHS symmetric partner; this arc is the RHS partner)
- Arc 170 slice 1f Gap F-1 — `preregister_struct_accessors_from_form` (the field-name machinery `Struct/from`'s compile-check would lean on)
- Memory `feedback_eprintln_is_terminal` — Failure struct construction is an immediate beneficiary; today's positional `Failure/new` is the canonical pain point this arc relieves
