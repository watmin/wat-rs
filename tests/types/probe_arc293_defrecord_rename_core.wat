;; tests/types/probe_arc293_defrecord_rename_core.wat — positive: :wat::core::defrecord is the record decl head

(:wat::core::defrecord :geo::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :u::wants-pt [r <- :geo::Pt] -> :geo::Pt r)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:u::wants-pt (:geo::Pt 1 2))
  nil)
