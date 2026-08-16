;; tests/types/probe_arc293_record_surface_core.wat — co-located fixture (core record positive)
;;
;; Arc 293.3-records — a CORE record structurally satisfies a defsurface.

(:wat::core::defsurface :geo::Shape :nature :wat::core::Struct :features [color <- :wat::core::String])
(:wat::core::defrecord :geo::Circle [color <- :wat::core::String  radius <- :wat::core::f64])
(:wat::core::defn :geo::describe [s <- :geo::Shape] -> :wat::core::String
  "ok")
(:wat::core::defn :probe::drive [] -> :wat::core::String
  (:geo::describe (:geo::Circle :color "red" :radius 2.0)))
