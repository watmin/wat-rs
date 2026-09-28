;; tests/types/probe_arc293_holder_substitution_c2.wat — case 2: holon record widened to :wat::core::Record

(:wat::holon::defrecord :geo::HPt [x <- wat.type/i64  y <- wat.type/i64])
(:wat::core::defn :u::wants-record [r <- wat.type/Record] -> wat.type/Record r)
(:wat::core::defn :user::main [] -> wat.type/nil
  (:u::wants-record (:geo::HPt :x 1 :y 2))
  nil)
