;; tests/types/probe_arc293_holder_substitution_c1.wat — case 1: core record accepted where :wat::Record wanted

(:wat::core::defrecord :geo::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :u::wants-record [r <- :wat::Record] -> :wat::Record r)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:u::wants-record (:geo::Pt 1 2))
  nil)
