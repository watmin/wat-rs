(:wat::core::defn :fix::demo [xs <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::let [n (:wat::core::length xs) m (:wat::core::first xs)]
    (:wat::core::do (:wat::kernel::println "a") (:wat::kernel::println "b") n)))
