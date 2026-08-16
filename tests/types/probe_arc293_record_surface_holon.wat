;; tests/types/probe_arc293_record_surface_holon.wat — co-located fixture (holon record positive)
;;
;; Arc 293.3-records — a HOLON record satisfies a core surface (R2 headline).

(:wat::core::defsurface :geo::Shape :nature :wat::core::Struct :features [color <- :wat::core::String])
(:wat::holon::defrecord :geo::HCircle [color <- :wat::core::String  radius <- :wat::core::f64])
(:wat::core::defn :geo::describe [s <- :geo::Shape] -> :wat::core::String
  "ok")
(:wat::core::defn :probe::drive [] -> :wat::core::String
  (:geo::describe (:geo::HCircle :color "red" :radius 2.0)))
