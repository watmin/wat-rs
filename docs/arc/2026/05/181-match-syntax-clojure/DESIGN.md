# Arc 181 — `match` arm syntax (Clojure-flavored, parallel to cond)

**Status:** stub opened 2026-05-12 per user direction.

## Motivation

User direction 2026-05-12: match arms still use scheme
paren-wrap shape `(pattern body)`; pair with arc 180 (cond
flat pairs) for parallel surface.

Current shape (scheme paren-wrap arm):

```scheme
(:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
  ((Some line)
    (:wat::io::IOWriter/print stdout line))
  (:None
    (:wat::io::IOWriter/print stderr "no input\n")))
```

Target shape (Clojure flat pattern/body pairs, mirroring
core.match + arc 180 cond):

```scheme
(:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
  (Some line)  (:wat::io::IOWriter/print stdout line)
  :None        (:wat::io::IOWriter/print stderr "no input\n"))
```

Patterns stay as data (keyword for unit-variant, list for
tagged-variant); the body follows directly without outer
paren grouping.

## Sketch (placeholder; user fills the design)

TBD. Substrate parser + arm-iteration logic change; workspace
sweep of every match call site. Walker for legacy paren-wrap
shape during migration window.

## Cross-references

- arc 091 (pattern grammar + classifier + type-check)
- arc 098 (runtime walker for patterns)
- arc 180 (cond flat-pairs; parallel revision)
- arc 168 (let flat-vector shape) — original flat-shape precedent
