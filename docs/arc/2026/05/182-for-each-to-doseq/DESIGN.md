# Arc 182 — `for-each` → `doseq` (Clojure naming)

**Status:** stub opened 2026-05-12 per user direction.

## Motivation

User direction 2026-05-12: rename the side-effect iteration
form from scheme/SRFI-named `for-each` to Clojure's `doseq`.

The one-canonical-path doctrine from `docs/ITERATION-PATTERNS.md`
holds — this is a rename, not a synonym-addition. After the arc:
`for-each` is dead; `doseq` is the canonical side-effect
iteration form.

Current:

```scheme
(:wat::core::for-each
  (:wat::core::fn [item <- :Item] -> :wat::core::nil
    (:my::log item))
  items)
```

Target:

```scheme
(:wat::core::doseq
  (:wat::core::fn [item <- :Item] -> :wat::core::nil
    (:my::log item))
  items)
```

Possibly with macro-binding shape closer to Clojure's:

```scheme
(:wat::core::doseq [item <- :Item items]
  (:my::log item))
```

(TBD per user design — the latter avoids the explicit fn
wrapper and matches Clojure's `(doseq [x xs] body)` shape.)

## Sketch (placeholder; user fills the design)

TBD.

## Cross-references

- `docs/ITERATION-PATTERNS.md` — the canonical-iteration doctrine
- arc 168 (let flat-vector shape) — `[binding xs]` binding shape
  precedent if the macro-binding variant is chosen
- arc 174 (defclause) — sibling Clojure-ish form work
