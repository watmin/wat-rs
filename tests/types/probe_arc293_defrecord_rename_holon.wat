;; tests/types/probe_arc293_defrecord_rename_holon.wat — positive: :wat::holon::defrecord is the holon record decl head

(:wat::holon::defrecord :geo::HPt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :u::wants-holon [r <- :wat::holon::Record] -> :wat::holon::Record r)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:u::wants-holon (:geo::HPt 1 2))
  nil)
