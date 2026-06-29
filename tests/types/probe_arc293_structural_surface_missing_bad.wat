;; tests/types/probe_arc293_structural_surface_missing_bad.wat — negative fixture
;;
;; Arc 293.3-core — a struct MISSING the surface member must FAIL.

(:wat::core::defsurface :geo::Shape
  :features [color <- :wat::core::String])

(:wat::core::defstruct :geo::Bare
  [other <- :wat::core::i64])

(:wat::core::defn :geo::accepts-shape [s <- :geo::Shape] -> :wat::core::bool
  true)

(:wat::core::defn :user::main [] -> :wat::core::bool
  (:geo::accepts-shape (:geo::Bare 42)))
