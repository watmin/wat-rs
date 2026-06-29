;; tests/types/probe_arc293_record_surface_missing_bad.wat — negative fixture
;;
;; Arc 293.3-records — a record MISSING a surface member is rejected.

(:wat::core::defsurface :geo::Shape :features [color <- :wat::core::String])
(:wat::core::defrecord :geo::Bare [other <- :wat::core::i64])
(:wat::core::defn :geo::describe [s <- :geo::Shape] -> :wat::core::String
  "ok")
(:wat::core::defn :user::main [] -> :wat::core::String
  (:geo::describe (:geo::Bare 5)))
