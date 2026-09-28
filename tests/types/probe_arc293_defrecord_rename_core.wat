;; tests/types/probe_arc293_defrecord_rename_core.wat — positive: :wat::core::defrecord is the record decl head

(:wat::core::defrecord :geo::Pt [x <- wat.type/i64  y <- wat.type/i64])
(:wat::core::defn :u::wants-pt [r <- :geo::Pt] -> :geo::Pt r)
(:wat::core::defn :user::main [] -> wat.type/nil
  (:u::wants-pt (:geo::Pt :x 1 :y 2))
  nil)
