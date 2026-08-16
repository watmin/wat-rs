;; tests/types/probe_arc293_structural_surface.wat — co-located fixture (positive case)
;;
;; Arc 293.3-core — a STRUCT structurally satisfies a defsurface.
;; RED at HEAD: defsurface is unknown; :geo::Shape does not resolve.

(:wat::core::defsurface :geo::Shape
  :nature :wat::core::Struct
  :features [color <- :wat::core::String])

(:wat::core::defstruct :geo::Circle
  [color <- :wat::core::String  radius <- :wat::core::f64])

;; accepts ANYTHING with the Shape surface; Circle has `color` ⇒ structurally satisfies it
(:wat::core::defn :geo::accepts-shape [s <- :geo::Shape] -> :wat::core::bool
  true)

(:wat::core::defn :probe::drive [] -> :wat::core::bool
  (:geo::accepts-shape (:geo::Circle :color "red" :radius 2.0)))
