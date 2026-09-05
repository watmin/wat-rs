(:wat::core::defn :fix::u [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::do (:wat::kernel::println "a") (:wat::kernel::println "b")
    (:wat::core::+ x 1)))
