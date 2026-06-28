;; tests/types/probe_arc293_holder_substitution_c2.wat — case 2: holon record widened to :wat::Record

(:wat::holon::defrecord :geo::HPt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :u::wants-record [r <- :wat::Record] -> :wat::Record r)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:u::wants-record (:geo::HPt 1 2))
  nil)
