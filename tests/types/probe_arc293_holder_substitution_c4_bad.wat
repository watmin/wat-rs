;; tests/types/probe_arc293_holder_substitution_c4_bad.wat — case 4: core record REJECTED where :wat::holon::Record wanted

(:wat::core::defrecord :geo::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :u::wants-holon [r <- :wat::holon::Record] -> :wat::holon::Record r)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:u::wants-holon (:geo::Pt 1 2))
  nil)
