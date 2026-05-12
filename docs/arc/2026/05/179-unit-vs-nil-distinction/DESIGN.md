# Arc 179 — `()` ≠ `:wat::core::nil` (distinct types)

**Status:** stub opened 2026-05-12 per user direction.

## Motivation

> *"i want to make () != :wat::core::nil ... they both represent
> Rust's Unit... but... i saw a bunch of new additions who
> declared a ret val of :wat::core::nil and then their final
> body statement was ()"*

Today both `()` (the empty literal) and `:wat::core::nil` (the
canonical "no value" type) collapse to Rust's `Unit` at the
substrate level. The result: code can declare `-> :wat::core::nil`
and have its final body statement be `()` — the type-checker
accepts the symmetry, but the intent reads inconsistent.

This arc splits the two. Likely shape (TBD per user design):
- `()` becomes a distinct empty-shape (empty tuple? empty
  collection literal?)
- `:wat::core::nil` remains the canonical no-value return type
- Type-checker rejects mixing them; sites currently using `()`
  where `:wat::core::nil` is intended need explicit `:wat::core::nil`

## Sketch (placeholder; user fills the design)

TBD — user holds the design in their head; this stub is the
persistence handle.

## Cross-references

- arc 109 slice 1d (mint :wat::core::unit; retire :() as type)
  — the predecessor that minted unit; this arc separates the
  literal from the type-name
- arc 153 (rename unit → nil) — the user-facing keyword swap
- arc 109 follow-up: rename :wat::core::unit → :wat::core::Unit
  (task #182, pending) — orthogonal capitalization concern
- arc 165 (tuple PascalCase rename) — may inform `()` =
  "empty Tuple" framing
