;; What do the four rete holon ops actually RETURN for a matching and a non-matching pair?
;; The ledger's cells need discriminating thresholds, and a threshold guessed is a cell that
;; passes for the wrong reason. Measured here, in host wat, before any generator change.

(:wat::core::defn :probe::alpha [] -> :wat::holon::HolonAST
  (:wat::holon::to-holon (:wat::core::Vector :wat::core::i64 1 2 3)))
(:wat::core::defn :probe::beta [] -> :wat::holon::HolonAST
  (:wat::holon::to-holon (:wat::core::Vector :wat::core::i64 7 8 9)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "cosine  a a:") (:wat::kernel::println (:wat::holon::cosine (:probe::alpha) (:probe::alpha)))
    (:wat::kernel::println "cosine  a b:") (:wat::kernel::println (:wat::holon::cosine (:probe::alpha) (:probe::beta)))
    (:wat::kernel::println "dot     a a:") (:wat::kernel::println (:wat::holon::dot (:probe::alpha) (:probe::alpha)))
    (:wat::kernel::println "dot     a b:") (:wat::kernel::println (:wat::holon::dot (:probe::alpha) (:probe::beta)))
    (:wat::kernel::println "coinc   a a:") (:wat::kernel::println (:wat::holon::coincident? (:probe::alpha) (:probe::alpha)))
    (:wat::kernel::println "coinc   a b:") (:wat::kernel::println (:wat::holon::coincident? (:probe::alpha) (:probe::beta)))
    (:wat::kernel::println "presen  a a:") (:wat::kernel::println (:wat::holon::presence? (:probe::alpha) (:probe::alpha)))
    (:wat::kernel::println "presen  a b:") (:wat::kernel::println (:wat::holon::presence? (:probe::alpha) (:probe::beta)))))
