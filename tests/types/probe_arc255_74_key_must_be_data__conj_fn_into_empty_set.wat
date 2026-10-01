(:wat::core::defn :probe::inc [x <- wat.type/i64] -> wat.type/i64 (:wat::i64::+ x 1))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let [s (:wat::core::conj #{} :probe::inc)]
    (:wat::kernel::println "fn conj'd into set")))
