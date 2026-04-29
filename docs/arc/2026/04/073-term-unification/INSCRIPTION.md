# Arc 073 — Term store / HolonAST as Prolog term — INSCRIPTION

**Status:** PARTIAL — slices 1-3 shipped; slice 4 (`TermStore<V>`)
**superseded by [arc 074](../074-holon-store/INSCRIPTION.md)**
(`HolonStore<V>`). Sealed 2026-04-28.

This INSCRIPTION formalizes the partial-and-superseded state already
declared in [`DESIGN.md`](./DESIGN.md). The shipped substrate
primitives stand on their own; the cache use case that motivated the
arc found a different operational shape (coordinate-cell, not
template-bucket) and shipped under arc 074.

---

## What shipped (slices 1-3)

- `HolonAST::SlotMarker` — 12th HolonAST variant. (`holon-rs@7586f66`,
  `wat-rs@fb7650c`)
- `:wat::holon::term::template` / `term::slots` / `term::ranges` —
  decomposition primitives that split a HolonAST into its template
  shape + the slot positions + the per-slot ranges. (`holon-rs@e5b96c9`,
  `wat-rs@709ec72`)
- `:wat::holon::term::matches?` — per-slot fuzzy unification predicate.
  Returns `true` when the candidate AST matches the template shape AND
  every slot position falls within its declared range. (`wat-rs@d86e32c`)
- `:wat::core::=` extended to accept HolonAST pairs (substrate gap
  closed alongside slice 2).

These are useful for any consumer that wants explicit
template-and-slots reasoning. Possible future consumers: pattern
matching against AST shapes, query-by-template surfaces, structural
constraint checking.

## What's abandoned (slice 4)

The original slice 4 plan — `TermStore<V>` as the cache primitive —
fell over when reviewing the actual operational mechanism. The cache
isn't a template-keyed bucket; it's a coordinate-keyed bucket with
cosine readout. Templates would have served the THEORETICAL
pre-filtering ("asts in different neighborhoods can't match"), but
the operational mechanism is `pos: f64`-indexed cells with cosine.

Arc 074's `HolonStore<V>` is what shipped for the cache use case.

The half-built slice-4 artifacts (`term_store.rs.draft`,
`BUILD-LOG.md`, `FEEDBACK-FROM-OTHER-SESSION.md`) are preserved under
`./archived/` as historical record of a path explored and rejected.

## Cross-references

- [Arc 074 — `HolonStore<V>`](../074-holon-store/INSCRIPTION.md) —
  the operational cache primitive that shipped instead.
- [Arc 076 — therm-routed hologram](../076-therm-routed-hologram/INSCRIPTION.md)
  — built on arc 074's coordinate-cell shape.
