;; tests/types/probe_arc293_defrecord_rename_holon.wat — positive: :wat::holon::defrecord is the holon record decl head

(:wat::holon::defrecord :geo::HPt [x <- wat.type/i64  y <- wat.type/i64])
(:wat::core::defn :u::wants-holon [r <- :wat::holon::Record] -> :wat::holon::Record r)
(:wat::core::defn :user::main [] -> wat.type/nil
  (:u::wants-holon (:geo::HPt :x 1 :y 2))
  nil)
