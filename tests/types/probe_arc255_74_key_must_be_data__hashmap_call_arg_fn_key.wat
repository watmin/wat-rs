(:wat::core::defn :probe::inc [x <- wat.type/i64] -> wat.type/i64 (:wat::i64::+ x 1))
(:wat::core::defn :probe::as-map [m <- (wat.type/HashMap :- [[wat.type/i64 :-> wat.type/i64] wat.type/i64])]
  -> (wat.type/HashMap :- [[wat.type/i64 :-> wat.type/i64] wat.type/i64])
  m)
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let [m (:probe::as-map {:probe::inc 1})]
    (:wat::kernel::println "fn key via call-arg")))
