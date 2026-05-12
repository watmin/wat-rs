# Arc 174 — `defclause` (N-ary multi-clause functions, Erlang-style)

**Status:** stub opened 2026-05-12 per user direction.

## Motivation

> *"implement defclause — this is where we'll have N-ary
> functions — it'll be erlang-y and replace dispatch"*

Erlang-style multi-clause function definitions. A single name
binds N clauses; each clause has its own arity + argument-shape
pattern; the runtime dispatches to the first matching clause.

Replaces the existing dispatch mechanism (arc 146) with a
language-level construct in the `def`-family.

## Sketch (placeholder; user fills the design)

TBD — user holds the design in their head; this stub is the
persistence handle.

## Cross-references

- arc 146 (substrate dispatch mechanism) — this arc supersedes
- arc 148 (arithmetic / comparison via dispatch handlers) — likely consumer
- arc 166 (defn form) — `defclause` joins the `def`-family
- arc 167 (fn flat signature) — shares the argument-shape primitives
