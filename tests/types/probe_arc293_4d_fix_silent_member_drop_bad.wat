;; 293.4d-fix — members written OUTSIDE the single `[...]` vector must be a HARD ERROR.
;; This is the stale `definterface` shape (field in the vector, method members as separate
;; top-level args). It has exactly 4 args after the head, and arg[1] is `[color]` (not :holder),
;; so `parse_defsurface` USED to read `[color]` as the whole member list and SILENTLY DROP the
;; (area …) / (label …) method members — declaring a surface weaker than written, no error.
;; After 293.4d-fix: a leftover form after the member vector is rejected.

(:wat::core::defsurface :t::Bad
  [color <- :wat::core::String]
  (area  [self] -> :wat::core::f64)
  (label [self] -> :wat::core::String))
