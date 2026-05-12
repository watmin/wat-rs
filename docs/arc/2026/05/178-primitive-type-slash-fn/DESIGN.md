# Arc 178 — Primitive functions in `:wat::core::Type/fn` form

**Status:** stub opened 2026-05-12 per user direction.

## Motivation

> *"revise primitive functions to be of the form
> :wat::core::type/fn"*

Reshape primitive function names from their current heterogeneous
shapes to the canonical `Type/verb` form. Examples (TBD per
user design):

- `:wat::core::i64/+`
- `:wat::core::f64/abs`
- `:wat::core::Vector/length`
- `:wat::core::HashMap/get`

Aligns with the arc 191 "Type/verb shape" doctrine already in
flight for Option/Result methods. Universalizes that pattern
across the core primitives.

## Sketch (placeholder; user fills the design)

TBD.

## Cross-references

- arc 191 (Option/Result method forms — Type/verb shape precedent)
- arc 109 § kill-std (namespace flattening doctrine)
- arc 146 (dispatch mechanism) — current primitive-dispatch path
- arc 148 (arithmetic / comparison handlers — current per-Type impl
  registration; this arc reshapes the SURFACE these handlers expose)
- arc 171 (comma → apostrophe in keyword bodies — apostrophe-
  discriminator for arity / type-variant suffixes)
- arc 174 (defclause — N-ary dispatch construct that may host
  these primitives)
