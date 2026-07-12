;; tests/types/probe_arc293_holder_substitution_c3.wat — case 3: holon record accepted where :wat::holon::Record wanted

(:wat::holon::defrecord :geo::HPt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :u::wants-holon [r <- :wat::holon::Record] -> :wat::holon::Record r)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:u::wants-holon (:geo::HPt :x 1 :y 2))
  nil)
