# Arc 180 — `cond` flat predicate/result pairs (Clojure-flavored)

**Status:** stub opened 2026-05-12 per user direction.

## Motivation

User direction 2026-05-12: cond is the most-used multi-way form
still in scheme paren-pair syntax. Pair with let/fn/defn flat
shapes (arcs 168/167/166) for surface consistency.

Current shape (scheme paren-pair):

```scheme
(:wat::core::cond -> :String
  ((:wat::core::= code 0) "success")
  ((:wat::core::= code 1) "runtime error")
  (:else                  "unknown"))
```

Target shape (Clojure flat predicate/result pairs):

```scheme
(:wat::core::cond -> :String
  (:wat::core::= code 0)  "success"
  (:wat::core::= code 1)  "runtime error"
  :else                   "unknown")
```

## Sketch (placeholder; user fills the design)

TBD. Substrate parser change + workspace sweep of every cond
call site. Walker for legacy paren-pair shape during migration
window (Pattern 3 substrate-as-teacher).

## Cross-references

- arc 168 (let flat-vector shape) — predecessor flat-shape revamp
- arc 167 (fn flat signature) — same family
- arc 091 / arc 098 (pattern grammar; if match adopts same shape
  in arc 181, the pair-of-predicates idiom carries over)
- arc 181 (match syntax revision; parallel)
