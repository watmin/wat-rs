;; GREEN — the four names the blanket census leaves, in real `:-` use.
(:wat::core::defn :user::takes-pair [p <- (wat.type/Tuple :- [wat.type/i64 wat.type/String])]
  -> :wat::core::i64 1)
(:wat::core::defn :user::nested []
  -> (wat.type/Tuple :- [(wat.type/Vector :- [wat.type/i64]) wat.type/String])
  (:wat::core::Tuple [] "s"))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "x"))
