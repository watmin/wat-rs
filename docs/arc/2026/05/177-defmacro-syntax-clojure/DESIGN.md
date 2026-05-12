# Arc 177 — `defmacro` syntax revision (defn/fn-symmetric, Clojure-flavored)

**Status:** stub opened 2026-05-12 per user direction.

## Motivation

> *"revise defmacro syntax — specifically make the args like
> defn, fn and overall more clojure-y"*

`defmacro`'s argument-list shape should mirror arc 166 (defn) +
arc 167 (fn flat signature). Today it's still in a hybrid shape
that breaks symmetry with the `def`-family forms. Symmetric
arg-list reduces surface cognitive load + lets one mental
model cover defn / fn / defmacro / `defclause` (arc 174).

## Sketch (placeholder; user fills the design)

TBD.

## Cross-references

- arc 166 (defn form) — args shape to mirror
- arc 167 (fn flat signature) — args shape to mirror
- arc 172 (Scheme → Clojure macro flavor swap; lexical pivot, shipped)
- arc 173 (Clojure macro feature parity; auto-gensym, &form/&env)
- arc 174 (defclause; sibling in the `def`-family shape revision)
