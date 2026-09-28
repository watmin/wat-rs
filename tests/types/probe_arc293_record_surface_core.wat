;; tests/types/probe_arc293_record_surface_core.wat — co-located fixture (core record positive)
;;
;; Arc 293.3-records — a CORE record structurally satisfies a defsurface.

(:wat::core::defsurface :geo::Shape :nature :wat::core::Struct :features [color <- wat.type/String])
(:wat::core::defrecord :geo::Circle [color <- wat.type/String  radius <- wat.type/f64])
(:wat::core::defn :geo::describe [s <- :geo::Shape] -> wat.type/String
  "ok")
(:wat::core::defn :probe::drive [] -> wat.type/String
  (:geo::describe (:geo::Circle :color "red" :radius 2.0)))
